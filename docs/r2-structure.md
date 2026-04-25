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

**Gate-ownership discipline.** Every R1 gate listed in `ROADMAP.md §"Lane acceptance — .dag gates"` closes in R1 under the locked all-R1-gates-green criterion (see R1 closure criteria below). That means concerns gated there — **lens purity** (`lens_producer_files_remaining` on T-PB-A via PR #752), **self-hosting shim-floor close** (T-PB-A `pb_hand_rust_at_shim_floor` + `pb_compiler_std_ratchet_zero` + T-PB-B `pb_rust_tests_outside_residual_zero`), and **E-family carrier port closure** (the T-LaneE critical path enabling `complexity_merge_sort_is_nlogn` + `complexity_v3_matches_v2_oracle`) — are **R1 scope, not R2**. R2 does not duplicate release authority over gates ROADMAP already assigns to R1 lanes.

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
   - **Top-level `ValueBody` list/sum subset + `std.unicode` bootstrap inclusion sufficient to close `sub_charclass_in_std_unicode` phase-2** — enough Class 5 Gap 3 substrate capability for `data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]` to lower structurally (rather than fall to `ValueBody::Unparsed` and trigger R14 hard-fail), plus the `Dag::new()` bootstrap/load-set decision that makes `std.unicode::CharClass` resolvable from `tokenize.dag`. Consumer: tokenizer (R1 T-Sub deferred this phase to substrate per Surface Manager handoff 2026-04-24; reclassified to R2 per ROADMAP amendment). Does NOT commit to the full Class 5 Gap 3 substrate-capability close (other top-level `ValueBody` consumers may need additional variants beyond list/sum); scoped to what unblocks the tokenizer charclass row only.

4. **Remaining R2+ impossible-bug classes** — three classes currently tagged `[R2+]` in `ROADMAP.md §"Lane acceptance — .dag gates"` (T-Demo row; THESIS §"Enumerable impossible-bug classes" is the authority on scheduling tags):
   - Nested-optional flatten
   - Unhandled diagnostic paths
   - Unenumerated effects

5. **§6a per-method-metadata pick** — per `docs/design-substrate-carrier-port-program.md §6a`, a deferred design call on where per-method metadata (`size_effect`, `cost_shape`, `callback_element_position` on `ordered_ring_templates()` et al.) lives. Four options in the design doc: (0) keep lens-local lookup tables; (1) substrate field-level refinements; (2) per-algebra metadata carriers; (3) unified `MethodContract` carrier. E-I pre-flight evidence has landed in R1, so the "defer until E-I evidence" trigger has fired; this is the R2 residual after T-LaneE (E-family carrier port) closes in R1. S-sized design-call close, not substrate-capability work.

6. **R2 closure demo** — simple "it runs" artifact per lane close. Director-coordinated. Not a lane — see Demo discipline below.

## Manager structure

**1 standing manager + Director.** Count = concurrent critical paths. R2 has one: Grounding's `Pilot → Rust → Engine → Tests → Dissolve`. All other R2 work (Goals 2–5) is parallel modeling-faithfulness / substrate / impossible-bug / metadata-pick closure with no critical-path coordination pressure — Director dispatches workers directly against the shared fill queue.

**Cross-manager notifications convention.** The R1 brief pattern of `Cross-manager notifications queued` sections continues: Grounding Manager's brief carries one; Director surfaces any parallel-lane blockers or dependencies on the Grounding critical path through the same channel. With only one standing manager in R2, the convention degenerates in practice to "Grounding Manager → Director" and "Director → Grounding Manager" signals.

### Grounding Manager

Continues `docs/briefs/grounding-manager.md` (refreshed for R2 scope on promotion). Owns T-Ground sub-program.

- **Critical path:** T-Ground-Pilot → T-Ground-Rust → T-Ground-Engine → T-Ground-Tests → T-Ground-Dissolve (per `ROADMAP.md §"Post-R1 Grounding lanes"` — Rust is on the critical path because Engine blocks on layers 1–3 populated and Rust is the first layer-populating target).
- **Fill queue:** T-Ground-Python, T-Ground-Go (2-way parallel after Pilot validates; run alongside Rust but are not gated by Engine-blocking).

### Director (ad-hoc)

- R1 residual closure surveillance (none expected per all-R1-gates-green closure criterion).
- **R2 non-Grounding lane dispatch:** T-Modeling (Goal 2), T-Substrate (Goal 3), T-ImpossibleBugs (Goal 4), T-PerMethodMetadata (Goal 5). All parallel-capable — no critical path among them and no critical-path relationship to Grounding — so no standing manager is justified. Director picks top-priority unblocked work for any idle worker.
- R2 demo coordination: surfaces "it runs" artifacts at each lane close to user.
- Weekly dependency health check: which lanes are within 1 step of unblocking? Which workers are on fill vs. ready?

## Lane structure

| Lane | Size | Manager | Covers |
|---|---|---|---|
| T-Ground | XL | Grounding | Full T-Ground-* sub-program (Goal 1) |
| T-Modeling | M | Director (ad-hoc) | int-lit / Secret<T> / Dimensions (Goal 2) |
| T-Substrate | M | Director (ad-hoc) | Four scoped-subset sub-lanes (Goal 3): cardinality-for-int-lit; nominal-opaque-for-Secret; parametric-algebra-attachment-for-Dimensions; top-level-ValueBody-list/sum + std.unicode-bootstrap-for-tokenizer-charclass — each scoped to its paired R2 consumer (T-Modeling × 3 + tokenizer × 1), not full substrate-capability |
| T-ImpossibleBugs | S | Director (ad-hoc) | nested-optional flatten / unhandled-diagnostic-paths / unenumerated-effects (Goal 4) |
| T-PerMethodMetadata | S | Director (ad-hoc) | §6a per-method-metadata carrier pick (Goal 5) — design-call close, not substrate-capability work |

**Goal 6 (R2 closure demo) is not a lane.** It is a cross-lane closure discipline (see "Demo discipline" below): each lane's closure PR ships its own simple "it runs" artifact; Director coordinates surfacing. No separate T-Demo lane owner, no separate demo-authoring critical path.

**Lanes deliberately absent (R1 gates, closed by R1 lane acceptance):**
- T-LensMigration / `lens_producer_files_remaining` — R1 T-PB-A gate per PR #752.
- T-ShimFloor / `pb_hand_rust_at_shim_floor` / `pb_compiler_std_ratchet_zero` / `pb_rust_tests_outside_residual_zero` — R1 T-PB-A + T-PB-B gates.
- T-EFamilyClose — R1 T-LaneE's critical-path carrier work (E-T, E-C, E-I, E-P, E-M sub-lanes), enabling the R1 `complexity_merge_sort_is_nlogn` + `complexity_v3_matches_v2_oracle` gates. All E-family carrier-port work closes in R1; only the §6a metadata-pick residual inherits to R2 (Goal 5).
- T-TestGen-tail (`testgen_mock_backed_integration_safe` / `MockBackedInvariant` wiring) — R1 T-TestGen gate per `ROADMAP.md §"Lane acceptance — .dag gates"`. Closes in R1.

R2 does not re-own any of the above; under all-R1-gates-green R1 closure, those gates ARE the close conditions and R2 inherits nothing there — with two named exceptions:

1. **Goal 5's §6a per-method-metadata pick** — was not an R1 gate; deferred design call inherits to R2.
2. **`sub_charclass_in_std_unicode` phase-2** — was an R1 T-Sub gate, but reclassified to R2 substrate-capability per ROADMAP amendment (2026-04-24) following Surface Manager's handoff that the remaining work is Class 5 Gap 3 substrate-capability scope, not T-Sub-only surface fix. Now a 4th sub-lane under R2 T-Substrate (Goal 3); see lane row above.

## Dependency DAG

```
T-Ground:         Pilot → Rust → Engine → Tests → Dissolve   (critical path)
                  Python, Go run parallel after Pilot (fill queue; not Engine-blocking)
T-Substrate:      cardinality-for-int-lit (subset) ──→ unblocks T-Modeling int-lit
                  nominal-opaque-for-Secret (subset) ─→ unblocks T-Modeling Secret<T>
                  parametric-algebra-for-Dimensions (subset) ─→ unblocks T-Modeling Dimensions
                  ValueBody-list/sum + std.unicode-bootstrap (subset) ─→ unblocks tokenizer charclass phase-2
T-Modeling:       int-lit      ← T-Substrate cardinality-for-int-lit
                  Secret<T>    ← T-Substrate nominal-opaque-for-Secret
                  Dimensions   ← T-Substrate parametric-algebra-for-Dimensions
Tokenizer charclass phase-2 (consumer of T-Substrate's 4th subset; not a peer lane —
                  consumed inside T-Substrate scope, deliverable owned by tokenizer code):
                  retype to Char / List<Char> / CharClass ← T-Substrate ValueBody-list/sum + std.unicode-bootstrap
T-ImpossibleBugs: 3 independent classes (any worker)
T-PerMethodMetadata: §6a pick (any worker; independent)
(Goal 6 demo artifacts ship with each lane's closure PR — not a
 separate dependency-DAG node; see Demo discipline section.)
```

Parallel-capable work at any time: Grounding has 2 fill slots (Python, Go) alongside its critical path; Director dispatch has 3 T-Substrate sub-lanes + 3 T-Modeling items (each pair-blocked) + 3 T-ImpossibleBugs classes + 1 T-PerMethodMetadata pick, for roughly 7–10 slots depending on T-Substrate unblock timing.

## R1 closure criteria

**All R1 gates green.** R1 closes when all 9 lane gates named in `ROADMAP.md §"Lane acceptance — .dag gates"` evaluate green, including omni-emit (`emit_omni_demo_fixtures_green`), the T-PB-A self-hosting gates (including `lens_producer_files_remaining` added via PR #752), the T-PB-B tests-as-data gates, and the T-LaneE complexity-lens gates (which ride on the E-family carrier-port chain). No director-defined subset-close.

Rationale: consistent with anti-deferral stance — tail-shaped work closes before R1 declares done; R2 doesn't inherit R1 residuals. Consequence: R2 scope does NOT include Lens Purity by Construction, self-hosting shim-floor close, or E-family carrier closure — those are all R1 gate concerns, closed by R1's own acceptance criterion. R2 is free to focus on the thesis claims R1's gates don't cover.

## Transition mechanics

1. **R1 gates green** → Director declares R1 closed.
2. **R1 residual sweep** — every open R1 ledger row gets an R1-or-R2 assignment. No orphaning. Done in the R1 closure PR. Expected to be short under all-gates-green criterion.
3. **Manager dissolution** — all R1 standing managers (Surface, Substrate, Testgen, Self-hosting) archive with closure banners. Their scopes are fully absorbed by R1's gate acceptance; no inheritance into R2 managers. Grounding Manager remains, refreshed for R2 scope.
4. **R2 open** — this doc promotes to `ROADMAP.md` as `## Release R2 Program` section. `docs/briefs/grounding-manager.md` refreshed for R2 scope. No new manager briefs authored (Director dispatches T-Modeling / T-Substrate / T-ImpossibleBugs / T-PerMethodMetadata directly; no Structural Close Manager).

**v2 retirement (explicit non-scoping note):** The v2 compiler at `src/v2/` persists as test oracle into R2 and is NOT on the R2 ledger. Its retirement is external post-R2 operational cleanup — no thesis claim depends on v2 being absent. Differential-test-oracle retirement is bounded by adoption/documentation concerns, not release-gate discipline.

## Demo discipline — visibility as structural requirement

Simple "look, it runs" or "before/after analysis" artifact ships with each lane closure PR. Director coordinates surfacing to user. No time-based cadence; the gate-close natural rhythm carries the visibility load directly — a demo lands whenever a lane closes, not on a schedule.

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
- **Manager count = 1 + Director.** One standing manager (Grounding) matches R2's single critical path. T-Modeling / T-Substrate / T-ImpossibleBugs are parallel-capable with no critical-path coordination pressure; Director dispatches directly. Adjustable upward if parallel fill-queue depth becomes unmanageable in practice.
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

### 2. Pre-promotion `≤5 irreducible-shim` gate-name review (gate before ROADMAP promotion)

Surfaced by user question on 2026-04-24: the "≤5 irreducible-shim" floor in `docs/design-pure-bootstrap.md` is framed as a principled target but is actually a generous ceiling. The doc itself names the narrower case: *"If (4) and (5) can be generated, the shim is 3 files"* (§"Irreducible shim (target state)") — and v2 achieves the floor with 2 files (CLI + interpreter). The principled boundary floor is 2–3, not 5.

"≤5" reads as a principled absolute in gate names (e.g., `pb_hand_rust_at_shim_floor`) when it's really an upper bound carried for continuity with the 2024 design doc. Leaving the number unchallenged risks the "close-everything" claim of R2 resting on a shim-floor number that's looser than the principled floor.

**Required before promotion:** one of —
- **Option A (sharpen-and-keep)** — edit `design-pure-bootstrap.md` to explicitly name the trajectory (`baseline → ~20 → 5 → 3 → 2`); keep `≤5` as the gate name but document that the principled boundary-determined floor is 2–3; promote R2 structure as-is.
- **Option B (rename)** — rename the gate to `pb_hand_rust_at_boundary_floor` (threshold = boundary-determined, not a fixed number); land the doc edit + gate rename together; promote R2 structure after rename lands.

Authority: `docs/design-pure-bootstrap.md` §"Irreducible shim (target state)"; v2 empirical evidence (CLI + interpreter = 2 files per the same doc's "Inspiration: v2's model" section).

Non-blocking for this PR. Not done in this PR because the gate rename is a cross-doc edit with R1-lane-acceptance implications (T-PB-A's gate name would change); both land as prerequisites to the ROADMAP promotion.

## Cross-refs

- Parent: `ROADMAP.md` (sections: `## Release R1 Program`; `## Post-R1 Program — Grounding Completeness`; `## Tracked debts — 2026-04 analyses`).
- Substrate design: `docs/design-substrate-carrier-port-program.md` (E-family lanes + §6a per-method-metadata).
- Self-hosting anchor: `docs/design-pure-bootstrap.md` (≤5 shim floor + SG census).
- Thesis: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags authority); `THESIS.md §"Thesis claims — complete list"` (Tier-1 claim lineage).
- Lens capability: `docs/v3-lens-capability-register.md` (per-lens capability tracking).
- DB history: `docs/db-history/db-18.md` (DB-18 Part-2 shipped: workflow-effect carrier + Rust reflection; Part-3 queued: Go accessor). Note: `ROADMAP.md §"Post-R1 Program — Grounding Completeness"` tags "DB-18 parametric algebra attachment" as a post-R1 blocker; that label is not obviously aligned with db-history's DB-18 scope — a pre-promotion rename or new DB number may be warranted for the R2 parametric-algebra prereq.
- Related PRs: #745 (P4 int-literal row — substrate motivation for T-Modeling), #752 (T-PB-A lens-producer priority slice — R1 gate, not R2).
