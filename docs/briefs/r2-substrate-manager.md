# R2 Substrate Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827; refreshed 2026-04-28 post-#1078 merge to absorb 8-question design locks + Lens<C> primitive sub-lane + post-R2 continuation). Eligible to spawn pre-R1-close per `r2-structure.md` Transition mechanics step 4 — Substrate's ValueBody-list/sum sub-lane is a prereq for R1C-A Sub-deliverable A, so pre-R1-close spawn actively unblocks R1 closure rather than competing with it. NEW manager — no migration source.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers (count rose from 6 to 7 with R2-Evaluator added 2026-04-28 per #1078). Substrate is **co-XL with Evaluator** — together the largest single concentrations of R2 work.
- **Program scope sources:**
  - T-Substrate (R2 prereq sub-lanes): [`docs/r2-structure.md` §"Goals" item 3](../r2-structure.md).
  - T-Substrate-Lens-Primitive (added 2026-04-28 via #1078): [`docs/r2-structure.md` §"Goals" item 3 (Substrate prereqs) — 5th sub-bullet "T-Substrate-Lens-Primitive"](../r2-structure.md) + [`docs/design-lens-framework.md`](../design-lens-framework.md). Anchor in the Substrate Manager section at `r2-structure.md:96`.
  - B4 Identity-Carrier Substrate Pass program: [`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md).
  - **Post-R2 continuation:** T-CostLens-Composition (R3 lane under Substrate continuation per Director cascade Item 3 ratified 2026-04-28). Substrate Manager is one of **two** R2 managers that continue operating into R3 — the other being Pure Bootstrap (which carries T-LensProducer-Retirement + T-FixedPoint + T-Tier3-Dissolution + 3 distributed bridges per Director cascade Item 4). Verification is a new R3 manager (not an R2 continuation). Modeling and Impossible-Bugs Managers archive at R2 close.
- **Cross-program producer/readiness owner:** T-Substrate sub-lanes either produce carriers or validate existing substrate readiness for Modeling Manager (3 sub-lanes), Grounding Manager (Coercion-Fold consumes ValueBody-list/sum), and Evaluator Manager (Lens<C> primitive). The Dimensions sub-lane is already substrate-ready by audit, so it is a readiness signal rather than new carrier work.
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): the 3-step decision procedure (DAG-ancestor check → coproduct-vs-coordinate check → primitive-vs-lens-extensible check) is **mandatory** for every new substrate type/variant/field this manager introduces. Self-serve through the procedure before escalating substrate-shape questions to Director. `feedback_substrate_principle_audit.md` extends this with a 6-question structural-recovery audit; both apply.
- **Watch condition:** if Substrate becomes the new bottleneck (workers idle >7 days waiting for Substrate-authored briefs), split B4 into a dedicated standing **B4 Identity-Carrier Manager** per `docs/r2-structure.md:88` watch trigger. R2 Release Manager surfaces this signal via velocity-tripwire reporting.

## Program scope

**Three coupled sub-programs in R2 + one continuation into R3:**

### A. T-Substrate (4 sub-lanes — Goal 3 substrate prereqs)

Each sub-lane is **scoped to its paired R2 consumer set**, not full substrate-capability close.

1. **Cardinality-substrate subset for int-literal magnitude** — enough cardinality modeling to let `IntLit` carry magnitude that narrows to target int algebra at reconciliation. Consumer: Modeling Manager's int-lit item.
2. **Nominal-opaque substrate subset for `Secret<T>`** — enough nominal-type modeling to carry construction-restriction (`where only X may construct`). Consumer: Modeling Manager's `Secret<T>` item.
3. **Parametric-algebra-attachment subset for `Dimension<Carrier>`** — enough substrate to inhabit `Dimension<Unit>` in an abelian group algebra (compile error on unit-mismatch). Consumer: Modeling Manager's Dimensions item.
4. **Top-level `ValueBody` list/sum + `std.unicode` bootstrap subset** — enough Class 5 Gap 3 substrate for `data ascii_scan_order: List<CharClass> = [...]` to lower structurally. **Two named consumers**: Modeling Manager's tokenizer charclass phase-2 + Grounding Manager's Coercion-Fold sub-lane (full pilot enumeration via symbolic walk of `rust_pilot_primitives: List<RustPrimitive>`).

### B. T-Substrate-Lens-Primitive (added 2026-04-28 via #1078)

Substrate primitive landing for `Lens<C>` per the 8-question dialogue (Q6/Q7/Q8 locked) + the design-lens-framework spec.

**6-field structural shape** (per Q6/Q7/Q8 locks; cascaded fix from quiet-koi-451's review):
```
Lens<C> = (
  name: String,
  read: (Dag, Behavior) → Witness<C>,        // Q6: Witness<C> stays as-is; structural failures use lens-owned Diagnostic.kind declarations
  sequential: Monoid<C>,                       // structural inhabitance; replaces parallel compose+unit cascade
  branch: (C, C) → C,
  iterate: (LoopBound, C) → C,
  validate: (Dag, C) → OptionalDiagnostic     // Q7: per-call validate; fold accumulates into DimensionFail.violations
)
```

Consumers:
- Evaluator Manager — implements `fold_lens<C>: Lens<C> → Dag → DimensionReport<C>`.
- Substrate Manager (R3 continuation) — T-CostLens-Composition consumes Lens<C> for cost-algebra witnesses.
- Q8 lock: cross-product `Lens<C> × Lens<D>` is conjunctive (both validate runs; conjunctive fold).

### C. B4 Identity-Carrier Substrate Pass program (12 sub-briefs)

From [`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md). Treats the §0 class from `docs/briefs/debt-paydown-synthesis-2026-04-25.md` as one M-scope substrate program, not 8 item-by-item paydowns.

**Phase 1 carriers (parallel after audits):**
- B4.1 `DeclarationRef` consumer migration (existing carrier at `src/v3/spec/v3_l1.dag:69`; consumer migration only, no landing)
- B4.2 fold-shape carrier (NEW)
- B4.3 emit-helper carrier (NEW)
- B4.4 extdeps-fixture-set carrier (NEW)

**Phase 2 site dissolutions (mechanical; dispatched as Phase 1 carriers land):**
- B4.5–B4.12: 8 sites (consumer migration of new substrate)

**Phase 3:** discipline ratchet (one-time, after Phase 1) — reviewer-discipline addition for new sentinel-string sites.

### D. Post-R2 continuation: T-CostLens-Composition (R3 lane)

**Per Director cascade Item 3 ratified 2026-04-28** ([`docs/r3-structure.md`](../r3-structure.md) §"Lane structure"). Substrate continues into R3 to complete the cost-algebra composition lens — the only Substrate work that survives R2 close.

Why Substrate-owned (not Verification-owned): T-CostLens-Composition is a **structural Lens<C> instance** producing `RealizationCost { storage: Cost<Bits>, access: Map<AlgebraOp, Cost<CPUCycles>> }` witnesses (per Q3 lock). The lens framework's structural inhabitance IS the namespacing mechanism; lens-instance authoring is substrate-completion work, not verification work. Verification Manager (R3) asserts the gate fires by construction; doesn't author the lens.

Depends on R2-T-Substrate-Lens-Primitive landing (the framework primitive that T-CostLens-Composition is a lens-instance of).

## Pre-dispatch design lock cadence (consumed; per #1078 lock)

The 8-question design dialogue resolved into a cadence of design PRs whose merge gates worker dispatch on this manager's lanes. **Substrate Manager owns PR-PreF + PR-K**; Director authors them inline in the design docs OR as separate PRs.

| Cadence PR | Locks | Consumer |
|---|---|---|
| **PR-PreF** | `Interval<D>` substrate consolidation — shared parent for `CardinalityBound` / `SizeBound` / `LoopBound::Cardinality` (additive retrofit; `LoopBound::Descent` and `CostBound` stay distinct). Sets up Q1's `BoundDeclaration = Interval<Int>` instance to consume a clean parent rather than introduce a fifth distinct bound type. | All subsequent cadence PRs (PR-F through PR-K), plus T-Substrate cardinality-for-int-lit and T-Modeling Q1-consuming items |
| **PR-I** | Q3 (per-primitive realization cost) — `Cost<Unit> = Dimension<Unit, SymbolicExpr>`; Bits/CPUCycles primitives sibling to SI base units; `RealizationCost { storage: Cost<Bits>, access: Map<AlgebraOp, Cost<CPUCycles>> }` per-primitive field on LanguageSpec | T-Ground-LanguageSpec + T-Verification-L4-L7-Direct |
| **PR-K** | Q6 + Q7 + Q8 (lens framework spec — structural inhabitance shape, validate signature, conjunctive cross-product) | T-Substrate-Lens-Primitive |

Dispatch sequence: **PR-PreF lands first** (foundational substrate; everything else depends on it). PR-I and PR-K are independent of each other once PR-PreF lands; both can land in parallel. Worker dispatch on T-Substrate-Lens-Primitive blocks on PR-K merge; cardinality-for-int-lit blocks on PR-PreF merge.

## Owned deliverables (through R2 close + R3 T-CostLens-Composition continuation)

| Sub-lane | Size | Current status | Carrier shape |
|---|---|---|---|
| T-Substrate cardinality-for-int-lit | M | BRIEF LANDED (`t-substrate-cardinality-int-lit-worker.md`, PR #806 merged 2026-04-25). **Now consumes PR-PreF Interval<D>** when it lands. | range facts + reconciliation narrowing; Int128/Word128 carrier deferred to sibling sub-lane |
| T-Substrate nominal-opaque-for-Secret | M | BRIEF AUTHORED (PR #836); **fail-closed field-projection enforcement LANDED via #937** (NominalOpacityViolation diagnostic + production enforcement before nominal-opaque field descent; complements #900 carrier-only staging). Secret<T> consumer migration still pending per `r2-modeling-secret-graduation-worker.md`. | nominal-type construction/access restriction |
| T-Substrate parametric-algebra-for-Dimensions | M | CLOSED BY AUDIT (PR #836); substrate already exists, consumer dispatchable. **T-Cost-Dimension fail-closed symbolic-cost analysis LANDED via #1003** (DominateScanAcc conjunctive accumulator; relevant to Dimensions consumer modeling). | existing `Declaration.phantom_params` + `phantom_unit_mismatch` carrier |
| T-Substrate ValueBody-list/sum + std.unicode | L | BRIEF LANDED (PR #790 merged 2026-04-25) | top-level list/sum literal lowering + bootstrap/load-set |
| **T-Substrate ValueBody-Map** *(NEW; R2 unblocker)* | M | **SUBSTRATE LANDED via #1017** (string-keyed `ValueBody::Map` + nested `FieldValue::Map` carriers; structural lowering of `Map<String, _>` literals). **Tightening landed via #1068** (`FieldMap` newtype with private storage + duplicate-key validation at construction). Consumer plumbing (read-path/API + arrow-body evaluation) pending — unblocks PB Manager `kernel_algebra_profile` mirror dissolution. | top-level map literal substrate |
| **T-Substrate-Lens-Primitive** *(NEW 2026-04-28)* | M | NOT YET AUTHORED — gated on PR-K lock (Q6+Q7+Q8) | `Lens<C>` 6-field record in `src/v3/std/dimensions.dag` (sibling to existing `AnalysisDimension<Carrier>` + `Dimension<Unit, Carrier>`); Witness<C>+OptionalDiagnostic+DimensionReport<C> already in substrate |
| **PR-PreF Interval<D> consolidation** *(NEW 2026-04-28)* | S | NOT YET AUTHORED — Director-authored or Substrate-Manager-authored | shared parent type for CardinalityBound / SizeBound / LoopBound::Cardinality |
| B4.1 DeclarationRef consumer migration | M | LANDED (PR #826 merged 2026-04-26) | existing carrier consumer migration |
| B4.2 fold-shape carrier | S | BRIEF AUTHORED (PR #836); **first-consumer wiring LANDED** (`feat(v3): add B4.2 structural fold eligibility`) | structural fold-eligibility query/carrier decision |
| B4.3 emit-helper carrier | S | LANDED (PR #824 merged 2026-04-26) | typed role marker on Bind/Branch nodes |
| B4.4 extdeps-fixture-set carrier | S | LANDED (PR #825 merged 2026-04-26) | typed extdeps-bootstrap-set declaration |
| B4.5–B4.12 Phase 2 site dissolutions | S each | QUEUE AUTHORED (`b4-phase-2-site-dissolution-queue.md`); **B4.8 LANDED via #1069** (emit-helper file-marker bind selection in roundtrip fixture; first Phase-2 site dissolved); B4.5/B4.6/B4.7/B4.9–B4.12 still queued as Phase 1 carriers land. | mechanical consumer migration per site |
| **R3 continuation: T-CostLens-Composition** *(NEW 2026-04-28; R3 lane)* | M | NOT YET AUTHORED — gated on T-Substrate-Lens-Primitive landing + R2 close | structural Lens<RealizationCost> instance; Monoid<SymbolicCost> + JoinSemilattice + BoundedLattice<BigOClass> witnesses |

## Cross-program dependencies

**Produces (6 carrier-readiness signals):**
- Cardinality-for-int-lit → Modeling Manager (int-lit; via PR-PreF Interval<D>)
- Nominal-opaque-for-Secret → Modeling Manager (Secret<T>)
- Parametric-algebra-for-Dimensions → Modeling Manager (Dimensions; via existing `phantom_params`)
- ValueBody-list/sum + std.unicode → Modeling Manager (charclass phase-2) + Grounding Manager (Coercion-Fold sub-lane)
- **ValueBody::Map carrier read-path/API + arrow-body evaluation** → Pure Bootstrap Manager (`kernel_algebra_profile` mirror dissolution). Substrate landed via #1017 + tightened #1068; consumer plumbing is the remaining produced signal.
- **Lens<C> primitive** → Evaluator Manager (implements `fold_lens<C>`); Substrate Manager R3 continuation (T-CostLens-Composition consumes)

**Consumes:** none in R2 (Substrate is the substrate). R3 continuation consumes its own R2 output (Lens<C> primitive) plus Evaluator's `fold_lens<C>` runtime.

**Adjacent territory:**
- B4's §0.7 file-preference rank carrier touches Pure Bootstrap territory. Coordinate with Pure Bootstrap Manager.
- **Diagnostic-kind extensibility (Q6 lock)** — structural-validation failures use lens-owned, namespaced `Diagnostic.kind` declarations per `docs/design-lens-framework.md` §"Q6.5 — Two-layer authority for diagnostic kinds (cross-manager protocol)". Substrate Manager owns the substrate mechanics for reflecting/resolving those namespaced kinds under T-Substrate-Lens-Primitive; Evaluator Manager owns carrying produced kinds through `DimensionFail.violations`. The current bootstrap `CompilerDiagnosticKind` sum remains compiler-native vocabulary, not the per-lens extension point.

## Locked design decisions consumed (per #1078 8-question dialogue)

Worker briefs MUST consume these without re-litigation:

- **Q1**: `Interval<D>` shared parent in substrate (PR-PreF prepended); `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent` (asymmetric match rule: target's `Unbounded` universal-accepts; target's `ExactInterval(lo,hi)` requires exact range match).
- **Q3**: `Cost<Unit> = Dimension<Unit, SymbolicExpr>`; Bits/CPUCycles substrate primitives sibling to SI base units; `RealizationCost { storage: Cost<Bits>, access: Map<AlgebraOp, Cost<CPUCycles>> }`.
- **Q5**: cardinality is the connectives axis; collapses by construction with Q1's `Interval<Cardinal>` instances on each connective; PR-J = no-op.
- **Q6**: `Witness<C>` substrate stays as-is; rich structural validation failures encode into `Diagnostic.kind` extensions.
- **Q7**: per-call validate yields one `OptionalDiagnostic`; fold accumulates into `DimensionFail.violations: List<Diagnostic>`.
- **Q8**: cross-product validate is conjunctive (`Lens<C> × Lens<D>` runs both; conjunctive fold).

Full disposition: [`docs/r2-structure.md`](../r2-structure.md) §4 + [`docs/design-emission-model.md`](../design-emission-model.md) Q1-Q5 + [`docs/design-lens-framework.md`](../design-lens-framework.md) Q6-Q8.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (post-#1078-merge):** brief authoring + the cadence sequence (PR-PreF, PR-I, PR-K) locked. Substrate Manager spawns once a design-lock PR is dispatchable; pre-R1-close spawn allowed and actively unblocks R1C-A Sub-deliverable A.
- **Post-spawn (manager active):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Substrate + T-Substrate-Lens-Primitive + B4 sub-briefs without Director.
- Dispatches workers against all sub-briefs.
- Resolves Substrate-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every Substrate worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; every PR introducing hand-Rust under `src/v3/` fills the per-PR gate.
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1) applies to every brief introducing new substrate types/variants/fields; document the procedure outcome in the brief.
- **Cross-program signal authority:** carrier-readiness signals → cross-manager queue; per-sub-lane closure → R2 Release Manager (closure ledger); Lens<C> primitive readiness → Evaluator Manager directly.

## Reporting cadence

- Sub-lane / Phase close → R2 Release Manager (closure ledger). Each sub-lane's structural acceptance gate (per the **structural-acceptance-per-lane-close discipline** in `r2-structure.md`) IS the demo.
- Cross-program signals (6 carrier-readiness for 6 carriers) → cross-manager queue (consuming managers ack and dispatch consumer-migration work).
- Blockers + scope changes → Director.
- **Weekly health surfacing to Director:** which sub-lanes within 1 step of unblocking, which workers fill vs. ready, which cadence PRs (PR-PreF/PR-I/PR-K) landed vs pending, R3 continuation readiness signal.

## Acceptance — `.dag` gates

Each sub-lane closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

- `interval_d_shared_parent_consolidation_landed` — PR-PreF Interval<D> consolidation in substrate; CardinalityBound/SizeBound/LoopBound::Cardinality consume the parent
- `cardinality_int_lit_substrate_landed` — Cardinality substrate subset reconciles IntLit magnitude to target int algebra
- `nominal_opaque_for_secret_substrate_landed` — `Secret<T>` construction-restriction enforced structurally
- `parametric_algebra_for_dimensions_audit_closed` — `phantom_params` + `phantom_unit_mismatch` carry compile-error path
- `valuebody_list_sum_unicode_bootstrap_landed` — `data ascii_scan_order: List<CharClass>` lowers structurally
- `lens_c_primitive_substrate_landed` — `Lens<C>` 6-field record in substrate; consumed by Evaluator's `fold_lens<C>`
- `b4_phase_1_carriers_landed` (B4.1/B4.2/B4.3/B4.4) and `b4_phase_2_dissolutions_landed` (B4.5–B4.12)
- **R3 continuation:** `cost_lens_composition_witnesses_correct` — Monoid<SymbolicCost> + JoinSemilattice + BoundedLattice<BigOClass> structural inhabitance demonstrated on representative cost expressions

## Sub-briefs (authored / pending)

Authored:
- B4 program brief ([`b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md))
- B4.1 DeclarationRef consumer migration ([`b4-1-declarationref-consumer-migration-worker.md`](b4-1-declarationref-consumer-migration-worker.md)) — landed PR #826
- B4.2 structural fold-shape (`b4-2-structural-fold-shape-carrier-worker.md`) — authored PR #836
- B4.3 structural emit-helper carrier (`b4-3-structural-emit-helper-carrier-worker.md`) — landed PR #824
- B4.4 structural extdeps-fixture-set carrier (`b4-4-structural-extdeps-fixture-set-carrier-worker.md`) — landed PR #825
- T-Substrate cardinality-for-int-lit (`t-substrate-cardinality-int-lit-worker.md`) — landed PR #806
- T-Substrate nominal-opaque-for-Secret (`r2-substrate-nominal-opaque-for-secret-subset.md`) — authored PR #836
- T-Substrate parametric-algebra-for-Dimensions — closed by audit
- T-Substrate ValueBody-list/sum (`t-substrate-valuebody-list-worker.md`) — landed PR #790

Pending — post-spawn manager-authored autonomously:
- **PR-PreF Interval<D> consolidation** worker brief (or Director-authored inline in design doc)
- **T-Substrate-Lens-Primitive** worker brief (gated on PR-K design lock)
- **R3 T-CostLens-Composition** worker brief (gated on Lens<C> primitive landing + R2 close)
- B4.2 implementation dispatch (brief exists; implementation not yet landed)
- B4.5–B4.12 Phase 2 implementation briefs that become live as Phase 1 carriers land

## Working state (fill on spawn)

Spawn refresh, 2026-04-28 (post-#1078, status-refresh against landed PRs):

- **T-Substrate prereqs (R2):** cardinality-for-int-lit producer landed (#806); nominal-opaque substrate landed (#900) + fail-closed field-projection enforcement landed (#937); parametric-algebra closed by audit; ValueBody-list/sum landed (#790); **ValueBody::Map carrier landed (#1017) + tightened (#1068)** — kernel_algebra_profile no longer a "future sub-lane" at substrate level, only consumer plumbing remains.
- **T-Substrate-Lens-Primitive:** NEW lane added 2026-04-28; gated on PR-K cadence lock.
- **PR-PreF Interval<D>:** NEW prepended cadence; foundational substrate consolidation; gates everything else.
- **B4 Phase 1:** B4.1 (#826), B4.3 (#824), B4.4 (#825) landed; B4.2 first-consumer wiring landed.
- **B4 Phase 2:** B4.8 LANDED via #1069 (first Phase-2 dissolution); B4.5/B4.6/B4.7/B4.9–B4.12 still queued.
- **R3 continuation T-CostLens-Composition:** deferred to R3 spin-up; gated on Lens<C> primitive landing.
- **Adjacent landings:** T-Cost-Dimension fail-closed symbolic-cost analysis (#1003) — relevant precedent for Dimensions consumer fail-closed semantics.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Substrate Manager"
- Lens framework spec: `docs/design-lens-framework.md` (Q6+Q7+Q8 locks; 6-field shape with sequential: Monoid<C>)
- Cost algebra spec: `docs/design-emission-model.md` Q3 (`Cost<Unit>` + Bits/CPUCycles)
- Q1-Q5/Q6-Q8 disposition: `docs/r2-structure.md` §4 + design docs
- INVARIANTS substrate-fact-introduction procedure: `INVARIANTS.md` §P1
- B4 program brief: `docs/briefs/b4-identity-carrier-substrate-pass.md`
- Synthesis source: `docs/briefs/debt-paydown-synthesis-2026-04-25.md` §0
- Substrate design: `docs/design-substrate-carrier-port-program.md`
- R3 continuation lane: `docs/r3-structure.md` §"Lane structure" T-CostLens-Composition (Substrate continuation)
- Thesis-claim disposition: `docs/thesis/r2-r3-thesis-mapping.md`
