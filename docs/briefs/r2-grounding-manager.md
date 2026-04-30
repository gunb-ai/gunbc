# R2 Grounding Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827; refreshed 2026-04-28 post-#1078 merge to absorb the engine-reframe cascade — coercion-engine framing replaced with 5 substrate-completion lanes per `docs/design-emission-model.md`). Eligible to spawn pre-R1-close per `r2-structure.md` Transition mechanics step 4 (no technical R1 dependency). Migrates content from [`grounding-manager.md`](grounding-manager.md) (which archives on R2 promotion).

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers. Owns **T-Ground XL** sub-program (Goal 1 — Grounding Completeness, the program with R2's only true critical path).
- **Program scope source:** [`THESIS.md §"Tier 1 — Structural correctness"`](../../THESIS.md) (Grounding completeness sub-claim) + [`ROADMAP.md §"Post-R1 Program — Grounding Completeness"`](../../ROADMAP.md) + [`docs/design-emission-model.md`](../design-emission-model.md) (engine-reframe spec).
- **No-engine emission model — locked direction (per `docs/design-emission-model.md` + #1078):** THESIS:171 is explicit: "Coercion = emission. No separate coercion engine." The prior `T-Ground-Engine` lane name implied an authority that "picks up slack" when structure under-determines targets — that violates fail-closed (P3) + parallel-authority discipline (P2). The reframe **dissolves the engine framing** into 5 substrate-completion lanes that surface the real modeling problems the engine framing was hiding.
- **Cross-program consumer:** Coercion-Fold full pilot enumeration via symbolic walk of `rust_pilot_primitives: List<RustPrimitive>` is gated on Substrate Manager's ValueBody-list/sum + `std.unicode` bootstrap carrier. Coordinate via R1 `Cross-manager notifications queued` brief pattern.
- **Demo coordination:** signal lane-close to R2 Release Manager (closure ledger; per the **structural-acceptance-per-lane-close discipline** locked in `r2-structure.md` — the demo IS the structural gate, not a separate artifact).
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): self-serve through the 3-step decision procedure before escalating substrate-shape questions to Director or Substrate Manager.

## Program scope (T-Ground XL — 11 lanes)

R2's Goal 1 — **Grounding Completeness**. Target-side primitive types (Rust / Python / Go) structurally declared in `.dag` rather than table-driven; coercion **as emission** via algebraic-inhabitance search; Track-13 dissolution.

The one true critical path in R2: `Pilot → Rust → LanguageSpec → Coercion-Fold → Tests → Dissolve`. Python and Go run as fill-queue alongside the critical path.

### 11-lane structure (post-engine-reframe; locked 2026-04-28)

**Foundational (already landed):**
1. **T-Ground-Pilot** S — DONE (PR #765 merged 2026-04-25). Toy inhabitance-search engine for Rust integer family + bool + Unit.

**Per-target primitive declarations:**
2. **T-Ground-Rust** XL — Rust target-spec primitive declarations end-to-end. Critical path because Coercion-Fold blocks on layers 1–3 populated.
3. **T-Ground-Python** L — Python target-spec primitive declarations; not Coercion-Fold-blocking. Fill queue.
4. **T-Ground-Go** L — Go target-spec primitive declarations; not Coercion-Fold-blocking. Fill queue.

**5 substrate-completion lanes (replace prior single Engine lane; locked 2026-04-28 per #1078 cascade):**
5. **T-Ground-Coercion-Fold** S — Refinement composition as a structural fold over substrate facts. Replaces the prior single-engine Phase 2 with a fold over ValueBody-list/sum substrate. Includes mirror-consistency probe footprint (PR #989, currently merged on main; post-merge realignment per `design-emission-model.md` §"Affected lanes" option (c) — hold further engine-framed slices, queue cleanup wave once LanguageSpec lands).
6. **T-Ground-LanguageSpec** M — Structured language-spec substrate declaration. Absorbs target primitive/range duplication retirement (Reflective Pattern E) — `Dag::rust_pilot_primitives()` consumed structurally; `RUST_PILOT_PRIMITIVES` Rust mirror retired. Per Q3 lock: each per-target primitive carries `RealizationCost { storage: Cost<Bits>, access: Map<AlgebraOp, Cost<CPUCycles>> }`.
7. **T-Ground-Lifetime-Analyzer** M — Derives ownership/lifetime structurally from program use. **No annotation surface** — replaces the retracted "Annotation" lane per Modeling problem 3 corrected. Lifetime is structural fact emerging from program shape, not user-authored markup.
8. **T-Ground-Diagnostic** S — Declared structural ordering for diagnostic enumeration; fail-closed diagnostic surface. Q6.5 two-layer authority LANDED via #1129 ([`docs/design-lens-framework.md` §"Q6.5 — Two-layer authority for diagnostic kinds"](../design-lens-framework.md)) — Layer 1 `CompilerDiagnosticKind` is Substrate-owned (this lane consumes; doesn't extend); cross-target diagnostic-ordering work doesn't introduce lens-instance kinds.
9. **T-Ground-CrossTarget-Meta** S — Cross-target uniformity meta-spec. Owns L6 per-(form × target) substrate-load completeness check (different input space from Lens<C> instances; not a structural lens — runtime check). Q5 lock confirmed cardinality-as-connectives axis collapses with PR-PreF Interval<Cardinal>; PR-J likely no-op.

**Closure:**
10. **T-Ground-Tests** S — L4 verification of routing correctness via routing-stability tests + algebra-satisfaction certification.
11. **T-Ground-Dissolve** S — Track-13 dissolution: delete `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` once Coercion-Fold carries the load. Final critical-path step.

## Pre-dispatch design lock cadence (consumed; per #1078 lock)

The 8-question design dialogue resolved into a cadence whose merge gates worker dispatch on this manager's lanes:

| Cadence PR | Locks | Consumer (this manager) |
|---|---|---|
| **PR-PreF** *(Substrate-owned; foundational)* | `Interval<D>` substrate consolidation | All Q1-consuming lanes |
| **PR-F** | Q1 (`BoundDeclaration = StaticBound(Interval<Int>) \| PlatformDependent`; consumes PR-PreF) + Q2 Rust axes | T-Ground-Coercion-Fold + T-Ground-Rust |
| **PR-G** | Q2 Python axes | T-Ground-Python |
| **PR-H** | Q2 Go axes | T-Ground-Go |
| **PR-I** | Q3 (per-primitive realization cost — `Cost<Unit> = Dimension<Unit, SymbolicExpr>`; Bits/CPUCycles primitives) + Q4 (universal four-property gate) | T-Ground-LanguageSpec + T-Verification-L4-L7-Direct (R3) |
| **PR-J** | Q5 (cardinality enumeration — likely no-op given PR-PreF consolidation) | T-Ground-CrossTarget-Meta |

**Asymmetric BoundDeclaration match rule (Q1 lock):** target's `Unbounded` universally accepts; target's `ExactInterval(lo,hi)` requires exact range match. Worker briefs MUST consume this without re-litigation.

## Owned deliverables (through R2 close)

| Lane | Size | Status (at brief authoring) | Description |
|---|---|---|---|
| T-Ground-Pilot | S | DONE (PR #765 merged 2026-04-25) | Toy inhabitance-search engine; routing-stability tests |
| T-Ground-Rust | XL | **PARTIALLY LANDED** — IntegerRangeFact mirror dissolved via #1005 (single-authority `rust_pilot_primitives` + IntegerAlgebra/TargetCarrier identity witnesses + fail-closed bootstrap validation); remainder gated on PR-F | Rust primitive declarations end-to-end; consumes Q1 BoundDeclaration + Q2 Rust axes |
| T-Ground-Python | L | **PARTIALLY LANDED** — `dsl/extdeps/languages/python/primitives.dag` landed via #1080 (lively-wolf-462); remainder fill queue / gated on PR-G | Python primitive declarations |
| T-Ground-Go | L | **PARTIALLY LANDED** — `go/primitives.dag` spec tranche 1 landed (`ac765ce10`); additional Go primitives via #1046 (bright-otter-594); remainder fill queue / gated on PR-H | Go primitive declarations |
| **T-Ground-Coercion-Fold** *(NEW; replaces engine framing)* | S | **PRE-CASCADE FOOTPRINT LANDED** via PR #989 + commit `c0cc8b260` (Phase 2 pilot-list enumeration slice 1; mirror-consistency probe). Per `design-emission-model.md` §"Affected lanes" option (c): hold further engine-framed slices + queue cleanup wave once LanguageSpec lands. Future slices gated on PR-F + Substrate ValueBody-list/sum | Refinement composition as structural fold over substrate facts |
| **T-Ground-LanguageSpec** *(NEW)* | M | **BRIEF LANDED** — [`t-ground-languagespec.md`](t-ground-languagespec.md) (#1168 + nits #1172 + rename #1174); **PHASE 1 LANDED** (registry-backed row population — Rust 9 + Python 18 + Go 13 rows via #1195 + hot-fix #1196); **PHASE 2 PARTIAL** (dead `MethodTranslation` × 3 retired via #1210 + ROADMAP closure #1213); remaining emit-side authorities (`*_method_templates`, `rust_simple_method_specs`, `rust_method_wraps_result`) deferred to Pure-Bootstrap-Zero / v2 retirement per audit. Phase 1.5 (Rust higher-order rows; gated on Substrate substrate-shape decision for the existing dual-template `HigherOrderMethodSpec` carrier (`dsl/extdeps/languages/rust/emit.dag:265`); cross-manager request to jolly-ram-908 (#1130)) + B (`RealizationCost` per-primitive; gated on PR-I) outstanding. | Structured language-spec substrate; absorbs Reflective Pattern E retirement; per-primitive `RealizationCost` |
| **T-Ground-Lifetime-Analyzer** *(NEW; no annotations)* | M | **BRIEF LANDED** — [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md) (#1177); **IMPL LANDED** — R2 scope (a)/(b)/(c) (#1206) + fail-closed Dag-extraction fix (#1218) + path-prefix authority transitional-staging note (#1220) | Structural ownership/lifetime derivation from program use |
| **T-Ground-Diagnostic** *(NEW)* | S | **BRIEF LANDED** — [`t-ground-diagnostic.md`](t-ground-diagnostic.md) (#1216); implementation pending | Declared structural ordering for diagnostic enumeration; `EmissionDiagnostic` carrier |
| **T-Ground-CrossTarget-Meta** *(NEW)* | S | **BRIEF LANDED** — [`t-ground-cross-target-meta.md`](t-ground-cross-target-meta.md) (#1224 + L6 form-axis anchor #1229); **implementation gated on PR-J merge (likely no-op)** | Cross-target uniformity meta-spec; owns L6 per-(form × target) load-completeness check |
| T-Ground-Tests | S | **BRIEF LANDED** — [`t-ground-tests.md`](t-ground-tests.md) (#1223); implementation pending — gated on Q4 lock (PR-I) + Coercion-Fold body | L4 routing correctness verification |
| T-Ground-Dissolve | S | NOT YET AUTHORED | Track-13 dissolution: delete `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` |

## Cross-program dependencies

**Produces (none — Grounding doesn't produce carriers other managers consume).**

**Consumes:**
- **Substrate Manager — ValueBody-list/sum + std.unicode bootstrap carrier**: Coercion-Fold full pilot enumeration via symbolic walk of `rust_pilot_primitives: List<RustPrimitive>` is gated on this. Substrate Manager signals readiness via cross-manager queue; Grounding Manager dispatches Coercion-Fold consumer migration on receipt.
- **Substrate Manager — PR-PreF Interval<D> consolidation**: required for Q1 BoundDeclaration consumer in Coercion-Fold lane.
- **Substrate Manager — `Diagnostic.kind` extensibility (Q6.5 lock LANDED via #1129)**: Diagnostic lane consumes Layer 1 `CompilerDiagnosticKind` (Substrate-owned closed sum). Lane is a Layer-1 consumer, not a Layer-2 lens-instance author — no cross-manager handoff needed.

**Adjacent territory:** none (Grounding owns its program completely; cross-target meta-spec is fully internal).

## Locked design decisions consumed (per #1078 8-question dialogue)

Worker briefs MUST consume these without re-litigation:

- **Q1**: `Interval<D>` shared parent (PR-PreF prepended); `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent`; asymmetric match rule (target's `Unbounded` universal-accepts; target's `ExactInterval(lo,hi)` requires exact range match).
- **Q2 (b3' emission-biased non-violating minimal target modeling)**: `ReferenceModel<T>` shared parent; four-property gate (Faithful / Correct / Minimal / Performant). Each per-target axis (Rust/Python/Go) instances `ReferenceModel<TargetPrimitive>`.
- **Q3**: `RealizationCost { storage: Cost<Bits>, access: Map<AlgebraOp, Cost<CPUCycles>> }`; `Cost<Unit> = Dimension<Unit, SymbolicExpr>`; Bits/CPUCycles substrate primitives sibling to SI base units.
- **Q4**: universal four-property gate (cascaded from Q2) — every reference model verified against the four properties.
- **Q5**: cardinality is the connectives axis; PR-J no-op given PR-PreF consolidation.

Full disposition: [`docs/r2-structure.md`](../r2-structure.md) §4 + [`docs/design-emission-model.md`](../design-emission-model.md) Q1-Q5.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (post-#1078-merge):** brief authoring + cadence sequence locked. Manager spawns once a design-lock PR (PR-F earliest) is dispatchable; pre-R1-close spawn allowed (no technical R1 dependency).
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Ground sub-briefs (11 lanes) without Director.
- Dispatches workers against T-Ground sub-briefs.
- Resolves T-Ground-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every T-Ground worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; every PR introducing hand-Rust under `src/v3/` fills the per-PR gate.
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1) applies to LanguageSpec and Lifetime-Analyzer briefs introducing new substrate types/variants/fields.
- **Cross-program signal authority:** carrier-consumption requests → Substrate Manager via cross-manager queue; per-lane closure → R2 Release Manager (closure ledger).

## Reporting cadence

- Lane-close → R2 Release Manager (closure ledger). Each lane's structural acceptance gate IS the demo per **structural-acceptance-per-lane-close discipline**.
- Cross-program signals (Substrate carrier consumption, Diagnostic.kind coordination) → cross-manager queue.
- Blockers + scope changes → Director.
- **Weekly health surfacing to Director:** which lanes within 1 step of unblocking, which workers fill vs. ready, which cadence PRs (PR-F/G/H/I/J) landed vs pending.

## Acceptance — `.dag` gates

Each lane closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

- `pilot_inhabitance_routing_stability_landed` — DONE (PR #765)
- `rust_target_primitives_declared_structurally`
- `python_target_primitives_declared_structurally`
- `go_target_primitives_declared_structurally`
- `coercion_fold_emission_correct` — refinement composition as fold over substrate facts produces correct emission across pilot enumeration
- `language_spec_realization_cost_landed` — per-primitive `RealizationCost { storage, access }` populated; `RUST_PILOT_PRIMITIVES` mirror retired
- `lifetime_analyzer_structural_derivation_landed` — ownership/lifetime derived from program use; no annotation surface introduced
- `diagnostic_structural_ordering_landed` — declared enumeration order; fail-closed diagnostic surface
- `cross_target_meta_l6_load_completeness_landed` — L6 per-(form × target) substrate-load completeness check fires by construction
- `routing_correctness_l4_verified` — Tests lane closes
- `track_13_dissolution_complete` — `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` deleted

## Sub-briefs (authored / pending)

Authored:
- T-Ground-CrossTarget-Meta brief ([`t-ground-cross-target-meta.md`](t-ground-cross-target-meta.md); implementation pending PR-J)
- T-Ground-Pilot (PR #765, merged)
- T-Ground-Coercion-Fold Phase 1 typestructure (PR #788, merged; framed under prior engine-naming — to be cascaded post-LanguageSpec)
- PR #989 (slice 1 mirror-consistency probe; merged on main pre-cascade — cleanup queued per `design-emission-model.md` §"Affected lanes" option (c))

Pending — post-spawn manager-authored autonomously:
- T-Ground-Rust full implementation (gated on PR-F)
- T-Ground-Coercion-Fold Phase 2 implementation (gated on PR-F + Substrate ValueBody-list/sum)
- T-Ground-LanguageSpec brief + Phase 1 + Phase 2 partial — **LANDED** (#1168 + #1172 + #1174 + #1195 + #1196 + #1210 + #1213); Phase 1.5 (Rust higher-order; gated on Substrate substrate-shape decision for the existing dual-template `HigherOrderMethodSpec` carrier (`dsl/extdeps/languages/rust/emit.dag:265`); cross-manager request to jolly-ram-908 (#1130)) + B (`RealizationCost` per-primitive; gated on PR-I) outstanding
- T-Ground-Lifetime-Analyzer brief + R2 scope (a)/(b)/(c) impl — **LANDED** (#1177 + #1206 + #1218 + #1220); R3 scope (d)/(e)/(f) closures/async/Pin folds into R3 `T-LensProducer-Retirement`
- T-Ground-Diagnostic (S) — brief [`t-ground-diagnostic.md`](t-ground-diagnostic.md) **LANDED** (#1216); implementation pending
- T-Ground-CrossTarget-Meta (S; gated on PR-J no-op confirmation) — brief [`t-ground-cross-target-meta.md`](t-ground-cross-target-meta.md) **LANDED** (#1224 + #1229); implementation pending
- T-Ground-Tests (S) — brief [`t-ground-tests.md`](t-ground-tests.md) **LANDED** (#1223); implementation pending — gated on Q4 (PR-I) + Coercion-Fold body
- T-Ground-Dissolve (Track-13 cleanup)
- T-Ground-Python
- T-Ground-Go
- **Cleanup wave**: post-LanguageSpec, retire PR #989's engine-framed structure into the Coercion-Fold framing (per `design-emission-model.md` §"Affected lanes" option (c))

## Working state (fill on spawn)

Spawn refresh, 2026-04-28 (post-#1078 engine-reframe cascade + status-refresh against landed PRs):

- 11-lane structure replaces the prior 7-lane Pilot/Rust/Engine/Tests/Dissolve framing.
- 5 new substrate-completion lanes added (Coercion-Fold S + LanguageSpec M + Lifetime-Analyzer M + Diagnostic S + CrossTarget-Meta S).
- **Per-target work has begun pre-cadence:** Rust IntegerRangeFact mirror dissolved (#1005); Python primitives.dag landed (#1080); Go primitives spec tranche 1 + additional Go primitives landed (`ac765ce10` + #1046). Manager-spawn task: align this work with Q1 BoundDeclaration + Q2 axes once PR-F/G/H land; may require small refactors to the already-landed primitive declarations.
- PR #989 (slice 1 of Phase 2) merged on main pre-cascade — cleanup wave queued; hold further engine-framed slices.
- Cadence PR-F through PR-J gate worker dispatch on dependent lanes; PR-PreF (Substrate-owned) gates Q1-consuming lanes.
- The "Annotation" lane was retracted in favor of structural derivation per Modeling problem 3 corrected; Lifetime-Analyzer replaces it.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Grounding Manager" (now 11 lanes; engine-reframe locked 2026-04-28)
- Engine-reframe spec: `docs/design-emission-model.md` (no-engine emission discipline; 8 worked examples; Q1-Q5 locks)
- Q6-Q8 (lens framework): `docs/design-lens-framework.md`
- INVARIANTS substrate-fact-introduction procedure: `INVARIANTS.md` §P1
- Migrating from: `docs/briefs/grounding-manager.md` (archives on R2 promotion)
- Pre-spawn engineering: `docs/briefs/grounding-pilot-receipt.md`, `docs/briefs/t-ground-engine-substrate-escalation.md` (both pre-cascade — post-cascade naming aligns with new lane structure)
- Thesis-claim disposition: `docs/thesis/r2-r3-thesis-mapping.md`
