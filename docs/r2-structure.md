# R2 Structure

**Status:** `PROPOSAL` — pending user sign-off + R1 closure + promotion to `ROADMAP.md` as `## Release R2 Program` section.

**Authority:** single-source while open. Amendments before promotion land in this doc. After promotion, amendments follow the same discipline as R1's `## Release R1 Program` section (director-authored PRs with manager acknowledgement).

**Scope naming note:** `docs/db-history/db-18.md` uses "R2 carrier" as internal DB-stage nomenclature that predates release-level R# naming. Our release-level R2 (this doc) is unrelated to DB-18's stage label; no collision of meaning, just of string.

## Summary

R2 is the **close-everything** release. Every remaining Tier-1 thesis claim not closed by R1 lands here. The two co-anchor claims — **Grounding Completeness** and **Lens Purity by Construction** — are joined by modeling-faithfulness dissolution, substrate prereqs, remaining impossible-bug classes, self-hosting shim-floor close, and tests-as-data closure.

Two framing decisions drive scope + coordination:

1. **Anti-deferral principle.** If dissolution direction is clear and named, deferral is problem-finding, not problem-solving. R2 absorbs what has named dissolution directions, regardless of current execution velocity. (Velocity is a trailing observation; it can accelerate or slow between waves. The principle is what's load-bearing.)
2. **Light-touch throughput-oriented coordination.** Manager count = concurrent critical paths, not total scope.

## Program count — 2 active releases total

- **R1** closing.
- **R2** = close-everything.
- **R3** reserved as *escape hatch only*, for items that genuinely cannot close in R2 despite honest effort. Invocation should be rare and itself signal a problem worth examining — if dissolution is surfacing faster than closure, that's a leading indicator to address, not a scope-inflation signal.

Post-R2 is external work (adoption, documentation, community) — not on the thesis-claim release ledger.

## Goals

1. **Grounding Completeness** — target-side primitive types for Rust/Python/Go structurally declared; coercion via inhabitance search; Track-13 dissolution. Inherits from `ROADMAP.md:149` "Post-R1 Program — Grounding Completeness" → promotes to R2 lane `T-Ground`.

2. **Lens Purity by Construction** — every lens body `.dag`-authored; kernel closure replaces reviewer-convention. `lens_producer_files_remaining` gate (introduced via PR #752) lands at zero.

3. **Self-hosting shim-floor close** — T-PB-A non-test census reaches ≤5 irreducible shims per `docs/design-pure-bootstrap.md`; T-PB-B outside-residual-zero (per `TESTING.md §Post-R2 shape`); compiler-std consolidation ratchet → 0.

4. **Modeling-faithfulness dissolution** — three Tier-1 type-refinement gaps close:
   - Surface int-literal magnitude at concept layer (P4 row on `ROADMAP.md`; originating analysis on PR #745)
   - `Secret<T>` nominal-opaque graduation (`ROADMAP.md` post-merge-debt section, 2026-04-23 thesis-doc surface)
   - `Dimension<Carrier>` typed value wrapper with phantom-parameter unit-mismatch enforcement (ibid.)

5. **Substrate prereqs** — named as explicit R2 sub-lanes with **scoped acceptance criteria** (sufficient-to-unblock, not full-capability). Each prereq is pinned to a specific Goal 4 item; full substrate-capability lanes retain open design calls that may predate or postdate R2, and this structure does not commit R2 to close them all:

   - **Cardinality-substrate subset sufficient to close int-literal magnitude refinement** — enough cardinality modeling to let `IntLit` carry a magnitude that narrows to target int algebra at reconciliation. Does NOT commit to the full cardinality-substrate capability (fixed-width-types by-construction, container cardinality bounds in Grounding, etc. — those remain open design calls outside R2 scope unless additional R2 items demand them).
   - **Nominal-opaque substrate sufficient to graduate `Secret<T>`** — enough nominal-type modeling to carry construction-restriction (`where only X may construct`) semantics. Adjacent to DB-11 alias-RHS `where` (landed in R1 via PR #703); may or may not overlap DB-18 territory. Acceptance is `Secret<T>` graduation, not a general nominal-type program.
   - **Parametric algebra attachment subset sufficient to inhabit `Dimension<Carrier>` in an abelian group algebra** — enough substrate capability to let `Dimension<Unit>` carry phantom-parameter arithmetic (propagate through operations, compile error on unit-mismatch). Primary authority is `ROADMAP.md:155` which tags this dependency `DB-18 parametric algebra attachment` — but `docs/db-history/db-18.md` currently scopes DB-18 to workflow-effect carrier + Rust reflection (Part 2 shipped) + Go-accessor follow-up (Part 3), not parametric algebra attachment. That mismatch is an existing ROADMAP ↔ db-history inconsistency, not one introduced by this doc; a pre-promotion DB-lane rename or new DB number may be warranted. R2 acceptance is: `Dimension<Unit>` phantom-parameter arithmetic compiles with unit-mismatch errors, independent of the DB-tag the substrate ends up carrying.

6. **Remaining R2+ impossible-bug classes** — three classes currently tagged `[R2+]` at `ROADMAP.md:72` (THESIS §"Enumerable impossible-bug classes" is the authority on scheduling tags):
   - Nested-optional flatten
   - Unhandled diagnostic paths
   - Unenumerated effects

7. **E-family carrier port closure** — E-I finish → E-P → E-M → §6a per-method-metadata call, per `docs/design-substrate-carrier-port-program.md`. Per-method-metadata option-pick deferred in R1; decides here.

8. **R2 closure demo** — simple "it runs" artifact per lane close. Director-coordinated. No dedicated demo lane (see Demo discipline below).

## Manager structure

**2 standing managers + Director.** Count = concurrent critical paths (one per manager).

### Grounding Manager

Continues `docs/briefs/grounding-manager.md` (refreshed for R2 scope on promotion). Owns T-Ground sub-program.

- **Critical path:** T-Ground-Pilot → T-Ground-Rust → T-Ground-Engine → T-Ground-Tests → T-Ground-Dissolve (per `ROADMAP.md:169` — Rust is on the critical path because Engine blocks on layers 1–3 populated and Rust is the first layer-populating target).
- **Fill queue:** T-Ground-Python, T-Ground-Go (2-way parallel after Pilot validates; run alongside Rust but are not gated by Engine-blocking).

### Structural Close Manager

New brief at `docs/briefs/r2-structural-close-manager.md` (to author on promotion). Consolidates former R1 Self-hosting + Substrate + Testgen-tail authorities. Naming rationale: scope covers E-family carrier port + substrate prereqs + modeling-faithfulness + shim floor + lens migration + testgen predicate wiring + impossible-bug class closure — "self-hosting" is accurate for only a subset, so "Structural Close" names the actual scope honestly.

- **Critical path:** T-EFamilyClose (E-I finish → E-P → E-M) + §6a metadata pick.
- **Fill queues** (all file-level or item-level parallel; any available worker picks top-priority unblocked):
  - T-LensMigration (per-file parallel)
  - T-ShimFloor (per-file parallel)
  - T-Modeling (int-lit / Secret<T> / Dimensions — 3-way parallel; each blocks on its scoped-subset prereq in T-Substrate)
  - T-Substrate (3 scoped-subset sub-lanes per Goal 5: cardinality-subset-for-int-lit; nominal-opaque-for-Secret; parametric-algebra-attachment-for-Dimensions — 3-way parallel; each sub-lane's close criterion is its paired T-Modeling unblock)
  - T-ImpossibleBugs (3 remaining classes — sparse; fills when other fill queues are saturated)

### Director (ad-hoc)

- R1 residual closure surveillance (none expected per all-gates-green closure criterion).
- R2 demo coordination: surfaces "it runs" artifacts at each lane close to user.
- Cross-manager dependency surfacing when critical paths block.
- Weekly dependency health check: which lanes are within 1 step of unblocking? Which workers are on fill vs. ready? Are bottlenecks compounding?

## Lane structure

| Lane | Size | Manager | Covers |
|---|---|---|---|
| T-Ground | XL | Grounding | Full T-Ground-* sub-program (Goal 1) |
| T-LensMigration | L | Structural Close | Every lens producer `.rs` → `.dag` (Goal 2) |
| T-EFamilyClose | M | Structural Close | E-I finish + E-P + E-M + §6a (Goal 7) |
| T-ShimFloor | M | Structural Close | T-PB-A non-lens reductions; T-PB-B outside-residual-zero (Goal 3) |
| T-Modeling | M | Structural Close | int-lit / Secret<T> / Dimensions (Goal 4) |
| T-Substrate | M | Structural Close | Three scoped-subset sub-lanes (Goal 5): cardinality-for-int-lit; nominal-opaque-for-Secret; parametric-algebra-attachment-for-Dimensions — each scoped to its paired T-Modeling unblock, not full substrate-capability |
| T-ImpossibleBugs | S | Structural Close | nested-optional flatten / unhandled-diagnostic-paths / unenumerated-effects (Goal 6) |

**Goal 8 (R2 closure demo) is not a lane.** It is a cross-lane closure discipline (see "Demo discipline" below): each lane's closure PR ships its own simple "it runs" artifact; Director coordinates surfacing. No separate T-Demo lane owner, no separate demo-authoring critical path.

## Dependency DAG

```
T-Ground:         Pilot → Rust → Engine → Tests → Dissolve   (critical path)
                  Python, Go run parallel after Pilot (fill queue; not Engine-blocking)
T-EFamilyClose:   E-I (in flight) → E-P → E-M → §6a pick
T-Substrate:      cardinality-for-int-lit (subset) ──→ unblocks T-Modeling int-lit
                  nominal-opaque-for-Secret (subset) ─→ unblocks T-Modeling Secret<T>
                  parametric-algebra-for-Dimensions (subset) ─→ unblocks T-Modeling Dimensions
T-Modeling:       int-lit      ← T-Substrate cardinality-for-int-lit
                  Secret<T>    ← T-Substrate nominal-opaque-for-Secret
                  Dimensions   ← T-Substrate parametric-algebra-for-Dimensions
T-LensMigration:  per-file independent (any worker)
T-ShimFloor:      per-file independent (any worker)
T-ImpossibleBugs: 3 independent classes (any worker)
(Goal 8 demo artifacts ship with each lane's closure PR — not a
 separate dependency-DAG node; see Demo discipline section.)
```

Parallel-capable work at any time ≥ N workers × fill-queue depth (5 fill queues on Structural Close Manager + 2 on Grounding).

## R1 closure criteria

**All R1 gates green.** R1 closes when all 9 lane gates named at `ROADMAP.md:61-73` evaluate green, including omni-emit (`emit_omni_demo_fixtures_green`). No director-defined subset-close. Rationale: consistent with anti-deferral stance — tail-shaped work closes before R1 declares done; R2 doesn't inherit R1 residuals.

## Transition mechanics

1. **R1 gates green** → Director declares R1 closed.
2. **R1 residual sweep** — every open R1 ledger row gets an R1-or-R2 assignment. No orphaning. Done in the R1 closure PR. Expected to be short under all-gates-green criterion.
3. **Manager dissolution** — R1 Surface Manager archives (closure banner); R1 Testgen Manager archives (scope folds into Structural Close); R1 Substrate Manager archives (E-family folds into Structural Close); R1 Self-hosting Manager expands and renames to Structural Close Manager.
4. **R2 open** — this doc promotes to `ROADMAP.md` as `## Release R2 Program` section. `docs/briefs/r2-structural-close-manager.md` authored. `docs/briefs/grounding-manager.md` refreshed for R2 scope.

## Demo discipline — visibility as structural requirement

Simple "look, it runs" or "before/after analysis" artifact ships with each lane closure PR. Director coordinates surfacing to user. No time-based cadence; the gate-close natural rhythm carries the visibility load directly — a demo lands whenever a lane closes, not on a schedule.

Forms that qualify:
- Running artifact + 1-paragraph "what this demonstrates"
- Before/after: "this program didn't compile; now it does"
- Census snapshot: "retired N hand-Rust files this milestone"
- Diagnostic demonstration: "here's a bad program, here's the error, here's the fix suggestion"

Purpose: proof-of-work visibility at director cadence. Without it, program slips invisibly over long horizons.

## Decisions locked

- **Goal 4 in R2** (not R3+). Anti-deferral principle: dissolution directions are named and clear for all three items (int-lit via concept-layer magnitude decoupling; `Secret<T>` via nominal-opaque; Dimensions via phantom-parameter algebra attachment), so deferral would be scope-theater. Director's initial "defer to R3+" counter reviewed and conceded post-reframe.
- **R1 closure criteria = all-gates-green**. Same anti-deferral principle applied to omni-emit.
- **Demo cadence = gate-close natural rhythm**. Simple artifact per close; no time-based schedule.
- **Manager count = 2 + Director**. Adjustable to 3 mid-R2 if Structural Close fill-queue depth becomes unmanageable (naming for a hypothetical third: "Modeling Manager" — would take T-Modeling + T-Substrate + T-ImpossibleBugs).
- **R2 includes substrate prereqs explicitly** per user's (i)-over-(ii) preference (honest scope over tight scope), with **scoped acceptance criteria** per Director refinement (each sub-lane closes on unblock of its paired Goal 4 item; full substrate-capability lanes are not R2-committed).
- **Anti-deferral principle is the frame, not velocity numbers.** Per Director observation: 16-hour R1 execution was a peak-day sample, not a baseline. The principle "if dissolution direction is clear and named, deferral is problem-finding not problem-solving" is what survives cadence shifts.

## Open calls

### 1. Post-R2 stance — strong vs weak endorsement (user decision)

The proposal currently reads as *strong*: "R2 = thesis close; post-R2 is external (adoption/docs/community); R3 reserved as escape hatch only." That's consistent with anti-deferral + close-everything-knowable. But it commits us to: no future thesis-claim release after R2.

Two readings for user to explicitly pick:

- **Strong endorsement** — yes, R2 = thesis close. R3 reserved only for escape hatch. Post-R2 work is all external. Commit; this doc's framing is final.
- **Weaker endorsement** — R2 is the current best scope; R3-as-structure remains available if R2 work surfaces genuinely new thesis-claim architecture that can't close in R2. Soften the "post-R2 = external" framing to preserve option value.

Director leans strong if user trusts the anti-deferral principle; leans weaker if user wants option value in case post-R2 evidence surfaces something load-bearing.

**Pending user decision.** Other R2 structure is locked; this is the one remaining endorsement before the doc promotes to ROADMAP.

### 2. Pre-promotion thesis-claim coverage mapping (gate before ROADMAP promotion)

Surfaced by codex API review on `6fdd8341`: the "close-everything/post-R2-external-only" framing requires an explicit mapping from THESIS tiers to concrete R1/R2/post-R2 disposition, so no thesis claim is implicitly-positioned. Otherwise "close-everything" is an assertion without audit.

THESIS authority (`THESIS.md:155-182`) lists:
- **Tier 1 — Structural correctness** (type mismatches, CX termination, coercion = emission, ownership no-alias, **Grounding completeness**).
- **Tier 2 — Runtime safety** (division-by-zero, integer overflow, out-of-bounds, force-unwrap, partial functions — proven safe or made total).
- **Tier 3 — Verification from structure** (L4 emitted ≡ .dag, L5 cross-target parity, L6 structural-form coverage, L7 algebraic laws).

**Required before promotion:** a table mapping every Tier-1 / Tier-2 / Tier-3 claim to its R1-closed / R2-gated / post-R2-external disposition, with any gaps (claim named in THESIS but not mapped) flagged as pre-promotion blockers. Non-blocking for this PR; blocking for ROADMAP promotion.

**Not done in this PR** because the PR is scope-setting and the coverage audit is a sibling deliverable; both land as prerequisites to the `## Release R2 Program` ROADMAP section.

## Cross-refs

- Parent: `ROADMAP.md` (R1 program at `:15`; Grounding Completeness post-R1 at `:149`; Tracked-debts ledger at `:291`).
- Substrate design: `docs/design-substrate-carrier-port-program.md` (E-family lanes + §6a per-method-metadata).
- Self-hosting anchor: `docs/design-pure-bootstrap.md` (≤5 shim floor + SG census).
- Thesis: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags authority); `THESIS.md §"Thesis claims — complete list"` (Tier-1 claim lineage).
- Lens capability: `docs/v3-lens-capability-register.md` (per-lens capability tracking).
- DB history: `docs/db-history/db-18.md` (DB-18 Part-2 shipped: workflow-effect carrier + Rust reflection; Part-3 queued: Go accessor). Note: `ROADMAP.md:155` tags "DB-18 parametric algebra attachment" as a post-R1 blocker; that label is not obviously aligned with db-history's DB-18 scope — a pre-promotion rename or new DB number may be warranted for the R2 parametric-algebra prereq.
- Related PRs: #745 (P4 int-literal row — substrate motivation for T-Modeling), #752 (T-PB-A lens-producer priority slice — substrate motivation for T-LensMigration gate).
