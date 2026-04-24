# R2 Structural Close Manager Brief

**Status:** `PROPOSAL` — pending R2 proposal
([`docs/r2-structure.md`](../r2-structure.md), PR #754) merge +
R1 all-gates-green closure. Until then, this brief is the R2
Structural Close Manager's scope-in-preparation; it does not
dispatch work. On R2 promotion, the `PROPOSAL` banner lifts and
this brief becomes the live dispatch authority for the manager.

**Naming rationale.** The scope below covers six lanes spanning
substrate carrier close, substrate prereqs, modeling-faithfulness,
shim floor, lens migration, and impossible-bug class closure.
"Self-hosting" is only accurate for two of those; "Structural
Close" names the actual thesis-level activity — closing the
remaining structural work so every Tier-1 / Tier-2 / Tier-3 claim
has named structural backing rather than convention or scaffolding.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md)
  (in PROPOSAL mode at PR #754). Names this manager as one of two
  R2 standing managers (alongside R2 Grounding Manager) and
  locates the six lanes listed below within the R2 program. On
  promotion, the authoritative section becomes
  `ROADMAP.md §"Release R2 Program"`.
- **Thesis claims tracked:** [`THESIS.md`](../../THESIS.md)
  §"Thesis claims — complete list" — multiple claims across
  Tier 1 / Tier 2 / Tier 3 and the Epistemic-stacking /
  Substrate-shape meta-claims. The specific claim → lane
  mapping lives in the pre-promotion thesis-claim coverage
  audit (R2 proposal Open Call 2); on promotion the audit
  table is linked here.
- **Design authorities this manager operationalizes:**
  - [`docs/design-substrate-carrier-port-program.md`](../design-substrate-carrier-port-program.md)
    — E-family carrier port program; §6a per-method-metadata
    call deferred from R1 decides in R2.
  - [`docs/design-pure-bootstrap.md`](../design-pure-bootstrap.md)
    — ≤5 irreducible-shim floor; SG-0 census partition.
  - [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md)
    — per-lens capability tracking; T-LensMigration gate reads
    `lens_producer_files_remaining → 0` from this register.
- **R1 precedents:** on R2 promotion, R1 Substrate /
  Self-hosting / Testgen / Surface Managers archive with
  closure banners. Their briefs remain in-tree as historical
  references. E-family work, T-PB-A / T-PB-B work, and the
  testgen predicate tail fold here. R1 Surface's emit-pipeline
  boundary responsibilities fold here for the parts R2 touches.
- **Coordination context:** on R2 promotion, this brief's
  coordination context becomes the R2 Director Brief
  (refactored from [R1 Director Brief](r1-director-brief.md)'s
  Staffing section). Scope changes route to director; manager
  owns lane-level dispatch inside scope.

## Slice

This manager owns six R2 lanes. The seventh R2 lane (T-Ground) is
owned by the R2 Grounding Manager; the eighth (cross-lane demo
discipline) is Director ad-hoc.

| Lane | Size | R2 Goal | Covers |
|---|---|---|---|
| **T-EFamilyClose** | M | 7 | E-I finish (if still in flight at R2 open) → E-P → E-M → §6a per-method-metadata call per `docs/design-substrate-carrier-port-program.md`. Scope reduces to §6a alone if E-I / E-P / E-M close as part of R1. |
| **T-ShimFloor** | M | 3 | T-PB-A non-lens-producer hand-Rust reduction toward ≤5 irreducible-shim floor; T-PB-B outside-residual-zero (`TESTING.md §Post-R2 shape` carve-out preserved); compiler–`std/` consolidation ratchet → 0. |
| **T-LensMigration** | L | 2 | Every lens producer `.rs` file → `.dag`-authored. Per-file parallel fill queue. Gate: `lens_producer_files_remaining → 0` (introduced via PR #752). Closes the "lens purity by construction" co-anchor claim. |
| **T-Modeling** | M | 4 | Three modeling-faithfulness dissolutions: surface int-literal magnitude at concept layer (PR #745 analysis); `Secret<T>` nominal-opaque graduation; `Dimension<Carrier>` phantom-parameter unit-mismatch enforcement. Each blocks on its paired T-Substrate sub-lane. |
| **T-Substrate** | M | 5 | Three scoped-subset substrate sub-lanes, each sufficient to unblock its paired T-Modeling item (not full substrate-capability close): (a) cardinality-substrate subset for int-literal magnitude; (b) nominal-opaque substrate subset for `Secret<T>`; (c) parametric-algebra-attachment subset for `Dimension<Carrier>`. Full-capability substrate work outside these subsets stays outside R2 scope unless other R2 items demand it. |
| **T-ImpossibleBugs** | S | 6 | Three remaining `[R2+]` impossible-bug classes tagged at `ROADMAP.md:72` (THESIS §"Enumerable impossible-bug classes" is the authority on scheduling tags): nested-optional flatten; unhandled diagnostic paths; unenumerated effects. Sparse lane — fills when other fill queues are saturated. |

## Framing question this manager answers

**Does every remaining Tier 1 / Tier 2 / Tier 3 thesis claim not
closed by R1 reach structural closure — carrier-port complete,
shim floor reached, lens producers all `.dag`-authored, modeling-
faithfulness gaps dissolved, substrate prereqs met for those
dissolutions, and the three remaining impossible-bug classes
proven impossible by construction?**

Today (pre-R2-promotion state):
- E-family carrier port is most of the way through R1 (E-T / E-C
  landed; E-I in flight; E-P / E-M work staged). Whether E-P / E-M
  close in R1 or inherit as R2 work depends on R1 closure
  criterion application; this brief scopes T-EFamilyClose to
  cover whatever remains at R2 open.
- T-PB-A reduction wave is active in R1; SG-0 non-test census is
  tracked live in `src/v3/compiler/tests/integration/sg0_census_test.rs`
  with the ≤5 floor as target.
- `lens_producer_files_remaining` gate is named but pre-zero; the
  set of lens producers still in `.rs` is enumerated in the R1
  self-hosting manager's T-PB-A reduction wave.
- The three modeling-faithfulness items (int-lit, `Secret<T>`,
  Dimensions) have named dissolution directions in R1 ledger rows
  but no compile-time enforcement yet.
- The three `[R2+]` impossible-bug classes are tagged but not yet
  dispatched; they wait on Tier 2 substrate that closes in R2.

The ask: close all six lanes. When this manager's scope closes
alongside R2 Grounding, the thesis-claim release ledger has no
open thesis-level items; post-R2 work is external (adoption,
documentation, community).

## Sequence + dispatch

**Critical path** is T-EFamilyClose. The other five lanes are
fill queues operating in parallel with the critical path; any
available worker picks top-priority unblocked work.

- **Day 1 (R2 open).** T-EFamilyClose dispatches. Whatever E-family
  work is still open at R2 open lands first. §6a per-method-
  metadata option pick happens here; the four options in
  `docs/design-substrate-carrier-port-program.md:169-197` are
  evaluated against E-I/E-P evidence gathered in R1.
- **Day 1.** T-LensMigration dispatches as per-file-parallel fill
  queue. Per-file retirements land as separate PRs; SG-0 ratchet
  + `lens_producer_files_remaining` gate observe progress. No
  cross-lane blocker.
- **Day 1.** T-ShimFloor dispatches as per-file-parallel fill
  queue for the non-lens-producer subset of T-PB-A. T-PB-B work
  (outside-residual-zero) runs in the same fill queue.
- **Day 1.** T-Substrate dispatches three sub-lanes in parallel.
  Each sub-lane's close criterion is its paired T-Modeling
  unblock, not general substrate-capability completion. The
  DB-18 ↔ ROADMAP label mismatch flagged in the R2 proposal
  (cross-ref at `docs/r2-structure.md §"Cross-refs"`) is
  resolved pre-dispatch: either the ROADMAP row relabels to
  match db-history's DB-18 scope, or a new DB number issues for
  the R2 parametric-algebra-attachment subset.
- **Gated on T-Substrate sub-lanes.** T-Modeling's three items
  dispatch in parallel, each unblocked by its paired T-Substrate
  sub-lane. Int-lit blocks on cardinality-substrate subset;
  `Secret<T>` blocks on nominal-opaque substrate subset;
  Dimensions blocks on parametric-algebra-attachment subset.
- **Gated on Tier 2 substrate reaching usable state.**
  T-ImpossibleBugs's three classes dispatch as fill. Nested-
  optional flatten and unhandled-diagnostic-paths depend on Tier 2
  runtime-safety proof infrastructure (reaches usable state as
  T-Modeling's int-lit + Dimensions work closes the proof
  surface); unenumerated-effects depends on effect-system surface
  closure (inherited from R1).

Lane ordering is a **fill-queue model**, not strict sequential.
T-EFamilyClose is the sole critical path. Cross-lane dispatch
failures surface as blocked workers on specific sub-lanes;
manager reassigns to unblocked queues.

## Hand-off points

R2 manager structure is **2 standing managers + Director** per
[`docs/r2-structure.md`](../r2-structure.md).

- **Sideways to R2 Grounding Manager.** Grounding's engine work
  (T-Ground-Engine) replaces the declared-carrier read path
  through the emit pipeline; Structural Close owns the emit-
  pipeline boundary in R2. Coordinate on:
  - Emit-pipeline changes Grounding's engine requires —
    Structural Close authors or reviews.
  - Substrate-capability overlap: Grounding's T-Ground-Rust /
    -Python / -Go lanes block on cardinality-substrate + DB-11
    closure. Structural Close's T-Substrate sub-lanes are
    scoped to R2 T-Modeling unblocks specifically; if Grounding
    needs broader cardinality-substrate work, flag upward
    rather than silently expanding T-Substrate scope.
  - `TestClaim` variants Grounding introduces (routing-
    stability, L4 witness-based): if these require new testgen
    predicate extensions, route to Director for ad-hoc
    dispatch (no standing Testgen manager in R2).
- **Sideways to Director.** The following route through Director
  as ad-hoc dispatches rather than standing-manager authority:
  - New testgen predicate schema extensions beyond what R1
    shipped (`ExecuteCommand`, `ForAllTargets`, `LensOutputEquals`,
    `DifferentialEquals`, `AlgebraicLaw`, `MockBackedInvariant`).
  - Release-demo artifacts per R2 proposal's Demo discipline —
    each lane closure PR ships a "here it runs" artifact;
    Director surfaces to user.
  - Cross-manager dependency surfacing when critical paths
    block.
- **Up to director.** Substrate scope-creep flags: if a
  T-Substrate sub-lane surfaces work broader than its scoped
  acceptance criterion, escalate rather than absorb. The R2
  proposal explicitly commits T-Substrate to "subset sufficient
  to unblock T-Modeling," not full-capability close — scope
  drift would re-open a design question the proposal already
  resolved.
- **Up to director.** §6a per-method-metadata option pick is
  a design call informed by E-I/E-P/E-M evidence. Before
  committing the option, surface the evidence summary to
  director for review (matches the R1 precedent for E-P
  attachment-shape design call).
- **Up to director.** Any proposal to amend thesis-claim
  closure criteria for a lane (e.g., T-LensMigration shipping
  with `lens_producer_files_remaining = N > 0` as "close
  enough") is a scope change.

## Working state

Lane-owner dispatch status (update as sub-deliverables close).
This section populates on R2 promotion; entries below are
scaffolding.

**T-EFamilyClose:**
- [ ] E-I landed (may close in R1; if so, strike through)
- [ ] E-P landed (may close in R1; if so, strike through)
- [ ] E-M landed (may close in R1; if so, strike through)
- [ ] §6a per-method-metadata option picked + applied
  (Option 0 lens-local / Option 1 type-decl annotations /
  Option 2 per-algebra metadata carrier / Option 3 unified
  `MethodContract` — per `docs/design-substrate-carrier-port-program.md:169-197`)

**T-ShimFloor:**
- [ ] Non-lens-producer T-PB-A reductions to ≤5 irreducible-shim floor
  (live baseline: `src/v3/compiler/tests/integration/sg0_census_test.rs`
  `EXPECTED_HAND_AUTHORED_NON_TEST`)
- [ ] T-PB-B outside-residual-zero per `TESTING.md §Post-R2 shape`
- [ ] Compiler–`std/` consolidation ratchet → 0
- [ ] `pb_hand_rust_at_shim_floor` predicate evaluates true
- [ ] `pb_rust_tests_outside_residual_zero` predicate evaluates true
- [ ] `pb_self_compile_fixed_point` predicate evaluates true
- [ ] `pb_compiler_std_ratchet_zero` predicate evaluates true

**T-LensMigration:**
- [ ] Lens-producer enumeration table (snapshot at R2 open)
- [ ] Per-file `.rs` → `.dag` migrations (per-file parallel fill)
- [ ] `lens_producer_files_remaining → 0` gate evaluates true

**T-Modeling:**
- [ ] Int-literal magnitude at concept layer — lowered + enforced
      (blocks on T-Substrate cardinality-subset)
- [ ] `Secret<T>` nominal-opaque graduation — construction-
      restriction compile-time enforced
      (blocks on T-Substrate nominal-opaque-subset)
- [ ] `Dimension<Carrier>` phantom-parameter arithmetic —
      unit-mismatch compile-time enforced
      (blocks on T-Substrate parametric-algebra-attachment-subset)

**T-Substrate:**
- [ ] Cardinality-substrate subset sufficient to close int-literal
      magnitude refinement
- [ ] Nominal-opaque substrate subset sufficient to graduate `Secret<T>`
- [ ] Parametric-algebra-attachment subset sufficient to inhabit
      `Dimension<Carrier>` in an abelian group algebra
- [ ] DB label reconciliation (ROADMAP ↔ db-history DB-18 scope
      mismatch resolved pre-dispatch)

**T-ImpossibleBugs:**
- [ ] Nested-optional flatten impossible-by-construction
- [ ] Unhandled diagnostic paths impossible-by-construction
- [ ] Unenumerated effects impossible-by-construction

Decisions log (append as they happen):

- _(none yet — brief is pre-dispatch)_

Open questions for director:

- _(none yet — brief is pre-dispatch)_

Cross-manager notifications queued:

- _(none yet — awaits R2 open + Grounding Manager dispatch
  signals)_
