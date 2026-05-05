# Thesis-claim coverage mapping — R1 / R2 / R3 / post-R3

**Status:** `LIVE` (2026-04-28). Closes Open call 1 from [`docs/r2-structure.md`](../r2-structure.md). Live thesis-claim coverage authority for ongoing R2/R3 amendments.

**Authority:** [`THESIS.md`](../../THESIS.md) §"Thesis claims — complete list" is the single source on which claims exist. This doc is the disposition table.

**Audit format:** every Tier-1 / Tier-2 / Tier-3 claim from THESIS gets a row with disposition (R1 / R2 / R3 / post-R3-external) + gate or lane name + evidence pointer + status. Gaps (claim named in THESIS but not mapped) are flagged inline.

## How this doc operates

- THESIS is single-source on which claims exist. Adding a claim there is a thesis commitment per `feedback_modeling_philosophy`.
- This doc is single-source on which release closes which claim. Adding a claim to a release here is a scope commitment.
- ROADMAP / r2-structure.md / r3-structure.md are single-source on lane / gate / acceptance details. This doc cross-references them.
- When a claim moves dispositions (e.g., R2 → R3), the corresponding rows in r2-structure.md and r3-structure.md update in the same PR.

## Disposition table — Tier 1 (Structural correctness)

Per THESIS §"Tier 1 — Structural correctness (impossible to write the bug)":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Type mismatches caught at compile time** | R1 | T-LaneE / Tier-1 type system | Inference + constraint solver (live since M1) | ✅ live |
| **Field typos caught at compile time** | R1 | T-LaneE / Tier-1 type system | Inference + Conj field resolution (live) | ✅ live |
| **Non-exhaustive matches caught at compile time** | R1 | T-LaneE / `sub_match_over_user_sum` | PR #702 | ✅ live |
| **Bare container types rejected at compile time** | R1 | T-LaneE / Tier-1 type system | Cardinality substrate (live) | ✅ live |
| **Circular dependencies caught at compile time** | R1 | T-LaneE / Tier-1 type system | Bounded-forward-execution premise (P4) | ✅ live |
| **Stale imports caught at compile time** | R1 | T-LaneE / Tier-1 type system | Module resolution + strict reference (live) | ✅ live |
| **Cross-target drift caught at compile time** | R2 | T-Substrate / SourceFiltering canonical authority | PR #1004 (Worker 2 brief) | ✅ landed in R2 |
| **CX gate: every recursive function terminates with a proven bound** | R1 | T-LaneE / `complexity_merge_sort_*` | E-family carrier port (R1 lane work) | 🟡 R1 closure |
| **Coercion = emission (no separate coercion engine)** | R3 | T-Verification-L4-L7-Direct (L4 emit/eval match) + T-Ground-Dissolve | R2-T-Ground-Dissolve closes; R3 verifies via L4 | 🟡 R2 partial / R3 close |
| **Ownership: no aliased mutation in emitted code** | R1 | T-LaneE / ownership lens | Live since L1 (CM-inventory) | ✅ live |
| **Grounding completeness — Rust target primitives structurally modeled** | R2 | T-Ground-Rust XL | R2 amendment 2026-04-28 (folds Rust XL into R2 scope) | 🟡 in flight (Pilot ✓; PR #989 slice-1 merged with engine framing — post-merge cleanup queued per [`docs/design-emission-model.md`](../design-emission-model.md) §"Affected lanes (post-merge realignment)"; Rust dispatch pending) |
| **Grounding completeness — Python target primitives structurally modeled** | R2 | T-Ground-Python L | R2 amendment 2026-04-28 (folds Python L into R2 scope) | ⏳ pending dispatch (post-R1 close) |
| **Grounding completeness — Go target primitives structurally modeled** | R2 | T-Ground-Go L | PR #910 (primitives.dag tranche 1) | ✅ landed in R2 |
| **Grounding completeness — algebra-homomorphism search (not name-keyed lookup)** | R2 | **5 NEW substrate-completion lanes** (per [`docs/design-emission-model.md`](../design-emission-model.md) engine reframe replacing the retracted T-Ground-Engine): T-Ground-Coercion-Fold S + T-Ground-LanguageSpec M + T-Ground-Lifetime-Analyzer M + T-Ground-Diagnostic S + T-Ground-CrossTarget-Meta S. Plus existing T-Ground-Dissolve S as a sibling lane in the 11-lane Grounding program (pre-existing, not part of the 5-new reframe) | Engine framing retracted via PR #1078; PR #989 slice-1 merged under prior framing, post-merge cleanup queued (option (c) in design doc); Track-13 dissolution PR pending | 🟡 5-new-lane substrate-completion structure pending dispatch (T-Ground-Dissolve is sibling, not part of the 5); PR #989 slice-1 cleanup queued |
| **Sealed-accessor patterns at type level (Secret<T>)** | R2 | T-Modeling Secret<T> + T-Substrate NominalOpacity | #900 carrier ✓; #937 walker ✓; PR A in flight (Copernicus); PR B pending | 🟡 in flight |
| **`AnalysisDimension<Carrier>` proof-dimension framework (one-parameter; behavioral analysis carrier)** | R2 | T-Modeling Dimensions | PR #886 — landed `AnalysisDimension<Carrier>` per `src/v3/std/dimensions.dag` (`name` / `witness_of: fn(Dag, Behavior) -> Witness<Carrier>` / `compose: Monoid<Carrier>` / `break_diagnostic`). Note: PR #886 also landed the separate phantom value wrapper `Dimension<Unit, Carrier>` (two parameters; typed-value-wrapper for `Duration<Seconds>`-shape values). The substrate deliberately split these into distinct types. PR #1607 (F2 dispatch) collapsed the prior `compose: fn(C,C)->C` + `identity: C` field pair into a single `compose: Monoid<Carrier>` field, mirroring the `Lens<C>` precedent (`sequential: Monoid<C>`) and dissolving the same drift the lens framework retired on 2026-04-28 — monoid-law authority now lives once on `Monoid<Carrier>` (`dsl/std/algebra.dag:110`) via structural inhabitance | ✅ landed in R2 (collapse merged in R3 substrate cleanup, PR #1607) |
| **Phantom-parameter typed value wrappers (`Duration<Unit>`, `Money<Currency>`)** | post-R3 modeling | not yet a named lane | ROADMAP `:450` — "The live tree does not yet support this shape." Not the same as `Dimension<Carrier>`; requires substrate for typed value wrappers with phantom parameters propagating through arithmetic + algebra inhabitance for those wrappers (abelian group with compare, no multiplication). Adjacent to DB-18 user-defined parametric algebra attachment | ⏳ post-R3 (no lane) |
| **User-authored lenses validate programs (THESIS §"User-defined dimensions")** | R1 + R3 verification | T-LensAPI (R1) + T-Verification-L4-L7-Direct (R3 verifies the claim end-to-end via L4 emit/eval match + L7 algebraic-law witnesses) | T-LensAPI lane R1 — `user_authored_lens_compiles` Day-1 gate; `lens_composition_associative` ext gate via `AlgebraicLaw` | 🟡 R1 closure for compile-side; R3 for runtime-validation receipt |
| **Fabrication path closures (B-series)** | R2 | T-Substrate B-wave Tier 0 | PR #817 (B2 Arrow re-derive→fail-closed); PR #820 (B1 Go UnknownVariant); PR #821 (B3 fold template-formal) | ✅ landed in R2 |

**Tier 1 gaps from THESIS:** none identified — every Tier-1 sub-claim mapped.

## Disposition table — Tier 2 (Runtime safety)

Per THESIS §"Tier 2 — Runtime safety (proven safe or total)":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Division by zero — proven safe or total** | R2 | T-ImpossibleBugs Class 2 (unhandled-diagnostic-paths via DivError) | PR #969 iterating CI | 🟡 in flight |
| **Integer overflow — proven safe at i64-bounded carrier** | R2 | T-Modeling int-lit + T-Substrate cardinality | PR #897 (i64-bounded consumer) | ✅ landed in R2 |
| **Integer overflow — proven safe at full magnitude (any refinement, including unbounded)** | R3 | T-Numeric-Construction (L-XL; reframed 2026-05-01 from T-Int128 — absorbs the overflow claim into refinement-parametric form) | r3-structure.md §T-Numeric-Construction; design doc `docs/design-numeric-construction.md` | ⏳ R3 dispatch |
| **Out-of-bounds — proven safe or total** | R2 | T-ImpossibleBugs Class 2 (unhandled-diagnostic-paths) | PR #969 iterating | 🟡 in flight |
| **Force-unwrap — proven safe or total** | R2 | T-ImpossibleBugs Class 1 (nested-optional flatten) | PR #890 ✓; PR #962 follow-ups ✓ | ✅ landed in R2 |
| **Partial functions — made total** | R2 + R3 | T-ImpossibleBugs (R2 partial) + T-Verification-L4-L7-Direct (R3 verifies totality via L4 emit/eval match — no failed evaluations) | R2 dissolves call sites; R3 harness verifies no partials remain | 🟡 R2 partial / R3 close |

**Tier 2 gaps from THESIS:** none identified — every Tier-2 sub-claim mapped, with int-lit's Int128/Word128 closure deferred to R3 (per existing R2 scope decision: "full Int128/Word128 closure remains separate Substrate scope").

## Disposition table — Tier 3 (Verification from structure)

Per THESIS §"Tier 3 — Verification from structure":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **L4: emitted code matches .dag evaluation** | R3 | T-Verification-L4-L7-Direct / `l4_emit_eval_match` | r3-structure.md §T-Verification-L4-L7-Direct | ⏳ R3 dispatch (gated on R2-Evaluator) |
| **L5: same .dag → same behavior in Rust/Python/Go** | R3 | T-Verification-L5-Corpus / `l5_cross_target_consistency` | r3-structure.md §T-Verification-L5-Corpus | ⏳ R3 dispatch (gated on R2-Evaluator + R2-Grounding-Rust + R2-Grounding-Python + T-Verification-L4-L7-Direct corpus) |
| **L6: every structural form compiles to every target** | R2 (structural fold, not runtime) | T-Ground-CrossTarget-Meta / `l6_structural_form_coverage` (cross-product fold over substrate × language-specs; compile-time checkable, no corpus or runtime) | Reclassified 2026-04-28 per Codex Pattern B finding: classifying L6 as corpus-driven verification let runtime authority gate a structurally-checkable property — same anti-pattern as the omni-coherence finding. The fold walks `(6 type connectives × 5 behaviors × cardinality variants) × Shape A targets` and verifies each pair has an emission path declared. Lane home: R2-T-Ground-CrossTarget-Meta (structural acceptance gate, not runtime check) | ⏳ R2 (structural acceptance gate) |
| **L7: operations obey declared algebraic laws** | R3 | T-Verification-L4-L7-Direct / `l7_algebraic_laws_witnessed` | r3-structure.md §T-Verification-L4-L7-Direct (witness construction via R2-Evaluator) | ⏳ R3 dispatch |

**Tier 3 gaps from THESIS:** none identified. **R3 verification surface = {L4, L5, L7}** = three runtime-verification claims. **L6 was reclassified to R2** as a structural cross-product fold (lives in T-Ground-CrossTarget-Meta) per Codex Pattern B finding 2026-04-28 — see L6 row above. The four THESIS levels are still all mapped, just split between R3 (runtime) and R2 (structural).

## Disposition table — Concept unifications

Per THESIS §"Concept unifications":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Coercion cost = complexity** | R3 | T-CostLens-Composition / `cost_lens_reads_target_realization` + `coercion_cost_equals_complexity_by_construction` + `no_coercion_cost_dimension` (R3 lane 10; Director-locked 2026-04-28; **owned by Substrate Manager (post-R2 continuation)** per Director directive 2026-04-28 — substrate-authoring of cost facts; Substrate authors / Verification asserts). Plus R2 substrate facts: per-operation cost on `dsl/std/algebra.dag` + per-primitive realization cost on language specs. Plus R2-T-Substrate-Lens-Primitive (T-CostLens-Composition is an instance of `Lens<C>` with `C = SymbolicCost`) | Per [`docs/design-emission-model.md`](../design-emission-model.md) Modeling problem 8 — cost lens reads (1) `.dag` algebra-level cost + (2) target-primitive realization cost via the language spec; composes structurally; verifies the unification holds by construction (not by reviewer convention). No "coercion cost" dimension | ⏳ R3 dispatch (gated on R2-Evaluator + **R2-T-Substrate-Lens-Primitive** + R2-T-Substrate per-op algebra cost + R2-T-Ground-LanguageSpec per-primitive realization cost) |
| **Coercion = emission** | R3 | T-Ground-Dissolve (R2 closes coercion scaffolding; R3 verifies emission-as-coercion claim) | T-Ground-Dissolve PR (R2) + R3 harness | 🟡 R2 close + R3 verify |
| **Target language spec = transport spec = interpreter runtime** | R3 | T-Verification-L5-Corpus (cross-target equivalence under shared interpreter — this is the L5 claim) | r3-structure.md | ⏳ R3 |
| **Idempotency + cancellation + redundancy = algebraic simplification** | R2 | T-ImpossibleBugs Class 3 (unenumerated effects — already exercises this via algebra inhabitance) | PR #971 ✓ | ✅ landed in R2 |

## Disposition table — Epistemic stacking

Per THESIS §"Epistemic stacking (load-bearing for codegen — must not be dropped)":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Every concept is a node in an ontological DAG rooted at minimal primitives** | R1 + R2 | T-Substrate (primitive substrate) + INVARIANTS P1 enforcement | `dsl/std/algebra.dag` is authority; live | ✅ live (continuous discipline) |
| **Concrete types attach by inhabitance (algebra-inhabitance carrier shape)** | R2 | T-Substrate parametric algebra + T-Modeling `Dimension<Carrier>` proof-dimension framework | PR #886 (proof-dimension framework) + parametric algebra (R2 carriers) — substrate shape that *enables* algebra-inhabitance declarations; the live tree does not yet have a phantom-parameter typed-value-wrapper consumer per ROADMAP `:450` | 🟡 carrier-shape landed in R2; phantom-parameter consumer is post-R3 modeling work |
| **Operations fall out from algebra inhabitance** | R3 | T-Verification-L4-L7-Direct (L7 algebraic laws witnessed via Evaluator) | r3-structure.md | ⏳ R3 |
| **Math primitives and domain primitives share one substrate** | R2 | T-Substrate + T-Modeling | NominalOpacity + Dimension<Carrier> on same substrate | ✅ landed in R2 |

## Disposition table — Substrate shape

Per THESIS §"Substrate shape (two coordinated substrates — must not be flattened)":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Types are Node trees with 6 connectives** | R1 | Live since M1 | Substrate stable; no C1 stop signal triggered | ✅ live |
| **Computation is 5 L1 behaviors** | R1 | Live since M1 (M0 validation) | Three reviewer rounds; stop signal never fired | ✅ live |
| **Substrate extension is C1-class stop signal** | R1 + R2 | Continuous discipline | R2 added significant capacity (NominalOpacity, ValueBody::Map, parametric algebra) all *inside* existing substrate | ✅ live (continuous discipline) |

## Disposition table — Free consequences

Per THESIS §"Free consequences (fall out when Tiers 1-2 close)":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Automatic parallelism from dependency graph** | R3 | T-Free-Consequences-Demonstration / `auto_parallelism_*` + `auto_loop_parallelism_*` gates | `docs/design-free-consequences.md` + 6 parallelism TestClaims; `Lens<Bind-Independence>` / `Lens<Iteration-Independence>` + `Lens<Effect-Commutativity>` + `Lens<Cost>` | 🟡 R3 |
| **Automatic memoization from purity + cost** | R3 | T-Free-Consequences-Demonstration / `auto_memoization_*` gates | `docs/design-free-consequences.md` + 2 memoization TestClaims; `Lens<Purity>` + `Lens<Cost>` | 🟡 R3 |
| **Incremental cross-run execution from purity + bounded execution + dependency graph** | post-R3 (indirect) | No dedicated R3 gate; falls out from existing Tier 1 + Tier 2 commitments | THESIS:208 + what-else-falls-out.md §"Incremental cross-run execution"; **post-R3 tracked, not a live capability until consumer artifact lands** — one `.dag` TestClaim or runner path or interpreter cache path proving changed inputs ⇒ re-executed and unchanged inputs ⇒ cache hit | ⏳ post-R3 (indirect) |
| **T-Incremental-Cross-Run-Demo** (named consumer-proof artifact for the row above) | post-R3 named | No dedicated R3 lane; named-but-deferred consumer-proof obligation only | Acceptance criteria: demonstrate (a) content-hash keying — pure expression result keyed by `hash(structural_form, input_hashes)`; (b) dependency invalidation — source change to subgraph X re-executes only subgraphs depending on X; (c) one `.dag` TestClaim or runner path or interpreter cache path proving changed inputs ⇒ re-executed and unchanged inputs ⇒ cache hit. Closes the SHIP_WITH_DEBT consumer-proof gap from PR #1738 meta-review (gpt-5-5-pro 2026-05-05) | ⏳ post-R3 (named) |
| **Space bound proofs from CX** | R1 | T-LaneE complexity-lens gates | E-family carrier port | 🟡 R1 closure |
| **Cross-language optimization from shared cost algebra** | R3 | T-Free-Consequences-Demonstration / `cross_target_optimization_*` gates | `docs/design-free-consequences.md` + 2 cross-target optimization TestClaims; `Lens<Cost>` + `LanguageSpec` | 🟡 R3 |

**Note on "free consequences":** these are *consequences*, but the 2026-04-30
R3 expansion operationalizes three of them as Lane 3 demonstration deliverables:
`docs/design-free-consequences.md` plus the 10-gate TestClaim suite named in
`docs/r3-structure.md`. Space-bound CX remains assigned to R1/T-LaneE authority
and is referenced, not re-derived, by Lane 3.

## Disposition table — Omni-emission

Per THESIS §"Omni-emission (1:1 effort applied to full-stack systems)":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **One workflow declaration projects onto every layer of a real application** | R3 | T-Omni-Shape-B (≥2 demos) | r3-structure.md §T-Omni-Shape-B | ⏳ R3 dispatch |
| **Coherence between layers is structural, not checked** | R3 (lane-local structural gate) | T-Omni-Shape-B / `omni_layers_share_one_node_tree` (structural acceptance predicate per THESIS:213 — verifies same-Node-tree derivation per workflow; distinct from L4/L5 runtime equivalence checks) | r3-structure.md §T-Omni-Shape-B; coherence is structural-by-construction (same Node tree) — the lane-local gate verifies the demos satisfy that construction, not that the harness establishes coherence | ⏳ R3 dispatch (gate is structural; not runtime equivalence) |
| **Shape A: O(1) per language target** | R2 | T-Ground (Rust + Python + Go) | 3 targets fully grounded by R2-close | 🟡 in flight |
| **Shape B: O(1) per artifact class** | R3 | T-Omni-Shape-B (≥2 Shape B targets demonstrate the claim) | r3-structure.md §T-Omni-Shape-B | ⏳ R3 |
| **Target-level cost complexity composes with .dag-level CX** | R3 | T-CostLens-Composition (R3 lane 10; Director-locked 2026-04-28; owned by Substrate Manager (post-R2 continuation) per the row above) — exact same composition fold as the "Coercion cost = complexity" row above; structural composition of `.dag` algebra-level cost + target-primitive realization cost via the language spec | Per [`docs/design-emission-model.md`](../design-emission-model.md) Modeling problem 8 — the unification "coercion cost = complexity" IS the structural composition of `.dag` CX with target-level cost; both rows resolve to the same R3 lane | ⏳ R3 dispatch (gated on R2-Evaluator + **R2-T-Substrate-Lens-Primitive** + R2-T-Substrate per-op algebra cost + R2-T-Ground-LanguageSpec per-primitive realization cost) |

## Disposition table — Self-hosting (4 facets)

Per THESIS §"Self-hosting — four facets" (Facet 4 added 2026-05-04 per Director ratification — committed structurally in THESIS.md alongside this disposition row):

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Facet 1: Compiler written in `.dag`** (partial today; full at SG-0=0) | R1 + R2 + R3 | T-PB-A SG-0 census reduction (R1 + R2) → 0 (R3) | 70 → 37+1 frag (R2 progress); 0 (R3 close via T-LensProducer-Retirement) | 🟡 R1+R2 partial / R3 close |
| **Facet 2: Compiler self-emits (fixed-point) — bit-identical output** | R3 | T-FixedPoint / `pb_self_compile_fixed_point` | r3-structure.md §T-FixedPoint | ⏳ R3 (gated on R2-Evaluator + T-LensProducer-Retirement) |
| **Facet 3: Tests are data — pipeline.rs equivalent ports to .dag** | R1 + R2 | T-PB-B (bulk migration of class-5 tests via ExecuteCommand) | R2 helper binary #1063 ✓; T-PB-B 2A/2B about to unblock post-#1049 | 🟡 R2 in flight |
| **Facet 4: Recursive-flex / self-application** (NEW 2026-05-04) — gunbc applies its own correctness/cost/parallelism/timing lenses to its own build pipeline. The compiler that compiles gunbc programs validates the workflow that produces gunbc itself. | R3 | T-Workflow-As-Data + T-Lens-Self-Application | THESIS.md §"Self-hosting — four facets" Facet 4; r3-structure.md §T-Workflow-As-Data + §T-Lens-Self-Application; Director ratification at [gunbc#828 inbox-4374342708](https://github.com/gunb-ai/gunbc/issues/828); Substrate Mgr design stance at [gunbc#1130 comment-4374109666](https://github.com/gunb-ai/gunbc/issues/1130#issuecomment-4374109666) | ⏳ R3 dispatch (gated on T-Lens-Behavioral-Parity COMPLETE + T-Lens-Application-Surface + R2-Evaluator) |

## Disposition table — Enumerable impossible-bug classes

Per THESIS §"Enumerable impossible-bug classes":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **R1 class: suboptimal-complexity contract violation** | R1 | T-LaneE complexity lens | E-family carrier port; live | 🟡 R1 closure |
| **R1 class: idempotency-contract violation** | R1 | T-LaneE idempotency lens | Live per lens capability register | ✅ live |
| **R1 class: transport/type drift** | R1 | T-Emit multi-target output | T-Emit lane | 🟡 R1 closure |
| **R2+ class: nested-optional flatten** | R2 | T-ImpossibleBugs Class 1 | PR #890 + PR #962 | ✅ landed in R2 |
| **R2+ class: unenumerated effects** | R2 | T-ImpossibleBugs Class 3 | PR #971 | ✅ landed in R2 |
| **R2+ class: unhandled diagnostic paths** | R2 | T-ImpossibleBugs Class 2 | PR #969 iterating CI | 🟡 in flight |

## Disposition table — Meta-process modeling

Per THESIS §"Meta-process modeling":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Build orchestration modeled as .dag workflows** | R3 | T-Workflow-As-Data (R3 Lane #17) | `docs/r3-structure.md` §T-Workflow-As-Data; THESIS:224 added "build orchestration" via PR #1738 | ⏳ R3 dispatch |

**Note:** Bootstrap / CI / dev process modeling (other items in the THESIS Meta-process bullet at line 224) are tracked via T-PB program / extdeps emission / ROADMAP entries respectively; not expanded into separate rows here pending future scope extension. Build orchestration is broken out because PR #1738's SHIP_WITH_DEBT meta-review (gpt-5-5-pro 2026-05-05) flagged it as needing accounting parity to a named consumer (T-Workflow-As-Data lane #17 is that consumer).

## Disposition table — Modeling discipline

Per THESIS §"Modeling discipline":

| Claim | Disposition | Lane / gate | Evidence | Status |
|---|---|---|---|---|
| **Every declared type has at least one structural consumer** | R1 + R2 | INVARIANTS P2 (Boundary Discipline) + E-6 (no target-spec field without same-PR consumer) | continuous discipline | ✅ live (continuous discipline) |
| **Every service boundary uses typed enums, not String/Bool proxies** | R2 | T-Substrate B-wave + T-Modeling | SourceFiltering canonical authority (#1004) + Dimensions (#886) | ✅ landed in R2 |
| **No fabrication sentinels** | R2 | INVARIANTS P3 (Fail-Closed) + B-series fixes | PR #817/#820/#821; Worker 1 cost/dimension dissolution in flight | 🟡 R2 in flight |
| **No duplicate record shapes** | R2 + R3 | T-Substrate (R2 closures) + T-Tier3-Dissolution (R3 mirror retirement) | R2 SourceFiltering closure; R3 mirror dissolution | 🟡 R2 partial / R3 close |

## Compromises summary

Compromises being made by the R2 + R3 split:

### What R2 explicitly defers to R3

| Item | Why | R3 lane |
|---|---|---|
| Tier 3 mirror dissolution (4 mirrors) | Requires Evaluator (R2 capacity) to execute std bodies | T-Tier3-Dissolution |
| Lens-producer file retirement (3 files) | Requires Evaluator + PB-1 generated bin-shim emit pattern | T-LensProducer-Retirement |
| R3 verification harness ({L4, L5, L7}; L6 reclassified to R2) | Requires Evaluator + full Grounding (Rust + Python) | T-Verification-L4-L7-Direct + T-Verification-L5-Corpus (L6 lives in R2-T-Ground-CrossTarget-Meta) |
| Self-hosting facet 2 fixed-point | Requires SG-0 = 0 (T-LensProducer-Retirement first) | T-FixedPoint |
| Tier 2 Int128/Word128 substrate (subsumed by T-Numeric-Construction reframe 2026-05-01) | Reframed as one refinement (`Int<128>`) consuming abstract `Int = AbelianGroup<Nat>` per construction chain | T-Numeric-Construction (absorbs T-Int128 + post-R3 BigInt + Float widening + UInt widening + IntLit refinement) |
| Shape B omni-emission demos (≥2) | Needs Evaluator + Shape B emitter `.dag` programs | T-Omni-Shape-B |
| Anthropic typed wire | Held in R2 pending OpenAI #1028 stabilization | T-Anthropic-Wire |

### What stays post-R3 (not in any release)

| Item | Why |
|---|---|
| Practical thesis pressure-test on real programs (`../ctrl/`) | Per [`docs/r2-structure.md`](../r2-structure.md) §"Decisions locked": pressure-test validates whether the structural thesis holds in practice; not itself a thesis claim |
| Adoption tooling, ecosystem, community | Not a thesis claim; downstream of structural close |
| v2 retirement | Operational cleanup; bounded by adoption concerns, not release-gate discipline |
| Shape A target saturation (TypeScript / Swift / HDL) | THESIS claim is `O(1)` per target — Rust + Python + Go proves the structural claim; saturation is adoption-driven |
| Shape B target saturation (full Terraform / SPICE / SQL DDL coverage) | Same as Shape A: ≥2 demos operationalize the structural claim; saturation is ecosystem buildout |
| Tier 1 type-refinement features beyond R2 modeling | If new modeling capabilities surface, they're substrate additions, not thesis-required |
| Phantom-parameter typed value wrappers (`Duration<Unit>`, `Money<Currency>`) | Per ROADMAP `:450`: the live tree does not yet support this shape. `Dimension<Carrier>` (PR #886) is a **one-parameter proof-dimension framework**, not a phantom-parameter value type. Dissolution requires substrate support for typed value wrappers with phantom parameters propagating through arithmetic + algebra inhabitance (abelian group with compare). Adjacent to DB-18 but a distinct modeling capability. Owner: unassigned; M scope when prioritized |

### Indirect / implicit claims (no dedicated lane)

*No items currently — see notes below.*

**Note on free consequences:** the prior "Free consequences (auto-parallelism, auto-memoization, cross-language optimization) — no dedicated lane" framing was retracted via PR #1738 follow-up: those three free consequences DO have dedicated R3 lanes per the §"Disposition table — Free consequences" above (T-Free-Consequences-Demonstration / `auto_parallelism_*` + `auto_memoization_*` + `cross_target_optimization_*` gates) following the 2026-04-30 R3 expansion (operationalized as Lane 3 demonstration deliverables per the prior **Note on "free consequences"** that immediately follows the §"Disposition table — Free consequences" above). Single-authority resolved by deleting the conflicting row — the disposition table is the single source. Same drift pattern as the 2026-04-28 concept-unifications retraction below.

**Note on concept unifications:** every concept unification listed in THESIS §"Concept unifications" *does* now have a dedicated lane — see the §"Disposition table — Concept unifications" above. ("Coercion cost = complexity" → T-CostLens-Composition; "Coercion = emission" → T-Ground-Dissolve + T-Verification-L4-L7-Direct; "Target language spec = transport spec = interpreter runtime" → T-Verification-L5-Corpus; "Idempotency + cancellation + redundancy = algebraic simplification" → T-ImpossibleBugs Class 3.) The prior "no dedicated lane" framing for concept unifications was retracted 2026-04-28 per codex BLOCKING finding on `c98981634`: it split the release-control fact, since the disposition table assigned dedicated lanes while this summary said "no dedicated lane". Single-authority resolved by deleting the row — the disposition table is the single source.

## Net read

**At R2-close, the capacity layer of the thesis is structurally complete:**
- Substrate carriers stable (no C1 stop signal)
- Evaluator runtime executes `.dag` bodies, applies lenses, constructs witnesses
- Three target families (Rust + Python + Go) structurally grounded
- 6 of 6 enumerable impossible-bug classes structurally caught (3 R1 + 3 R2+)
- Modeling discipline applied (single authority, no fabrication, typed carriers)

**At R3-close, the consequence layer falls out:**
- Tier 3 mirrors dissolved (mirror bodies replaced by Evaluator-backed authority inside `dag.rs` / `dag/effects.rs` / `workflow_idempotency.rs`; consumer/mirror-symbol count reaches zero); **SG-0 reaches 0 via T-LensProducer-Retirement + broader PB-Substrate / generated-file retirement, not as a direct Tier 3 SG-0 consequence** (Tier 3 SG-0 delta is usually 0 because the hand-authored file remains on the census after mirror-block retirement — per PB Manager review 2026-04-28); fixed-point self-hosting → facet 2 closes
- R3 verification harness ({L4, L5, L7}) proves emit/eval match + cross-target consistency + algebraic laws. L6 (structural form coverage) was reclassified to R2-T-Ground-CrossTarget-Meta as a structural cross-product fold; not part of the R3 runtime harness
- Tier 2 fully extends to Int128/Word128
- Omni-emission demonstrated end-to-end (≥2 Shape B targets)
- Provider parity (Anthropic + OpenAI typed wires)

**At post-R3 (external):**
- Practical thesis pressure-test on real programs (`../ctrl/` modeling)
- Ecosystem adoption (TypeScript, Swift, HDL, additional Shape B targets)
- v2 retirement
- Community + documentation

**The thesis is structurally demonstrated by R3-close.** Practical demonstration (whether the structural claims survive contact with real programs at adoption scale) is post-R3 work — and per existing decision, that's external to the release ledger.

## Cross-refs

- Parent: [`THESIS.md`](../../THESIS.md) §"Thesis claims — complete list" — single-source on which claims exist
- Sibling: [`docs/r2-structure.md`](../r2-structure.md) — R2 program structure (capacity layer)
- Sibling: [`docs/r3-structure.md`](../r3-structure.md) — R3 program structure (consequence layer)
- Authority: [`ROADMAP.md`](../../ROADMAP.md) — current state and lane status
- Self-hosting: [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) — 0-floor target authority
- Verification: [`docs/thesis/two-groundings-static-validation-vs-efficient-realization.md`](two-groundings-static-validation-vs-efficient-realization.md) — verification framing
- Grounding: [`docs/thesis/target-grounding-proposal.md`](target-grounding-proposal.md) — grounding work breakdown
