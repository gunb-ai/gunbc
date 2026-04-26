# R2 Structure

**Status:** `PROPOSAL` — pending user sign-off + R1 closure + promotion to `ROADMAP.md` as `## Release R2 Program` section.

**Authority:** single-source while open. Amendments before promotion land in this doc. After promotion, amendments follow the same discipline as R1's `## Release R1 Program` section (director-authored PRs with manager acknowledgement).

**Scope naming note:** `docs/db-history/db-18.md` uses "R2 carrier" as internal DB-stage nomenclature that predates release-level R# naming. Our release-level R2 (this doc) is unrelated to DB-18's stage label; no collision of meaning, just of string.

## Summary

R2 is the **close-everything** release — where "everything" means *every remaining Tier-1 thesis claim that R1's gate set does not already own*. R1 closes under all-R1-gates-green (see R1 closure criteria below), and that closure carries lens purity, self-hosting shim-floor close, E-family carrier closure, and tests-as-data closure out of R2's scope entirely. R2 is what's left: **Grounding Completeness** (the single co-anchor thesis claim), joined by modeling-faithfulness dissolution, scoped substrate prereqs for that, remaining R2+ impossible-bug classes, and the §6a per-method-metadata design-call residual.

Two framing decisions drive scope + coordination:

1. **Anti-deferral principle.** If dissolution direction is clear and named, deferral is problem-finding, not problem-solving. R2 absorbs what has named dissolution directions, regardless of current execution velocity. (Velocity is a trailing observation; it can accelerate or slow between waves. The principle is what's load-bearing.)
2. **Light-touch throughput-oriented coordination.** Manager count = concurrent critical paths, not total scope.

## Program count — 2 active releases total

- **R1** closing.
- **R2** = close-everything.
- **R3** reserved as *escape hatch only*, for items that genuinely cannot close in R2 despite honest effort. Invocation should be rare and itself signal a problem worth examining — if dissolution is surfacing faster than closure, that's a leading indicator to address, not a scope-inflation signal.

Post-R2 is external work (adoption, documentation, community, ecosystem modeling) — not on the thesis-claim release ledger. The first named post-R2 stream is modeling what currently lives in `../ctrl/` (user-flagged 2026-04-24) as a practical pressure-test for whether the structural thesis R2 closes actually holds when applied to real-program shapes. That pressure-test is not itself a thesis claim; it's a validation exercise against one.

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
     
     **Excluded from this sub-lane (substrate shape differs)**: `kernel_algebra_profile` mirror dissolution (existing P2 drift ratchet at `dag.rs:1530`) is a `Map<String, AlgebraProfile>` body — **map-shaped, not list-of-sum**. Hits the same R14 `ValueBody::Unparsed` hard-fail path, but requires a different `ValueBody` extension (e.g., `ValueBody::Map(...)` variant + lowerer support for top-level map literals). Tracked separately as a sibling future T-Substrate sub-lane / future cascade item; not bundled here per substrate-shape honesty (codex BLOCKING on PR #782 caught this scope conflation; corrected 2026-04-25).
     
     Does NOT commit to the full Class 5 Gap 3 substrate-capability close; scoped to the list/sum subset for the two named consumers. ROI is 2× the original single-consumer scoping (tokenizer + Engine), not 3× as initially framed.

4. **Remaining R2+ impossible-bug classes** — three classes currently tagged `[R2+]` in `ROADMAP.md §"Lane acceptance — .dag gates"` (T-Demo row; THESIS §"Enumerable impossible-bug classes" is the authority on scheduling tags):
   - Nested-optional flatten
   - Unhandled diagnostic paths
   - Unenumerated effects

5. **§6a per-method-metadata pick** — per `docs/design-substrate-carrier-port-program.md §6a`, a deferred design call on where per-method metadata (`size_effect`, `cost_shape`, `callback_element_position` on `ordered_ring_templates()` et al.) lives. Four options in the design doc: (0) keep lens-local lookup tables; (1) substrate field-level refinements; (2) per-algebra metadata carriers; (3) unified `MethodContract` carrier. E-I pre-flight evidence has landed in R1, so the "defer until E-I evidence" trigger has fired; this is the R2 residual after T-LaneE (E-family carrier port) closes in R1. S-sized design-call close, not substrate-capability work.

6. **R2 closure demo** — simple "it runs" artifact per lane close. **R2 Release Manager-coordinated** (was Director-coordinated under prior 1-manager structure; reassigned under 2026-04-26 rework — single authority per `feedback_node_not_god_struct`). Not a lane — see Demo discipline below.

## Manager structure

> **🔄 REVISED 2026-04-26.** The original "1 standing manager (Grounding) + Director ad-hoc" decision is **retracted.** Empirical signal: under that structure, Grounding Manager and (the prior) Zero-Floor Manager were idle while Director became the dispatch bottleneck for every other lane. Standing managers without owned deliverables degenerate into pass-through hops; concentrating brief-authoring on Director starved 3 of 4 non-Grounding lanes. Lesson saved as feedback (`feedback_standing_managers_need_owned_deliverables`). Restoring R1's program-manager pattern: **N managers, each owning a complete program with autonomous brief-authoring + dispatch authority through R2 close.** Director coordinates *across* programs but does not author every brief.

**6 standing managers + Director coordinator.** Each manager owns a mutually-exclusive program with its own deliverables, sub-briefs, and worker dispatch through R2 completion. Director's role is cross-program coordination (handoffs, conflict resolution, scope-change escalation), not brief authoring.

**Cross-manager dependency discipline.** Where one program produces a substrate carrier another program consumes (e.g., T-Substrate ValueBody-list/sum unblocks T-Modeling tokenizer charclass + T-Ground Engine sharpened-(b)), the producing manager owns the carrier landing; consuming managers own the migration. Cross-program handoffs land via the R1 `Cross-manager notifications queued` brief pattern — producing manager signals readiness; consuming managers ack and dispatch consumer-migration work.

### 1. Grounding Manager

Owns **T-Ground** sub-program (Goal 1 — Grounding Completeness, the program with R2's only true critical path).

- **Critical path:** T-Ground-Pilot → T-Ground-Rust → T-Ground-Engine → T-Ground-Tests → T-Ground-Dissolve (per `ROADMAP.md §"Post-R1 Grounding lanes"`).
- **Fill queue:** T-Ground-Python, T-Ground-Go (2-way parallel after Pilot validates).
- **Cross-program consumer:** Engine sharpened-(b) full pilot enumeration consumes T-Substrate ValueBody-list/sum carrier.
- **Authority:** authors all T-Ground sub-briefs autonomously; dispatches workers; reports cross-program signals to Director.

### 2. Substrate Manager

Owns **T-Substrate** lane (Goal 3 — substrate prereqs) **and the B4 Identity-Carrier Substrate Pass program** (from #810 synthesis; see `docs/briefs/b4-identity-carrier-substrate-pass.md`). The largest single concentration of R2 work; also the program that unblocks the most consumers.

- **T-Substrate sub-lanes:**
  - Cardinality-substrate subset for int-literal magnitude (unblocks Modeling Manager's int-lit item)
  - Nominal-opaque subset for `Secret<T>` (unblocks Modeling Manager's `Secret<T>` item)
  - Parametric-algebra-attachment subset for `Dimension<Carrier>` (unblocks Modeling Manager's Dimensions item)
  - Top-level `ValueBody` list/sum + `std.unicode` bootstrap (unblocks Modeling Manager's tokenizer charclass phase-2 + Grounding Manager's Engine sharpened-(b))
- **B4 Identity-Carrier Substrate Pass:** the 4 Phase 1 carriers (`DeclarationRef` consumer migration, fold-shape carrier, emit-helper carrier, extdeps-fixture-set carrier) and the 8 Phase 2 site dissolutions. Sub-briefs B4.1 through B4.12.
- **Watch condition (split trigger):** Substrate Manager's combined T-Substrate + B4 scope is intentionally heavy (8+ parallel slots). If Substrate becomes the new bottleneck the way Director was — measured as workers idle waiting for Substrate-authored briefs >7 days — split B4 into a dedicated standing **B4 Identity-Carrier Manager**. R2 Release Manager surfaces this signal via the velocity-tripwire reporting if it fires.
- **Cross-program producer:** all four T-Substrate sub-lanes produce carriers consumed by Modeling Manager + Grounding Manager.
- **Authority:** authors all T-Substrate + B4 sub-briefs autonomously; dispatches workers; signals readiness to consuming managers via cross-manager queue.

### 3. Modeling Manager

Owns **T-Modeling** lane (Goal 2 — modeling-faithfulness dissolution) and consumer-side migrations of T-Substrate carriers.

- **Items:**
  - Surface int-literal magnitude at concept layer (consumes T-Substrate cardinality subset)
  - `Secret<T>` nominal-opaque graduation (consumes T-Substrate nominal-opaque subset)
  - `Dimension<Carrier>` typed value wrapper with phantom-parameter unit-mismatch enforcement (consumes T-Substrate parametric-algebra subset)
  - Tokenizer charclass phase-2 (consumes T-Substrate ValueBody-list/sum subset)
- **Cross-program consumer:** waits on Substrate Manager's per-sub-lane readiness signals; each item dispatches as its substrate dependency lands.
- **Authority:** authors all T-Modeling sub-briefs autonomously; dispatches workers; signals item-close to **R2 Release Manager** for R2 demo coordination.

### 4. Impossible-Bugs Manager

Owns **T-ImpossibleBugs** lane (Goal 4 — remaining R2+ impossible-bug classes per `THESIS.md §"Enumerable impossible-bug classes"`).

- **Classes:**
  - Nested-optional flatten (gated on cardinality refinement; coordinate with Substrate Manager)
  - Unhandled diagnostic paths (Tier 2 substrate; coordinate with Substrate Manager if substrate work surfaces)
  - Unenumerated effects (post-effects-design-doc per #808; closed-system effects model is the canonical reference)
- **Cross-program coordination:** classes that surface substrate gaps escalate to Substrate Manager rather than expanding T-ImpossibleBugs scope.
- **Authority:** authors all T-ImpossibleBugs sub-briefs autonomously; dispatches workers; signals class-close to Director.

### 5. Pure Bootstrap Manager

Owns the **post-R1 work** of the Pure Bootstrap to Zero program (per `docs/design-pure-bootstrap-zero.md` LIVE 2026-04-25), continuing the multi-release dissolution toward 0-floor that the cascade promotion established. Replaces the prior idle Zero-Floor Manager with a manager that has owned deliverables and pull-not-push intake.

- **R1 vs R2 split (resolves the gate-vs-program ambiguity codex flagged):** R1's PB gate predicates (`pb_hand_rust_at_shim_floor`, `pb_compiler_std_ratchet_zero`, `pb_rust_tests_outside_residual_zero`, `lens_producer_files_remaining`) are `[ext]` — their thresholds read from the `sg0_census_test.rs` baseline at R1 close, **not** frozen at 0. R1 closes when those gates are green at the census-baked threshold (whatever it is at R1 close). The full 0-floor target is the Pure Bootstrap program's multi-release dissolution scope, which continues into R2 here. **R1 owns the gate close at the current threshold; R2 owns continued census reduction toward 0.** This matches what the cascade promotion did de facto when it promoted PB to LIVE (a standing program, not a one-time R1 gate).
- **R2 lanes:**
  - **Continued T-PB-A census reduction** post-R1: SG-0 census shrinks toward 0 (file-level + fragments), including the lens-producer priority slice. Each PR shrinks `EXPECTED_HAND_AUTHORED_NON_TEST` or `EXPECTED_HAND_AUTHORED_FRAGMENTS` by ≥1.
  - **Continued T-PB-B census reduction** post-R1: Rust-authored test census shrinks toward 0 via `ExecuteCommand`-based `.dag` `TestClaim` migration. PB-Runtime enabler landed (PR #792).
  - **Tier 2 carry-from-#810:** `patch_lower_helpers_generated_type_alias_refinement` retirement (PB-Tier1 priority hint per #810 §5).
  - **Mirror dissolutions:** termination/computation/induction/effect-carrier Rust mirrors (Tier 3 #10 + #12 from #810; dissolve as v3 lowers/evaluates `.dag` runtime values).
- **Cross-program coordination:** B4's §0.7 file-preference rank carrier touches PB territory; coordinate with Substrate Manager on shared substrate dependencies.
- **Authority:** authors all PB-* sub-briefs autonomously; dispatches workers; reports SG-0 census deltas to Director and to R2 Release Manager (for closure ledger).

### 6. R2 Release Manager

Owns release coordination + the smallest cross-cutting deliverables (Goal 5, Goal 6, the #810 discipline framework, B-wave Tier 0 dispatch coordination, R2 demo).

- **Owned deliverables:**
  - **Goal 5:** §6a per-method-metadata pick (S-sized design call; carrier choice from the 4 options in `docs/design-substrate-carrier-port-program.md §6a`).
  - **Goal 6:** R2 closure demo coordination — surface "it runs" artifacts at each lane close per Demo discipline section.
  - **B-wave Tier 0 coordination:** B1 (#820) / B2 (#817) / B3 (#821) implementation through-merge, including any audit-narrative iteration. Dispatch B5 (Loop construction-closure audit) + B6 (file-preference rank checklist fix) + B7 (priority-hint relay to Pure Bootstrap Manager).
  - **Discipline framework enforcement:** track velocity-tripwire ratio per integration-reflection cadence (per INVARIANTS.md §P5 "Dispatch-Discipline Mechanisms"); surface ≥3:1 readings to Director.
  - **Thesis-claim coverage mapping** (Open call 1 below): authors the table on R1 closure → R2 promotion transition.
  - **R2 closure ledger:** tracks lane-close green status; surfaces unblocked work to idle workers; coordinates v2-retirement post-R2.
- **Authority:** authors briefs for owned deliverables autonomously; dispatches workers; coordinates demo cadence with all other managers.

### Director (cross-program coordinator)

- **Cross-program conflict resolution.** When Substrate Manager's carrier shape conflicts with Modeling Manager's consumer needs (or any analogous cross-program collision), Director arbitrates.
- **Scope-change escalation.** If a manager discovers their program needs to expand (e.g., a class-of-pattern dissolution surfacing under what was a single-item brief), Director adjudicates whether to expand the program or split a new lane.
- **R1 residual closure surveillance** (none expected per all-R1-gates-green closure criterion).
- **Weekly dependency health check:** which lanes are within 1 step of unblocking? Which managers are blocked on cross-program signals? Surface to user when a program goes >7 days without lane-close.
- **What Director no longer does:** authoring sub-briefs for individual lane work. That returns to the responsible manager. Director's bandwidth is conserved for cross-program coordination.

## Lane structure

| Lane | Size | Manager | Covers |
|---|---|---|---|
| T-Ground | XL | **Grounding Manager** | Full T-Ground-* sub-program (Goal 1) — Pilot/Rust/Engine/Tests/Dissolve critical path + Python/Go fill |
| T-Substrate | XL | **Substrate Manager** | Four T-Substrate scoped-subset sub-lanes (Goal 3) **plus the B4 Identity-Carrier Substrate Pass program** (4 Phase 1 carriers + 8 Phase 2 site dissolutions, sub-briefs B4.1–B4.12, per `docs/briefs/b4-identity-carrier-substrate-pass.md`). Largest single program in R2; produces carriers consumed by Modeling Manager (3 sub-lanes) + Grounding Manager (Engine sharpened-(b)). Note: `kernel_algebra_profile` mirror dissolution is map-shaped — tracked separately as a future T-Substrate sub-lane requiring `ValueBody::Map` substrate work. |
| T-Modeling | M | **Modeling Manager** | int-lit magnitude / `Secret<T>` graduation / `Dimension<Carrier>` (Goal 2) **plus tokenizer charclass phase-2** (consumer of T-Substrate ValueBody-list/sum sub-lane). Each item dispatches as its T-Substrate dependency lands. |
| T-ImpossibleBugs | S | **Impossible-Bugs Manager** | nested-optional flatten / unhandled-diagnostic-paths / unenumerated-effects (Goal 4). Substrate-gap discoveries escalate to Substrate Manager. |
| T-PB | XL | **Pure Bootstrap Manager** | **Post-R1 continuation** of Pure Bootstrap to Zero program (R1 closes PB gates at `[ext]` `sg0_census_test.rs` thresholds; R2 continues SG-0 census reduction toward 0-floor target). Covers continued T-PB-A non-test hand-Rust reduction + lens-producer priority slice + continued T-PB-B Rust-authored test reduction + Tier 2 `patch_lower_helpers_*` retirement + termination/computation/induction/effect-carrier mirror dissolutions (Tier 3 #10 + #12 from #810). See R1 closure criteria for the gate-vs-program split. |
| T-Release | M | **R2 Release Manager** | §6a per-method-metadata pick (Goal 5) + R2 demo coordination (Goal 6) + B-wave Tier 0/2 dispatch (B1/B2/B3 through-merge, B5/B6/B7 authoring) + #810 discipline framework enforcement (velocity tripwire reporting) + thesis-claim coverage mapping (Open call 1) + R2 closure ledger + v2 retirement coordination. |

**Goal 6 (R2 closure demo) is not a lane.** It is a cross-lane closure discipline (see "Demo discipline" below): each lane's closure PR ships its own simple "it runs" artifact; **R2 Release Manager coordinates surfacing** (single authority per the 2026-04-26 rework). No separate T-Demo lane owner, no separate demo-authoring critical path.

**Lanes deliberately absent (R1 gates, closed by R1 lane acceptance):**
- T-LensMigration / `lens_producer_files_remaining` — R1 T-PB-A gate per PR #752. **Cascade-promotion update 2026-04-25 + R2-rework 2026-04-26:** Pure Bootstrap to Zero program (LIVE per `docs/design-pure-bootstrap-zero.md`) owns the lens-producer migration as part of its PB-* lane structure. R1 retains the gate at its `[ext]` threshold; **R2 Pure Bootstrap Manager owns the post-R1 continued reduction toward 0**. Not absent from R2 — the gate closes in R1, the program work continues in R2.
- ~~T-ShimFloor / `pb_hand_rust_at_shim_floor` / `pb_compiler_std_ratchet_zero` / `pb_rust_tests_outside_residual_zero` — R1 T-PB-A + T-PB-B gates.~~ **Cascade-promotion update 2026-04-25 + R2-rework 2026-04-26:** Pure Bootstrap to Zero program owns all shim-floor work; the program target is 0 (per ROADMAP T-PB-A row amendment in cascade promotion PR). R1's PB gate predicates close at their `[ext]` `sg0_census_test.rs` thresholds; **R2 Pure Bootstrap Manager owns continued census reduction post-R1**. Not absent from R2 — gates green in R1, dissolution work continues in R2 toward 0.
- T-EFamilyClose — R1 T-LaneE's critical-path carrier work (E-T, E-C, E-I, E-P, E-M sub-lanes), enabling the R1 `complexity_merge_sort_is_nlogn` + `complexity_merge_sort_v3_matches_v2_oracle` + `lane_e_bundled_witness_host_emit_parity` gates. All E-family carrier-port work closes in R1; only the §6a metadata-pick residual inherits to R2 (Goal 5).
- T-TestGen-tail (`testgen_mock_backed_integration_safe` / `MockBackedInvariant` wiring) — R1 T-TestGen gate per `ROADMAP.md §"Lane acceptance — .dag gates"`. Closes in R1.

R2 does not re-own R1 gate close authority; under all-R1-gates-green criterion, those gates ARE the close conditions. R2 inherits **post-gate program continuation** for two of the above (PB lens-producer + PB-A/PB-B census reduction), plus two named exceptions for non-PB scope:

1. **Goal 5's §6a per-method-metadata pick** — was not an R1 gate; deferred design call inherits to R2.
2. **`sub_charclass_in_std_unicode` phase-2** — was an R1 T-Sub gate, but reclassified to R2 substrate-capability per ROADMAP amendment (2026-04-24) following Surface Manager's handoff that the remaining work is Class 5 Gap 3 substrate-capability scope, not T-Sub-only surface fix. Now a 4th sub-lane under R2 T-Substrate (Goal 3); see lane row above.

Plus the **post-R1 PB program continuation** owned by R2 Pure Bootstrap Manager (per the gate-vs-program resolution in R1 closure criteria above): R1's PB gates close at their `[ext]` thresholds, R2 continues SG-0 census reduction toward the cascade-promoted 0-floor target.

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

Pure Bootstrap Manager (T-PB):
    T-PB-A (parallel slot per file): SG-0 census → 0
        Lens-producer priority slice    (`lens_producer_files_remaining` gate)
        Tier 2 patch_lower_helpers_* retirement
        Mirror dissolutions: termination/computation/induction/effect-carrier
    T-PB-B (parallel slot per test class): Rust-authored tests → 0
        ExecuteCommand-based .dag TestClaim migration (PB-Runtime / PR #792 enabler landed)
    Cross-program: B4's §0.7 file-preference rank carrier touches PB territory; coordinate with Substrate Manager.

R2 Release Manager (T-Release):
    Cross-cutting (parallel-dispatchable):
        §6a per-method-metadata pick    (Goal 5; design-call close)
        B-wave Tier 0 through-merge     (B1/B2/B3 implementation iteration)
        B-wave Tier 2 brief authoring   (B5 Loop construction-closure audit; B6 checklist fix; B7 priority hint)
        Discipline-framework enforcement (velocity tripwire reporting per cadence)
        Thesis-claim coverage mapping   (Open call 1; on R1 close → R2 promotion)
        R2 demo coordination            (Goal 6; surface artifacts at each lane close)
        v2-retirement coordination      (post-R2; tracked but not gated)

Director (cross-program coordinator):
    Conflict resolution + scope-change escalation only — no brief authoring.
```

**Parallel-capable work at steady state:** Grounding (1 critical-path slot + 2 fill) + Substrate (4 T-Substrate sub-lanes + 4 Phase 1 B4 carriers; up to 8 parallel) + Modeling (4 items, each pair-blocked on Substrate readiness) + ImpossibleBugs (3 independent classes) + PB (parallel per file/test) + Release (cross-cutting, 5+ parallel cross-cutting items). **Aspirational dispatch ceiling: ~20+ concurrent worker slots across 6 programs** (capacity, not committed throughput — actual concurrency depends on idle-worker availability and cross-program unblock timing), vs. ~9–13 under the prior 1-manager structure where Director was the brief-authoring bottleneck.

## R1 closure criteria

**All R1 gates green at their `[ext]` thresholds.** R1 closes when all 9 lane gates named in `ROADMAP.md §"Lane acceptance — .dag gates"` evaluate green, including omni-emit (`emit_omni_demo_fixtures_green`), the T-PB-A self-hosting gates (including `lens_producer_files_remaining` added via PR #752), the T-PB-B tests-as-data gates, and the T-LaneE complexity-lens gates (which ride on the E-family carrier-port chain). No director-defined subset-close.

**PB gates are `[ext]` — threshold reads from authority, not frozen at 0 in this doc.** The PB-A / PB-B / `lens_producer_files_remaining` predicates pull thresholds from `src/v3/compiler/tests/integration/sg0_census_test.rs` at evaluation time. R1 closes on those gates green at their **then-current** `sg0_census_test.rs` baseline. The full 0-floor target (per cascade-promoted `docs/design-pure-bootstrap-zero.md`) is the multi-release dissolution scope owned by **Pure Bootstrap Manager** in R2 — R1 owns the gate close at threshold; R2 owns continued census reduction toward 0. This is the structural resolution of the gate-vs-program ambiguity (codex P2 inline on PR #827, sha `bf9a05c7`).

Rationale: consistent with anti-deferral stance — tail-shaped work closes before R1 declares done; R2 doesn't inherit R1 residuals. Consequence: R2 scope does NOT include Lens Purity by Construction, self-hosting shim-floor close, or E-family carrier closure — those are all R1 gate concerns, closed by R1's own acceptance criterion. R2 is free to focus on the thesis claims R1's gates don't cover.

## Transition mechanics

1. **R1 gates green** → Director declares R1 closed.
2. **R1 residual sweep** — every open R1 ledger row gets an R1-or-R2 assignment. No orphaning. Done in the R1 closure PR. Expected to be short under all-gates-green criterion.
3. **R1 manager dissolution** — all R1 standing managers (Surface, Substrate, Testgen, Self-hosting, Release) archive with closure banners. Their scopes are fully absorbed by R1's gate acceptance; no inheritance into R2 managers.
4. **R2 manager spin-up** — six standing managers initialized per the revised manager structure above. Each manager gets a dedicated brief (`docs/briefs/r2-grounding-manager.md`, `docs/briefs/r2-substrate-manager.md`, `docs/briefs/r2-modeling-manager.md`, `docs/briefs/r2-impossible-bugs-manager.md`, `docs/briefs/r2-pure-bootstrap-manager.md`, `docs/briefs/r2-release-manager.md`) naming program scope + owned deliverables + cross-program dependencies + autonomous dispatch authority + reporting cadence. **Pre-stage skeleton briefs before R1 closes** (Director authors skeletons during R1 final week; promotion PR fills in scope-final details) so the R1→R2 transition is not gated on six fresh authoring cycles. Existing `docs/briefs/grounding-manager.md` migrates content into `r2-grounding-manager.md` and archives. Existing `docs/briefs/pure-bootstrap-zero-manager.md` migrates content into `r2-pure-bootstrap-manager.md` and archives.
5. **R2 open** — this doc promotes to `ROADMAP.md` as `## Release R2 Program` section.

**v2 retirement (explicit non-scoping note):** The v2 compiler at `src/v2/` persists as test oracle into R2 and is NOT on the R2 ledger. Its retirement is external post-R2 operational cleanup — no thesis claim depends on v2 being absent. Differential-test-oracle retirement is bounded by adoption/documentation concerns, not release-gate discipline.

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
- ~~**Manager count = 1 + Director.** One standing manager (Grounding) matches R2's single critical path. T-Modeling / T-Substrate / T-ImpossibleBugs are parallel-capable with no critical-path coordination pressure; Director dispatches directly. Adjustable upward if parallel fill-queue depth becomes unmanageable in practice.~~ **🔄 RETRACTED 2026-04-26.** Empirically wrong: standing managers without owned deliverables sat idle while Director became the dispatch bottleneck for every other lane, starving 3 of 4 non-Grounding lanes. R1's program-manager pattern restored — see "Manager structure" above for the 6-manager structure (each owning a complete program with autonomous brief-authoring + dispatch authority through R2 close). Lesson saved as `feedback_standing_managers_need_owned_deliverables`.
- **Manager count = 6 standing managers + Director coordinator.** Grounding / Substrate / Modeling / Impossible-Bugs / Pure Bootstrap / R2 Release. Each owns a mutually-exclusive program; cross-program dependencies handled via R1 `Cross-manager notifications queued` brief pattern. Director's role is cross-program conflict resolution + scope-change escalation, not brief authoring. (Locked 2026-04-26 amendment per user direction; supersedes prior 1-manager decision.)
- **R2 includes substrate prereqs explicitly** per user's (i)-over-(ii) preference (honest scope over tight scope), with **scoped acceptance criteria** per Director refinement (each sub-lane closes on unblock of its paired Goal 2 item; full substrate-capability lanes are not R2-committed).
- **Anti-deferral principle is the frame, not velocity numbers.** Per Director observation: 16-hour R1 execution was a peak-day sample, not a baseline. The principle "if dissolution direction is clear and named, deferral is problem-finding not problem-solving" is what survives cadence shifts.
- **`sub_charclass_in_std_unicode` phase-2 reclassified R1 → R2** (2026-04-24, ROADMAP amendment paired with this revision). Surface Manager handoff (sub-child `quiet-gull-882` triage) confirmed the remaining work is Class 5 Gap 3 substrate-capability scope, not T-Sub-only surface fix; reclassifying lets R1 close on T-Sub Day-1 + DB-11 without waiting on substrate-capability work. Phase-2 lands in R2 as a 4th T-Substrate scoped sub-lane (consumer: tokenizer). Reclassification scope is bounded — full Class 5 Gap 3 substrate-capability close remains outside R2 unless additional R2 items demand it; only the tokenizer-charclass-unblock subset is R2-committed.
- **Post-R2 stance = STRONG.** R2 is thesis close. Post-R2 work is external (adoption, documentation, community, ecosystem buildout). R3 is reserved as escape hatch only, not as a structural release program. User-locked 2026-04-24 after pressure-test framing: *"real programs are probably required to confirm the thesis is real, but we can keep R2 theoretical to stay fair — future work would pressure test the claims."* The R2 doc's "close-everything" claim is thus scoped to the **structural thesis** (what the compiler proves by construction); practical validation via modeling real programs (e.g., the user's `../ctrl/` follow-up) is a separate post-R2 stream that *tests whether the thesis holds in practice*, without itself being a thesis claim.

## Open calls

### 1. Pre-promotion thesis-claim coverage mapping (gate before ROADMAP promotion)

Surfaced by codex API review on `6fdd8341`: the "close-everything/post-R2-external-only" framing requires an explicit mapping from THESIS tiers to concrete R1/R2/post-R2 disposition, so no thesis claim is implicitly-positioned. Otherwise "close-everything" is an assertion without audit.

THESIS authority (`THESIS.md:155-182`) lists:
- **Tier 1 — Structural correctness** (type mismatches, CX termination, coercion = emission, ownership no-alias, **Grounding completeness**).
- **Tier 2 — Runtime safety** (division-by-zero, integer overflow, out-of-bounds, force-unwrap, partial functions — proven safe or made total).
- **Tier 3 — Verification from structure** (L4 emitted ≡ .dag, L5 cross-target parity, L6 structural-form coverage, L7 algebraic laws).

**Required before promotion:** a table mapping every Tier-1 / Tier-2 / Tier-3 claim to its R1-closed / R2-gated / post-R2-external disposition, with any gaps (claim named in THESIS but not mapped) flagged as pre-promotion blockers. Non-blocking for this PR; blocking for ROADMAP promotion.

**Audit format:** table with columns `Tier | Claim | Disposition (R1 / R2 / post-R2-external) | Gate or lane name | Evidence (PR# or gate name) | Status`. Gaps are rows with disposition column empty or claim not in THESIS list.

**Ownership:** Director authors as a sibling PR. PM reviews for completeness against `THESIS.md §"Thesis claims — complete list"`.

**Timeline:** lands as part of the R1 closure → R2 promotion transition (step 4 in Transition mechanics above). Not a separate program.

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
- Substrate design: `docs/design-substrate-carrier-port-program.md` (E-family lanes + §6a per-method-metadata).
- Self-hosting anchor: [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) (LIVE 2026-04-25; 0-floor target + SG census). Supersedes [`docs/design-pure-bootstrap.md`](design-pure-bootstrap.md) (now SUPERSEDED; ≤5-floor framing retracted).
- Thesis: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags authority); `THESIS.md §"Thesis claims — complete list"` (Tier-1 claim lineage).
- Lens capability: `docs/v3-lens-capability-register.md` (per-lens capability tracking).
- DB history: `docs/db-history/db-18.md` (DB-18 Part-2 shipped: workflow-effect carrier + Rust reflection; Part-3 queued: Go accessor). Note: `ROADMAP.md §"Post-R1 Program — Grounding Completeness"` tags "DB-18 parametric algebra attachment" as a post-R1 blocker; that label is not obviously aligned with db-history's DB-18 scope — a pre-promotion rename or new DB number may be warranted for the R2 parametric-algebra prereq.
- Related PRs: #745 (P4 int-literal row — substrate motivation for T-Modeling), #752 (T-PB-A lens-producer priority slice — R1 gate, not R2).
