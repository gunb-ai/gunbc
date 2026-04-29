# R3 Structure — Thesis Closure / Consequence Cycle

**Status:** `PROPOSAL` — pending R2 promotion + cascade alignment with [`docs/r2-structure.md`](r2-structure.md) Evaluator extension.

**Authority:** single-source while open. Amendments before promotion land in this doc. After promotion, amendments follow the same discipline as R1's `## Release R1 Program` section (director-authored PRs with manager acknowledgement).

**Supersedes:** [`docs/r2-structure.md`](r2-structure.md) §"Program count" framing of R3 as *escape hatch only*. R3 is now a structured **Thesis Closure** program — the consequence cycle running every thesis claim that becomes mechanical after R2 lands the Evaluator + complete Grounding.

## Frame

R2 closes the **capacity layer** of the thesis: substrate carriers, the Evaluator runtime, full target Grounding for Rust + Python + Go, and 6 enumerable impossible-bug classes structurally caught.

R3 closes the **consequence layer**: every thesis claim that *falls out* once the capacity layer exists. Tier 3 mirror dissolution, lens-producer retirement, R3 verification harness for {L4, L5, L7} (L6 reclassified to R2-T-Ground-CrossTarget-Meta as a structural cross-product fold), self-hosting facet 2 fixed-point, Tier 2 Int128/Word128 substrate, omni-emission Shape B demos, and Anthropic typed wire.

The split between R2 and R3 is structural, not arbitrary:
- **R2 = enabling work** (substrate + runtime + grounding) — design questions, novel substrate, multi-month critical paths under solo-dev sizing
- **R3 = mechanical consequences** (mirror dissolution, harness construction, scaffolding deletion) — work that becomes obvious once the Evaluator exists

The R2 vs R3 boundary is therefore: **does this work need new substrate / new runtime / new design, or does it follow mechanically from substrate + runtime that R2 already lands?**

## Summary

R3 has ten lanes (revised 2026-04-28 per Director review of #1078; T-CostLens-Composition added 2026-04-28 per user direction folding cost-lens-over-emission into R3), each closing a specific thesis claim or claim-cluster:

1. **T-Tier3-Dissolution** — retire the four hand-Rust mirrors of `.dag` types (termination, computation, induction, effect-carrier) by consuming the Evaluator
2. **T-LensProducer-Retirement** — retire `lens_apply.rs`, `lens_testgen.rs`, `regen_lens.rs` (the program-sized hand-Rust files) via PB-Runtime interpreter-as-data + PB-1 generated bin-shim emit pattern
3. **T-Verification-L4-L7-Direct** — Evaluator-direct verification harness: L4 emit/eval match + L7 algebraic-law witnesses. Can start as soon as Evaluator + R2 close. **Also serves as the structural test of the no-engine discipline** per [`docs/design-emission-model.md`](design-emission-model.md): L4 fails if the fold fabricates target choices `.dag` doesn't evaluate to; L7 fails if algebra inhabitance is engine-asserted vs structurally declared
4. **T-Verification-L5-Corpus** — corpus-driven verification: L5 cross-target consistency only. Depends on (a) all 3 Shape A targets grounded and (b) L4 corpus existing first. **Also tests no-engine discipline**: L5 fails if engine policy resolves inconsistently across targets. (L6 structural-form coverage was moved out of this lane: it's a structural cross-product fold over substrate × language-specs, checkable at compile time with no corpus or runtime; it now lives in R2's T-Ground-CrossTarget-Meta lane scope per `docs/design-emission-model.md` engine-reframe correction.)
5. **T-FixedPoint** — self-hosting facet 2: compile `compiler.dag` → bit-identical Rust output
6. **T-Int128** — Tier 2 Int128/Word128 substrate (the int-lit closure half deferred from R2)
7. **T-Omni-Shape-B** — at least 2 Shape B omni-emission demos exercising the "one workflow → full-stack artifacts" thesis claim. **Director-locked 2026-04-28**: primary pair = OpenAPI spec + Markdown drift-lock; SQL DDL is the alternative if OpenAPI runs into design surface issues. Other candidates (YAML/K8s, Terraform, SPICE, etc.) are post-R3 ecosystem work, not R3 demos
8. **T-Anthropic-Wire** — typed wire schema for Anthropic provider (held in R2 pending OpenAI #1028 stabilization)
9. **T-Bridge-Retirement** — unified ledger of named identity bridges retired (`SourceSpan.file` participation checks, `mark_bootstrap_secret_nominal_opacity()`, canonical lens-name dispatch, `include_str!` side channels, exact-string patching residual). Surfaced by Reflective Pattern B (2026-04-25 analysis); without a unified lane these get scattered across PB / Substrate / Verification work without a unified retirement ledger
10. **T-CostLens-Composition** — cost lens reads (1) `.dag` algebra-level cost + (2) target-primitive realization cost via the language spec; composes structurally; verifies the THESIS unification "**coercion cost = complexity**" holds **by construction** (not just by reviewer convention). **No "coercion cost" dimension** — falls out of the existing complexity lens reading substrate facts. Per Modeling problem 8 in [`docs/design-emission-model.md`](design-emission-model.md). Director-locked 2026-04-28 to land in R3 (deferring would leave the thesis unification asserted-not-structural)

**7 of 10 R3 lanes are gated on R2-Evaluator closing** (T-Tier3-Dissolution, T-LensProducer-Retirement, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition). The other 3 (T-Int128, T-Anthropic-Wire, T-Bridge-Retirement) are self-contained or substrate-completion work parallel to the Evaluator-gated lanes — they consume R2 substrate carriers but not the Evaluator itself, so they can dispatch in parallel with R2-Evaluator work or wait until R2-close per scheduling preference. Per-lane R2-close dependency is named in the §"Lane structure" table below; §"Dependency on R2" elaborates.

## Acceptance — `.dag` gates

Each lane owns one or more concrete `.dag` `TestClaim` gates. Authored as deliverables of the lane-brief drafting step (lane owners author them as `.dag` after dispatch).

- **T-Tier3-Dissolution.**
  - `tier3_termination_mirror_dissolved` — `dsl/std/termination.dag` is the only authority; `src/v3/compiler/src/dag.rs` carries no parallel mirror of `DescentEvidence` lattice operations
  - `tier3_computation_mirror_dissolved` — same shape for `std.computation` (`ShrinkFactor`, `IterationPrimitive`, `kernel_algebra_profile`)
  - `tier3_induction_mirror_dissolved` — same shape for `std.induction` (`SubValueRelation`, E-P evidence)
  - `tier3_effect_carrier_mirror_dissolved` — `workflow_idempotency.rs` retired; crate API consumes emitted/evaluated `std.effects` as sole authority
- **T-LensProducer-Retirement.**
  - `lens_apply_dot_rs_retired` — `src/v3/compiler/src/lens_apply.rs` deleted; lens application routes through PB-Runtime interpreter-as-data
  - `lens_testgen_dot_rs_retired` — `src/v3/compiler/src/lens_testgen.rs` deleted
  - `regen_lens_dot_rs_retired` — `src/v3/compiler/src/bin/regen_lens.rs` deleted
  - `sg0_non_test_zero` — SG-0 `EXPECTED_HAND_AUTHORED_NON_TEST` count reaches 0 per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md)
- **T-Verification-L4-L7-Direct** (Evaluator-direct).
  - `l4_emit_eval_match` — for every `.dag` program in the certification corpus, emitted target output equals `.dag` evaluation output (algebraic equality, not byte-equal)
  - `l7_algebraic_laws_witnessed` — every algebra declared in `dsl/std/algebra.dag` has a runtime-constructed witness for each of its laws (associativity, commutativity, identity, distributivity as applicable) — `AlgebraicLaw` TestPredicate evaluates via Evaluator-constructed witnesses, not host-mediated harness
- **T-Verification-L5-Corpus** (corpus-driven; depends on Direct landing first).
  - `l5_cross_target_consistency` — for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus

L6 (`l6_structural_form_coverage`) was moved out of this lane during the engine-reframe correction (2026-04-28): the property "every Tier-1 structural form (each of the 6 type connectives × each of the 5 behaviors × every cardinality variant) emits to every Shape A target" is a **structural cross-product fold** — checkable at compile time by walking `substrate × language-specs` and verifying each pair has an emission path declared. It does not need a corpus or runtime; classifying it as "corpus-driven verification" let runtime authority gate a structurally-checkable property (same pattern as the omni-coherence finding). L6 now lives in **R2's T-Ground-CrossTarget-Meta lane** per [`docs/design-emission-model.md`](design-emission-model.md) engine-reframe correction.
- **T-FixedPoint.**
  - `pb_self_compile_fixed_point` (R1 gate; closes here under stronger interpretation) — v3 binary compiles `compiler.dag` and produces bit-identical stage0 Rust + bit-identical emitted artifacts. Predicate is the same R1 predicate; R3 closes it under fixed-point semantics rather than the looser "compiler can compile itself" reading
  - **Two-horizon clarification (per R1 Closure Manager review 2026-04-28):** the predicate name `pb_self_compile_fixed_point` carries TWO horizons: (i) **R1 lane gate** — Pass = current `verification.dag` + `test_runner` evaluation under the R1 acceptance discipline; this is what `#1050` + `#1074` made green at landing. (ii) **R3 thesis facet 2** — closes under the stronger interpretation (bit-identical fixed-point + SG-0 choreography per Director decisions). **R1 close does NOT wait on R3.** R1's gate Pass is whatever the current predicate evaluates to; R3's elevated bar is a separate release/thesis acceptance, not a silent rename of the R1 predicate. See `r2-structure.md` §"R1 closure criteria" for the R1-side framing.
- **T-Int128.**
  - `tier2_int128_overflow_proven` — the runtime-safety claim for integer overflow extends from i64-bounded (R2 close) to Int128/Word128 substrate; no `IntLit` magnitude exceeds carrier without compile-time rejection
  - `int_lit_full_int128_word128_consumer` — int-literal magnitude consumer covers the full range, not just i64-bounded
- **T-Omni-Shape-B** (Director-locked target pair 2026-04-28: OpenAPI + Markdown drift-lock primary; SQL DDL alternative).
  - `omni_openapi_backend_emission_demo` — one workflow `.dag` declaration emits to (a) an OpenAPI spec describing the workflow's external API + (b) a runnable backend service implementing it. Both derive from the same `compile_to_dag` result
  - `omni_documentation_drift_lock_demo` — same workflow `.dag` emits to a Markdown documentation artifact that drift-locks against the implementation (cannot describe behavior the implementation doesn't have, by construction)
  - `omni_sql_ddl_alternative_demo` — alternative gate triggered ONLY if `omni_openapi_backend_emission_demo` hits design-surface issues that defer it; same workflow emits to a SQL DDL schema + the backend that implements it. Replaces the OpenAPI gate per the locked alternative; not a third gate
  - `omni_layers_share_one_node_tree` — **structural coherence gate** per THESIS:213: for each demo workflow, every emitted layer (Shape A backend + Shape B API spec + Shape B documentation, per the OpenAPI + Markdown lock) derives from the same `compile_to_dag` result. Structurally checkable at compile time: per-workflow count of `compile_to_dag` invocations = 1; all emitters consume the same `Dag` value via the typed substrate query surface. **This is the structural acceptance predicate for "coherence between layers is structural, not checked"** — the property holds by construction (same Node tree); the gate verifies the demos satisfy that construction. Distinct from L4/L5 which are runtime equivalence checks
  - Counts as 2 Shape B targets per THESIS §"Omni-emission" `O(1)` per Shape B target claim
- **T-Anthropic-Wire.**
  - `anthropic_wire_typed_serde_alignment` — Anthropic provider request/response types are typed end-to-end (mirrors the OpenAI alignment landed in R2)
  - `anthropic_unit_enum_role_serialization_correct` — role enum serializes to wire-required strings without bridging
- **T-CostLens-Composition.**
  - `cost_lens_reads_target_realization` — for every emitted target program, the cost lens reads (a) the program's `.dag` algebra-level cost and (b) the target language spec's per-primitive realization cost; composition is structural fold, not engine policy
  - `coercion_cost_equals_complexity_by_construction` — for the certification corpus, applying the cost lens to a program's *emitted* target produces the same total cost as decomposing it into algebra-level cost + per-primitive realization cost. Verifies the THESIS unification "coercion cost = complexity" holds structurally
  - `no_coercion_cost_dimension` — there is no separate "coercion cost" dimension or carrier in the substrate; cost queries route through the existing complexity lens
- **T-Bridge-Retirement.**
  - `bridge_source_span_file_participation_retired` — no production code path consults `SourceSpan.file` for participation/inclusion logic; participation is structural per declared facts
  - `bridge_mark_bootstrap_secret_nominal_opacity_retired` — name-keyed bootstrap bridge from #937 deleted; nominal-opacity authority lives in source-level declaration (PR A landed in R2)
  - `bridge_canonical_lens_name_dispatch_retired` — lens dispatch routes via `DeclarationRef`/typed identity, not canonical name strings
  - `bridge_include_str_side_channels_retired` — no `include_str!` macro reads source-substrate identity; substrate query surface used instead. **Open disposition (`pipeline_authority`, PR #1171, 2026-04-29):** `compile` remains `ArrowBody::Unparsed`, so compile-body stage order is not yet a structural Dag fact; runtime ordering reads `PipelineStageBinding` only — full gate for this site awaits derivation / lowered compile witness, not file IO.
  - `bridge_exact_string_patching_residual_retired` — `patch_lower_helpers_*` and similar exact-string patching scaffolds reach 0 residual (some retired in R2 #1014)
  - `bridge_retirement_ledger_zero` — unified ledger reports 0 named identity bridges remaining

## Lane structure

| Lane | Size | Manager | Covers | R2-close dependency |
|---|---|---|---|---|
| **T-Tier3-Dissolution** | M | **Tier 3 Manager** (or PB Manager continuing post-R2) | Four hand-Rust mirrors of `.dag` types retired (mirror bodies replaced by Evaluator-backed authority inside `dag.rs` / `dag/effects.rs` / `workflow_idempotency.rs`); **consumer count / mirror-symbol count reaches zero**. SG-0 delta is reported and **usually 0** because the hand-authored file remains on the census after mirror-block retirement — SG-0 reaches 0 through broader PB-Substrate / generated-file retirement + T-LensProducer-Retirement, not as a direct Tier 3 consequence (per PB Manager review 2026-04-28) | R2-Evaluator (executes std bodies); ValueBody::Map carrier (landed in R2 post-#1017; map read-path/API + arrow-body evaluation are the remaining substrate gaps for `kernel_algebra_profile`) |
| **T-LensProducer-Retirement** | XL | **PB Manager (post-R2 continuation)** | Three program-sized hand-Rust files retired via PB-Runtime + PB-1 patterns. **Internal sub-gates** (per Director directive 2026-04-28 — XL framing kept; sub-gate visibility for closure-ledger reporting): (i) `lens_apply.rs` retired (gated on PB-Runtime interpreter-as-data); (ii) `lens_testgen.rs` retired (same gate as `lens_apply.rs`); (iii) `regen_lens.rs` retired (gated on PB-1 bin-shim emit pattern — distinct gate). Closure ledger reports sub-gate progress so PB Manager can report sub-progress, but the lane is one program. **Plus advanced lifetime analyzer cases d/e/f** (closures, async lifetimes, self-referential/Pin) folded into this lane per design-emission-model.md Open call 2 — the lifetime analyzer is structurally what replaces `lens_apply.rs`'s reflection work, so advanced cases land alongside retirement | R2-Evaluator (interpreter-as-data); PB-1 generated bin-shim pattern (which itself depends on Evaluator); R2-T-Ground-Lifetime-Analyzer (a/b/c basic cases) |
| **T-Verification-L4-L7-Direct** | M | **Verification Manager** (new) | L4 emit/eval match harness + L7 algebraic-law witness construction. Evaluator-direct; can start as soon as R2-Evaluator + R2 close. **NOT a `Lens<C>` instance** (per codex BLOCKING `f5f63c7d9` — `Lens<C>.read: (Dag, Behavior) → Witness<C>` cannot read emitted target artifacts; L4/L7 are *runtime equivalence checks* that compare two computational results — emit-target output vs .dag eval result). **Consumes** `Lens<C>` instances as inputs where useful (e.g., `Lens<SymbolicCost>` for cost-related claims), but the lane itself is a corpus-driven runtime harness, not a structural fold. **Note:** L6's structural completeness check (in R2-T-Ground-CrossTarget-Meta) is also NOT a Lens<C> instance — different input space (per-(form × target) vs per-Behavior); L6 lives as its own substrate-load-time completeness primitive per codex BLOCKING `90220bd97`. | R2-Evaluator (witness construction) + **R2-T-Substrate-Lens-Primitive** (consumed as input substrate, not as L4-L7 framing) |
| **T-Verification-L5-Corpus** | M | **Verification Manager** | L5 cross-target equivalence only. Corpus-driven; needs (a) all 3 Shape A targets grounded, (b) L4 corpus from T-Verification-L4-L7-Direct existing first. (L6 form coverage moved to R2-T-Ground-CrossTarget-Meta as a structural cross-product fold; see §"Acceptance" note.) | R2-Grounding-Rust + R2-Grounding-Python + T-Verification-L4-L7-Direct |
| **T-FixedPoint** | M | **PB Manager** | `compiler.dag` compiles to bit-identical stage0 Rust + bit-identical emitted artifacts; R1's `pb_self_compile_fixed_point` gate closes under stronger interpretation | R2-Evaluator (executes compiler.dag); SG-0 zero from T-LensProducer-Retirement |
| **T-Int128** | M-L | **Substrate Manager (post-R2 continuation)** | Int128/Word128 substrate; int-literal full magnitude consumer | None (parallel to T-Tier3 + T-LensProducer; just substrate work) |
| **T-Omni-Shape-B** | L | **Demo Manager** (or R3 Release Manager) | At least 2 Shape B omni-emission demos exercising the full-stack thesis claim | R2-Evaluator (Shape B emitters are `.dag` programs walking typed values via fold/match — needs runtime to demonstrate properly) |
| **T-Anthropic-Wire** | M | **Substrate Manager (post-R2 continuation)** | Anthropic provider request/response typed end-to-end | None (parallel; held in R2 pending OpenAI stabilize) |
| **T-Bridge-Retirement** | M | **Verification (ledger only); retirement work distributed per bridge map** (Director-locked 2026-04-28: distribute-work-centralize-ledger discipline match — bridges retire in PB/Substrate territory; Verification owns the unified `bridge_retirement_ledger_zero` audit gate) | **Bridge distribution map** (5 named bridges): (1) `SourceSpan.file` participation checks → **Substrate** (typed identity surface); (2) `mark_bootstrap_secret_nominal_opacity()` → **Substrate** (Secret PR A continuation lineage); (3) canonical lens-name dispatch → **PB Manager** (lens-producer-retirement adjacent); (4) `include_str!` side channels (e.g., pipeline_authority.rs) → **PB Manager** (compiler-internal bootstrap); (5) `patch_lower_helpers_*` residual → **PB Manager** (Tier 2 retirement lineage; #1014 was first slice). **Net: 3 Substrate-owned + 3 PB-owned + 1 Verification-owned ledger.** Verification's `bridge_retirement_ledger_zero` audit gate verifies cross-program coordination/reporting cadence; the actual retirement work absorbs into existing Substrate / PB scopes without spawning a parallel manager | R2 substrate carriers (typed identity surfaces); per-bridge gates depend on the natural-owner program's prerequisites |
| **T-CostLens-Composition** | M | **Substrate Manager (post-R2 continuation)** (Director-locked 2026-04-28: substrate-shape match — T-CostLens-Composition is substrate-authoring of cost facts (per-op algebra cost + per-primitive realization cost) + Lens<SymbolicCost> instance demonstration. Substrate authors; Verification asserts the gate. Different concerns shouldn't fold into one manager.) | Cost lens composes `.dag` algebra-level cost + target-primitive realization cost via the language spec; structural fold, not engine policy. **Instance of `Lens<C>`** (from R2-T-Substrate-Lens-Primitive) with `C = SymbolicCost`. Verifies "coercion cost = complexity" holds by construction. No "coercion cost" dimension. Per Modeling problem 8 in [`docs/design-emission-model.md`](design-emission-model.md). | R2-Evaluator (witness construction for cost claims) + **R2-T-Substrate-Lens-Primitive (the `Lens<C>` shape)** + R2-T-Substrate (per-operation cost on every algebra) + R2-T-Ground-LanguageSpec (per-primitive realization-cost declarations) |

Critical path: **T-Verification-L4-L7-Direct → T-Verification-L5-Corpus** is the longest because Direct's corpus seeds Corpus's coverage suite. Other lanes parallel-dispatch after R2-Evaluator closes.

## Manager structure

R3 inherits R2's manager structure with three modifications:

1. **R2 managers continue post-R2-close** rather than dissolving. Substrate Manager continues with **T-Int128 + T-Anthropic-Wire + T-CostLens-Composition** (3 lanes — T-CostLens-Composition added 2026-04-28 per Director directive: substrate-shape match, Substrate authors / Verification asserts; replaces the prior Verification attribution); PB Manager continues with T-LensProducer-Retirement + T-FixedPoint + T-Tier3-Dissolution + 3 distributed bridge retirements (canonical lens-name dispatch / `include_str!` side channels / `patch_lower_helpers_*` residual — per T-Bridge-Retirement distribution map); Modeling/Impossible-Bugs Managers archive at R2 close. **Post-R2 emergent work disposition** (Director-locked 2026-04-28): if `ctrl/` pressure-test or other post-R2 work surfaces new impossible-bug classes, modeling refinements, or substrate gaps, those are absorbed by **Substrate Manager continuation** as substrate-completion work — they're evidence of substrate gaps (per closed-system principle: enumerated bug classes are exhaustive over substrate; new classes = enumeration was wrong = substrate gap to fill), not new lanes spawning new managers.
2. **Verification Manager (new)** — owns T-Verification-L4-L7-Direct + T-Verification-L5-Corpus + the `bridge_retirement_ledger_zero` audit gate of T-Bridge-Retirement (Director-locked 2026-04-28: ledger-only ownership; retirement work distributes per bridge map — see lane #9 above). Total: **2 lanes + 1 ledger gate**. Covers the **R3 verification surface {L4, L5, L7}** = three distinct runtime-verification thesis claims with shared infrastructure (the certification corpus + harness) plus cross-program bridge-retirement coordination. Why a new manager: this cluster shouldn't fold into Substrate (different concern) or PB (different concern); structural-acceptance-by-construction is its own discipline. **L6 is NOT in Verification Manager's scope** — it was reclassified out of R3 as a structural cross-product fold and lives in R2-T-Ground-CrossTarget-Meta (Grounding Manager's program). **T-CostLens-Composition is NOT in Verification Manager's scope** (Director-locked 2026-04-28: substrate-authoring of cost facts + Lens<SymbolicCost> instance — under Substrate continuation; see lane #10 above).
3. **R3 Release Manager (new, may be R2 Release Manager continuation)** — owns T-Omni-Shape-B, R3 closure ledger, R3 demo coordination. Goal-6-equivalent for R3.

Director's role unchanged: cross-program conflict resolution + scope-change escalation + weekly health check.

## Dependency DAG

```
                                  R2 close
                                     │
                                     ▼
                              R2-Evaluator landed
                                     │
       ┌────────────┬────────────┬───┴────┬────────────┬────────────┐
       │            │            │        │            │            │
       ▼            ▼            ▼        ▼            ▼            ▼
T-Tier3-Diss  T-LensProducer  T-V-L4-L7-Direct  T-FixedPoint  T-Omni-Shape-B  (T-Int128)
  (mirrors)   (3 files)        (L4+L7)           (gated on        (Shape B)    (parallel
                                                  T-LP-Retire)                  substrate)
                                  │
                                  ▼
                         T-V-L5-Corpus (L5 only; L6 moved to R2-T-Ground-CrossTarget-Meta)
                                  ▲
                                  │
                  (also gated on R2-Grounding-Rust
                   + R2-Grounding-Python landed)

                       T-Anthropic-Wire ◄── (parallel; gated on R2 OpenAI
                                            wire stabilization)
                       T-Bridge-Retirement ◄── (parallel substrate-completion;
                                              partial side-effect from
                                              T-LensProducer-Retirement)
                       T-CostLens-Composition ◄── (Evaluator-gated; also
                                                  gated on R2-T-Substrate
                                                  per-operation algebra cost
                                                  + R2-T-Ground-LanguageSpec
                                                  per-primitive realization cost)
```

**Parallel-capable work at steady state:** 7+ R3 lanes parallel-dispatchable post-R2-close. Critical path is `R2-Evaluator → T-LensProducer-Retirement → T-FixedPoint` (because fixed-point requires SG-0 = 0 which requires lens-producer retirement). Verification has its own internal critical path: `T-V-L4-L7-Direct → T-V-L5-Corpus` (because Corpus's L5 cross-target work consumes Direct's L4 corpus).

## Compromises being made

R3 commits to closing the consequence layer of the thesis. The following are *not* in R3 scope:

| Excluded | Why | Where it lives instead |
|---|---|---|
| **Practical pressure-test of thesis on real programs** | Per [`docs/r2-structure.md`](r2-structure.md), the user's `../ctrl/` modeling work is the empirical pressure-test for whether the structural thesis holds on real programs. R3 is structural close; pressure-test is post-R3 external | Post-R3 stream (per existing r2-structure.md decision) |
| **Adoption tooling, ecosystem, community** | Not a thesis claim; downstream of structural close | Post-R3 external |
| **v2 retirement** | Per r2-structure.md, v2 retirement is post-R3 operational cleanup, not on the release ledger | Post-R3 operational cleanup |
| **All Shape B targets** (full coverage) | THESIS §"Omni-emission" claims `O(1)` per Shape B *target class* — claim is structural, not "all targets ever conceived." R3 ships ≥2 demos to operationalize the claim; saturation is post-R3 work driven by adoption needs | Post-R3 ecosystem buildout |
| **TypeScript / Swift / HDL Shape A targets** | Same shape as Shape B saturation: the structural claim is `O(1)` per target; R2 ships Rust + Python + Go which proves the claim. Additional Shape A targets are adoption-driven, not thesis-required | Post-R3 ecosystem buildout |
| **Tier 1 type-refinement features beyond R2 modeling** | If new modeling capabilities surface (e.g., refined-type narrowing beyond `Secret<T>` and `Dimension<Carrier>`), they're additions to the substrate, not thesis-required | Post-R3 modeling work |

## Design challenges — direction ratified 2026-04-28; specific decisions split between DECIDED and SCHEDULED

Director review of #1078 (2026-04-28T01:32:45Z) ratified the recommendations below. **Two distinct release states** apply across the 8 challenges (per gpt-5-5-pro meta-review feedback at 03:02Z that "DECISIONS LOCKED" was conflating ratified-direction with specific-decision): [retraction-context: explaining the supersession of the prior state-name framing]

- **DECIDED** = specific design decision is final; no further PR needed for the design itself (only implementation): challenges #4 (SG-0 = 0 + ≤1 trampoline), #5 (L4-L7 split + L6 reclassified to R2), #6 (OpenAPI + Markdown drift-lock; SQL DDL alternative), #7 (perf measurable or post-R3), #8 (mechanical replication; post-R3 generalize-providers trigger)
- **DIRECTION RATIFIED, SPECIFIC DECISION SCHEDULED** = direction is locked but specific design lands in a named follow-up PR before R2-Evaluator dispatch: challenges #1 (Evaluator runtime-value model — PR-B), #2 (reflection completeness spec — PR-C), #3 (cross-target equivalence semantics — PR-D)

Before-dispatch design work is scoped as **explicit milestone PRs** rather than comment-thread resolution (see §"Pre-R2-Evaluator design lock cadence" below). The text below preserves each design challenge with its state and the cadence PR (if any) that closes it.

### 1. Evaluator runtime-value representation (R2-Evaluator scope, but R3 consumers depend on the choice)

**Question:** What's the typed runtime value model for executing `.dag` bodies?

**Sub-questions:**
- Closed-over environments: lexical or dynamic? `Loop` and `Bind` create binding scopes — does the evaluator carry environments explicitly or implicitly?
- Eager or lazy? `Loop` is bounded forward execution per P4; lazy seems compatible. Performance implications for L5 cross-target equivalence harness?
- Memoization: per-call or global? Affects how complex programs scale during L4-L7 verification
- Witness construction surface: are witnesses first-class runtime values, or constructed by a separate proof-mode evaluation pass?

**DECISION (Director-locked 2026-04-28):** Locked as Evaluator-Manager dispatch precondition; resolved via separate design PR (PR-B in cadence below). R3 consumes the outcome. R3 lanes (especially T-Verification-L4-L7-Direct) cannot start without this.

### 2. Lens reflection completeness scope

**Question:** What's "complete reflection" for `reflect_program_dag_nodes_in_file`?

Today it's shallow/lossy (per Reflective Pattern B): doesn't reflect full behavior bodies, branch arms, loop bounds, or witness structure. R3-T-LensProducer-Retirement requires complete reflection — *but* "complete" needs definition.

**Sub-questions:**
- Does complete reflection mean "every Node is reflected as a structural value"? Or "every Node is reflected via its substrate-declared accessor"?
- Loop iteration counts: structural facts or runtime facts?
- Branch arm coverage: every arm's body reflected, or only the executed arm?

**DECISION (Director-locked 2026-04-28):** Reflection completeness spec is a T-LensProducer-Retirement prerequisite. Authored as PR-C in cadence below. R3-T-LensProducer-Retirement consumes that spec.

**LOCKED 2026-04-29 → see [`docs/design-reflection-completeness.md`](design-reflection-completeness.md):** complete reflection = every substrate-declared field on every `Behavior` variant projected via the substrate-declared shape. Sub-questions resolved §5: (1) every Node reflected via substrate-declared shape — not a meaningful distinction since accessors and reflection produce the same content in different carriers; (2) loop iteration counts are *structural* (port references), not runtime (Evaluator concern); (3) every branch arm reflected — static analysis cannot pick an executed arm. No substrate-carrier change required. Gates T-LensProducer-Retirement sub-gates 1 + 2 (lens_apply.rs / lens_testgen.rs); sub-gate 3 (regen_lens.rs / bin-shim) gated separately on PB-Runtime spec.

### 3. Cross-target equivalence harness — what does "equivalent" mean?

**Question:** For L5 (`l5_cross_target_consistency`), how are emitted Rust/Python/Go programs compared?

**Sub-questions:**
- Byte-equal stdout? Algebraic-equal output values? Behavioral-equal under a chosen oracle?
- Float comparison: bit-equal or epsilon-equal? (relevant for any numeric program)
- Side-effects: how are effects normalized for comparison? Does the harness execute in isolated namespaces?
- Test corpus: who curates? How does it grow?

**DECISION (Director-locked 2026-04-28):** Algebraic equivalence over a curated corpus (not byte-equal across all programs). The L5 claim is that *semantics is invariant across targets* — that's algebraic, not lexical. L5 spec doc is PR-D in cadence below; gates T-Verification-L5-Corpus.

### 4. SG-0 zero requirement for fixed-point

**Question:** Does T-FixedPoint require SG-0 = 0 (full lens-producer retirement complete) before fixed-point semantics close, or only "non-test = 0"?

Per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md), the 0-floor target is total. But fixed-point compilation is a property of `compiler.dag` → bit-identical Rust output. There's a dependency tree:

```
T-LensProducer-Retirement (XL) → SG-0 non-test = 0 → T-FixedPoint (M)
                              ↓
                              also needed: PB-1 generated bin-shim pattern
```

**Sub-questions:**
- Does T-FixedPoint require *every* hand-Rust file retired, or just the lens-producer subset?
- Does the bin-shim pattern itself need to be expressible in `.dag`, or is the trampoline allowed to be hand-Rust under "first-time bootstrap" §`First-time bootstrap` resolution choice?

**DECISION (Director-locked 2026-04-28):** T-FixedPoint closes under "SG-0 non-test = 0 + ≤1 first-time-bootstrap trampoline allowed per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) §`First-time bootstrap`." The trampoline is *outside* `src/v3/`; the in-tree floor stays 0.

**LOCKED 2026-04-29 → see [`docs/design-pb-runtime-interpreter.md`](design-pb-runtime-interpreter.md):** PB-Runtime interpreter-as-data shape (Item 4) + PB-1 generated bin-shim emit pattern (Item 5) locked. PB-Runtime ≡ R2-Evaluator's runtime model expressed as `.dag` (not parallel; dissolution-shaped); 5-primitive constraint preserved (`Node` / `Conj` / `Disj` / `Cardinality` / `Bit`); `BinShim` substrate carrier + `.dag` emit pattern retires the bin/ class. Gates T-LensProducer-Retirement R3 sub-gates 1 + 2 (Item 4) and sub-gate 3 (Item 5). Bin-shim trampoline lives outside v3's source tree per the chosen first-time-bootstrap resolution (1/2/3).

### 5. Verification-lane sequencing (Direct vs Corpus)

**Question:** Are L4-L7 sequential or parallel?

L4 (emit/eval match) and L7 (algebraic-law witnesses) share the Evaluator; L5 (cross-target) needs all three Shape A targets grounded. L6 (form coverage) was reclassified out of R3 as a structural cross-product fold (lives in R2-T-Ground-CrossTarget-Meta).

**DECISION (Director-locked 2026-04-28; further corrected per Codex Pattern B finding 2026-04-28):** L4 + L7 land in parallel post-R2 as `T-Verification-L4-L7-Direct` (Evaluator-direct, runtime). L5 (after R2-Grounding-Rust + R2-Grounding-Python land) lands in `T-Verification-L5-Corpus`. **L6 was reclassified out of corpus-driven verification** — it's a structural cross-product fold over substrate × language-specs, checkable at compile time with no corpus or runtime; lives in R2's T-Ground-CrossTarget-Meta lane scope. The R3 verification surface is now {L4, L5, L7} = three runtime levels; L6 is a structural acceptance gate at R2.

### 6. Shape B target choice

**Question:** Which 2 Shape B targets does R3 demo?

**Candidates:** YAML/K8s, Terraform HCL, OpenAPI spec, JSON Schema, SPICE netlist, SQL DDL, Markdown documentation.

**DECISION (Director-locked 2026-04-28):** OpenAPI + Markdown drift-lock as the primary pair. Demo SQL DDL as alternative if OpenAPI runs into its own design surface (e.g., complex schema generation). Defer SPICE / HDL / niche targets to post-R3 ecosystem work.

- **Primary:** OpenAPI spec emission + Markdown documentation drift-lock — exercises both "one workflow → full-stack" and "documentation can't drift from implementation"
- **Alternative:** SQL DDL pair if OpenAPI proves too complex for R3 scope

### 7. Tier 3 mirror dissolution mechanics

**Question:** When a hand-Rust mirror is retired, what's the dissolution receipt?

The four mirrors (termination, computation, induction, effect-carrier) currently mirror `.dag` declarations. Once the Evaluator can execute `.dag` bodies, the mirrors can be deleted. But:

**Sub-questions:**
- Is consumer migration mechanical (just `use std::termination::merge_evidence` instead of the Rust mirror) or does it require API redesign?
- Performance implications: does running `.dag` bodies via Evaluator have measurable overhead vs. compiled-Rust mirrors? What's the acceptable threshold?

**DECISION (Director-locked 2026-04-28):** Either measurable as a `.dag` TestClaim (`tier3_mirror_dissolution_perf_within_budget` with explicit numeric threshold) OR explicitly post-R3 with no in-R3 perf gate. The narrative "≤2x slower acceptable" was ambiguous; the choice is between *enforced budget* and *deferred entirely*. Recommended path: **explicitly post-R3** unless someone authors the perf-budget claim with concrete numbers and tooling. R3 deliverable is structural close; perf is downstream.

### 8. R3 Anthropic vs R2 OpenAI

**Question:** How does the R3 Anthropic typed-wire lane reuse the R2 OpenAI work?

R2 #1028 lands OpenAI typed wire (held until stabilizes per current Substrate Manager direction). R3-T-Anthropic-Wire is parallel work for the Anthropic provider.

**DECISION (Director-locked 2026-04-28):** Mechanical replication of the R2 OpenAI pattern; named post-R3 *generalize-across-providers* dissolution opportunity. R3 ships Anthropic typed wire in parallel with the OpenAI shape; post-R3 work generalizes the pattern as a single provider-typing program if multiple providers warrant.

**Dissolution trigger (added 2026-04-28 per gpt-5-5-pro #1078 review):** when both R2 OpenAI typed wire (#1028) and R3 T-Anthropic-Wire have landed and stabilized, the next provider integration (or a 6-month elapsed-time check, whichever comes first) triggers the dissolution decision: **either** (a) extract the shared provider schema as a single `ProviderTypedWire<P>` substrate carrier with per-provider parameter rows in `dsl/extdeps/providers/*/`, **or** (b) add a ROADMAP row naming why provider-specific schemas remain structurally terminal (e.g., wire-format divergence beyond what a parameterized carrier can express). Without this checkable trigger, the post-R3 "dissolution opportunity" becomes a bridge that normalizes parallel authority — exactly the P5 anti-pattern.

## Pre-R2-Evaluator design lock cadence (added 2026-04-28 per Director rearrange #2)

The 8 design challenges above are not resolvable via comment-thread back-and-forth — that's ~1-2 weeks of substantive design work even at gunbc velocity. Director rearrange #2 pinned the resolution to explicit milestone PRs:

| PR | Scope | Closes | Status |
|---|---|---|---|
| **PR-A** (this PR — #1078) | r2-structure.md amendment + r3-structure.md pre-promotion + thesis-mapping + design-emission-model | Frame; lane structure; engine reframe | In review |
| **PR-B** | Evaluator runtime-value model decision (closed-over environments, lazy/eager, memoization, witness construction) | Design challenge #1 — biggest open question | Pending |
| **PR-C** | Reflection completeness spec doc (definition of "complete" for `reflect_program_dag_nodes_in_file`) | Design challenge #2 | Pending |
| **PR-D** | Cross-target equivalence semantics (algebraic-equal corpus) | Design challenge #3 | Pending |
| **PR-E** | Evaluator dispatch brief (after PR-A through PR-D land) | Worker dispatch precondition | Pending |

Workers cannot dispatch on under-specified scope, especially on multi-week T-Verification critical path. PR-B through PR-D are gates; PR-E starts Evaluator implementation work.

## Dependency on R2

R3 cannot start meaningful work until R2 closes. Specifically:

- **R2-Evaluator** is the upstream gate for **7 of 10 R3 lanes** (T-Tier3, T-LensProducer, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition). Without it, R3 dispatchers spin.
- **R2-Grounding-Rust + R2-Grounding-Python** are the upstream gate for T-Verification-L5-Corpus (specifically L5 cross-target).
- **R2 substrate carriers** (NominalOpacity, ValueBody::Map, parametric algebra) feed T-Int128 + T-Anthropic-Wire + T-Bridge-Retirement as parallel substrate-completion work.

**R3 worker dispatch precondition** (Director-locked 2026-04-28; clarified 2026-04-28 per gpt-5-5-pro BLOCKING on `dbc48dc0` re P2 single-authority discipline):

- **Applies to the 7 Evaluator-gated lanes** (T-Tier3-Dissolution, T-LensProducer-Retirement, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition): R2-Evaluator landed AND R2-Grounding-Rust+Python landed. Pre-R3 *brief authoring* may begin during R2 final week (Director-discretionary, mirroring R2's pre-R1-close pattern), but worker dispatch waits for the joint precondition. This prevents R3 brief authoring from spawning drift if R2 close definition slips.
- **Carve-out for the 3 non-Evaluator-gated lanes** (T-Int128, T-Anthropic-Wire, T-Bridge-Retirement): these are self-contained or substrate-completion work parallel to the Evaluator-gated lanes. They consume R2 substrate carriers but not the Evaluator itself, so they MAY dispatch pre-R3 (in parallel with R2-Evaluator work) per scheduling preference. The global R3 worker-dispatch precondition above applies ONLY to the 7 Evaluator-gated lanes; the 3 non-gated lanes operate as explicitly-scoped substrate-completion work outside that precondition.

This split resolves the prior single-authority drift between `:36` (non-gated lanes can dispatch in parallel) and the global precondition (worker dispatch waits): the precondition is now scoped to the 7 Evaluator-gated lanes only, with the 3 non-gated lanes explicitly carved out.

## R3 closure criteria

**All R3 gates green per `### Acceptance — .dag gates` section above.** R3 closes when each lane's TestClaim evaluates true at release.

**ROADMAP authority single source.** Once R3 promotes to ROADMAP as `## Release R3 Program`, gate semantics are owned there. r3-structure.md is the pre-promotion authority; ROADMAP is post-promotion authority.

## Decisions locked

- **R3 is the consequence cycle, not escape-hatch.** Supersedes the "escape hatch only" framing in [`docs/r2-structure.md`](r2-structure.md) §"Program count." Justification: thesis-claim mapping (Open call 1, now closed via [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md)) shows ~7 Tier-1/Tier-2/Tier-3 thesis claims become mechanical post-R2-Evaluator. Treating those as escape-hatch obscures the structural opportunity to close them as one cycle.
- **Evaluator is in R2, not R3.** The Evaluator is the *capacity* that R3 consumes. Putting Evaluator in R3 inverts the dependency and produces an R3 with no clean spin-up condition.
- **Manager continuation across R2-R3.** Substrate Manager and PB Manager continue across R2-close into R3 because their post-R2 work (Tier 3 dissolution, lens-producer retirement, fixed-point, Int128, Anthropic) is structurally homed in their existing programs. No clean-break manager dissolution at R2 close.
- **L4-L7 verification is its own program lane.** Folding L4-L7 into PB or Substrate would obscure the claim cluster. Verification Manager owns it as a dedicated standing manager.
- **Omni-emission Shape B demos count as 2 targets, not full saturation.** THESIS claim is `O(1)` per Shape B class; demonstrating ≥2 operationalizes the structural claim. Saturation is post-R3 ecosystem work.
- **Practical thesis pressure-test stays post-R3** per existing [`docs/r2-structure.md`](r2-structure.md) decision. R3 closes the structural thesis; `ctrl/` modeling validates whether it holds in practice.

## Open calls

### 1. ~~Pre-promotion design-question resolution~~ — **CLOSED 2026-04-28 per Director review of #1078**

> **🔄 CLOSED.** The 8 design challenges in §"Design challenges — direction ratified 2026-04-28; specific decisions split between DECIDED and SCHEDULED" were ratified in the Director review of #1078 (2026-04-28T01:32:45Z), with **two distinct release states** (per gpt-5-5-pro meta-review feedback at 03:02Z): **DECIDED** for challenges #4-#8 (specific design decision is final; no further design PR needed) and **DIRECTION RATIFIED, SPECIFIC DECISION SCHEDULED** for challenges #1-#3 (direction locked; specific design lands in named follow-up PRs PR-B/C/D before R2-Evaluator dispatch). The dispatch cadence is named in §"Pre-R2-Evaluator design lock cadence" (PR-A through PR-E). This open call is closed; the relocated authority is the locked-decisions section + cadence section above. New design questions surfaced after this date are tracked separately (e.g., the 4 open calls in §"Open design calls surfaced by the examples" of [`docs/design-emission-model.md`](design-emission-model.md)).

### 2. Tier 3 dissolution receipt format

When a Tier 3 mirror is retired, what's the receipt format? Today's expected pattern is:
- PR deletes the hand-Rust file
- PR adds consumer migration to the `.dag` authority
- PR body cites the `tier3_*_mirror_dissolved` gate
- SG-0 census shrinks correspondingly

**Sub-question:** does the test that verifies dissolution live in `.dag` (TestClaim evaluating the structural fact) or in Rust (regression test)? Given the 0-floor target, it must be `.dag`. What's the predicate name and shape?

**Recommendation:** `tier3_mirror_dissolved` predicate with parameters `{ mirror_name: String, std_authority: DeclarationRef, consumer_count_before: Int, consumer_count_after: Int }`. Each instantiation tests one mirror retirement.

**Ownership:** Tier 3 Manager / PB Manager (post-R2 continuation) on first retirement.

### 3. R3 demo discipline + omni-emission TDD (locked 2026-04-28 per user direction)

R2 demo discipline is "simple 'look, it runs' artifact per lane closure." R3 inherits but with additional requirement: **at least one R3 demo must exercise the omni-emission full-stack claim end-to-end** (from one workflow `.dag` to a runnable Shape A backend + Shape B API spec + Shape B documentation, per the OpenAPI + Markdown drift-lock). This is the load-bearing R3 demonstration.

**DECISION (locked 2026-04-28 per user direction):** **TDD for omni-emission** — author the test cases up-front during R3 design (before T-Omni-Shape-B implementation begins). Tests for cases-we-think-of-during-design rather than tests-after-the-fact. Concrete TDD targets to author up-front:
- **Same-Node-tree derivation test**: per-workflow count of `compile_to_dag` invocations = 1; all emitters consume the same `Dag` value (the `omni_layers_share_one_node_tree` structural acceptance gate per §"Acceptance" above)
- **Drift-lock test**: documentation artifact cannot describe behavior the implementation doesn't have (verifiable structurally — every claim in the Markdown traces to a `.dag` declaration)
- **Cross-target consistency test**: backend (Shape A) + API spec (Shape B) + documentation (Shape B) all agree on the workflow's request/response shapes (verifiable via algebraic equivalence on the Dag value all three derive from)
- **Ownership-derivation test** (per Modeling problem 3 corrected): for every program-side `String` value, ownership is derivable structurally from program use sites; no annotations required
- **Refinement-bound exact-match test**: every `Int(...)` declaration emits to a target primitive whose bound exactly matches; otherwise fail-closed with EmissionDiagnostic

**Owner:** R3 Release Manager (or Demo Manager) coordinates with T-Omni-Shape-B to land the fixtures. **Not a separate lane**; a deliverable inside T-Omni-Shape-B. **TDD discipline applies broadly** to R3 design-time test authoring (per user direction "TDD here for cases we think of during design") — applicable to T-Verification-L4-L7-Direct corpus + T-Bridge-Retirement gates + T-CostLens-Composition gates.

## Cross-refs

- Parent (release sequencing): [`docs/r2-structure.md`](r2-structure.md) — R2 program structure
- Thesis-claim mapping: [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md) — per-claim disposition table closing Open call 1 from r2-structure.md
- Self-hosting target: [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) (LIVE; 0-floor authority)
- Thesis claims: [`THESIS.md`](../THESIS.md) §"Thesis claims — complete list"; §"Self-hosting — three facets"; §"Enumerable impossible-bug classes"; §"Tests are structural data"
- Verification claims: [`docs/thesis/two-groundings-static-validation-vs-efficient-realization.md`](thesis/two-groundings-static-validation-vs-efficient-realization.md); [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md)
- Lens capability register: [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) — current lens shipped status
- INVARIANTS: [`INVARIANTS.md`](../INVARIANTS.md) §P5 "Progress Is Dissolution" — dissolution discipline applies to all R3 lanes
