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

R3 has **18 lanes + 1 standing program** (revised 2026-04-28 per Director review of #1078; T-CostLens-Composition added 2026-04-28 per user direction folding cost-lens-over-emission into R3; **expanded to 12 lanes 2026-04-30 per user directive "nothing can be deferred past R3" — added T-V2-Retirement + T-Free-Consequences-Demonstration**; **expanded to 16 lanes + 1 standing program 2026-05-02 per user directive that all "accidentally deferred" gaps absorb into R3 — added 4 new lanes (T-E-P-Producer-Broadening, T-Lens-Behavioral-Parity, T-Tests-As-Data-Completeness, T-Lens-Application-Surface) + 1 standing R3 Debt-Paydown program; T-Behavioral-Expectations-Documentation is parallel-dispatchable across existing lanes, not a separate lane (see §"Behavioral Expectations Documentation"); per Director ratification 2026-05-02 at [gunbc#828 comment 4362742638](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4362742638)**; **expanded to 18 lanes + 1 standing program 2026-05-04 per Brian directive committing recursive-flex / self-application case to R3 (gunbc applies own lenses to own build/CI workflow) — added 2 new lanes (T-Workflow-As-Data, T-Lens-Self-Application); per Director ratification 2026-05-04 at [gunbc#828 inbox-4374342708](https://github.com/gunb-ai/gunbc/issues/828)**), each closing a specific thesis claim or claim-cluster:

1. **T-Tier3-Dissolution** — retire the four hand-Rust mirrors of `.dag` types (termination, computation, induction, effect-carrier) by consuming the Evaluator
2. **T-LensProducer-Retirement** — retire `lens_apply.rs`, `lens_testgen.rs`, `regen_lens.rs` (the program-sized hand-Rust files) via PB-Runtime interpreter-as-data + PB-1 generated bin-shim emit pattern. **Scope expanded 2026-05-02**: includes ownership analysis cases d/e/f (closures + async + Pin) confirmed delivered before R3 close (not separately deferred).
3. **T-Verification-L4-L7-Direct** — Evaluator-direct verification harness: L4 emit/eval match + L7 algebraic-law witnesses. Can start as soon as Evaluator + R2 close. **Also serves as the structural test of the no-engine discipline** per [`docs/design-emission-model.md`](design-emission-model.md): L4 fails if the fold fabricates target choices `.dag` doesn't evaluate to; L7 fails if algebra inhabitance is engine-asserted vs structurally declared. **Scope expanded 2026-05-02**: per-(algebra, inhabitant, law) witness coverage exhaustive — catches the SymbolicCost product-zero bug class structurally (PR #1430 §G meta-theme #1).
4. **T-Verification-L5-Corpus** — corpus-driven verification: L5 cross-target consistency only. Depends on (a) all 3 Shape A targets grounded and (b) L4 corpus existing first. **Also tests no-engine discipline**: L5 fails if engine policy resolves inconsistently across targets. (L6 structural-form coverage was moved out of this lane: it's a structural cross-product fold over substrate × language-specs, checkable at compile time with no corpus or runtime; it now lives in R2's T-Ground-CrossTarget-Meta lane scope per `docs/design-emission-model.md` engine-reframe correction.)
5. **T-FixedPoint** — self-hosting facet 2: compile `compiler.dag` → bit-identical Rust output
6. **T-Numeric-Construction (REFRAMED 2026-05-01 from T-Int128)** — model abstract numeric concepts via construction chain (Magnitude → Nat → Int → Rational → Real) + width refinements (`Int<N>`, `Nat<N>`, `Real<N>`). **Absorbs:** T-Int128 + post-R3 BigInt deferral + Float widening + UInt widening + IntLit refinement. 13 types in scope: 3 direct (`Int`, `UInt`, `Float`) + 10 inherited via `Int` chain (`Char`, `EpochMs`, `Duration`, `Milliseconds`, `Seconds`, `RetryCount`, `HttpStatus`, `Port`, `PositiveInt`, `NonNegativeInt`). Per Director ratification 2026-05-01 ([gunbc#828 comment 4357704426](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357704426)). Design doc: [`docs/design-numeric-construction.md`](design-numeric-construction.md).
7. **T-Omni-Shape-B** — at least 2 Shape B omni-emission demos exercising the "one workflow → full-stack artifacts" thesis claim. **Director-locked 2026-04-28**: primary pair = OpenAPI spec + Markdown drift-lock; SQL DDL is the alternative if OpenAPI runs into design surface issues. Other candidates (YAML/K8s, Terraform, SPICE, etc.) are post-R3 ecosystem work, not R3 demos
8. **T-Anthropic-Wire** — typed wire schema for Anthropic provider (held in R2 pending OpenAI #1028 stabilization). **Scope expanded 2026-04-30**: includes `ProviderTypedWire<P>` carrier + per-provider parameter rows (path (a) commit per C2 ratification)
9. **T-Bridge-Retirement** — unified ledger of named identity bridges retired (`SourceSpan.file` participation checks, `mark_bootstrap_secret_nominal_opacity()`, canonical lens-name dispatch, `include_str!` side channels, exact-string patching residual). Surfaced by Reflective Pattern B (2026-04-25 analysis); without a unified lane these get scattered across PB / Substrate / Verification work without a unified retirement ledger
10. **T-CostLens-Composition** — cost lens reads (1) `.dag` algebra-level cost + (2) target-primitive realization cost via the language spec; composes structurally; verifies the THESIS unification "**coercion cost = complexity**" holds **by construction** (not just by reviewer convention). **No "coercion cost" dimension** — falls out of the existing complexity lens reading substrate facts. Per Modeling problem 8 in [`docs/design-emission-model.md`](design-emission-model.md). Director-locked 2026-04-28 to land in R3 (deferring would leave the thesis unification asserted-not-structural)
11. **T-V2-Retirement (NEW 2026-04-30)** — retire `src/v2/` (~79 .rs + ~32 .dag files); workspace member removed; bootstrap routes through PB-Runtime trampoline only. Largely consequence of T-FixedPoint + T-LensProducer-Retirement closing; pulled into R3 per user directive *"nothing can be deferred past R3."*
12. **T-Free-Consequences-Demonstration (NEW 2026-04-30)** — operationalizes thesis "free consequences" framing with `docs/design-free-consequences.md` + 10-gate TestClaim suite (auto-parallelism × 3 + auto-loop-parallelism × 3 + auto-memoization × 2 + cross-target-optimization × 2). Loop-iteration parallelism: sequential default + opt-in via `Lens<Iteration-Independence>`. Per user directive *"what guarantees does the compiler ACTUALLY provide."*
13. **T-E-P-Producer-Broadening (NEW 2026-05-02)** — broaden per-call `DescentEvidence` / `CallPattern` / `SubValueRelation` producer coverage from current first slice (recursive self-call + arithmetic-descent only) to full `ExprCall.descent_evidence` parity at live call sites. **Foundational** — affects complexity + cost lens behavioral parity. Substrate Mgr; M-L sized.
14. **T-Lens-Behavioral-Parity (NEW 2026-05-02)** — bring complexity / cost / parallelism / effect_enumeration lenses from PROXY/STUB/PARTIAL to BEHAVIORALLY COMPLETE per `docs/v3-lens-capability-register.md`. Lens consumers read per-call substrate facts (gated on T-E-P-Producer-Broadening). Includes: symbolic CostExpr full algebra (Sum/Mul/Log/Const) consumed by lens; work/span dimension split for complexity; asymptotic classification; cementing test against v2 oracle on same source; Stage 2e parallelism walk port from Rust to `.dag`; resource-threading migration for effect_enumeration. Substrate + Verification cross-program; L-XL sized. **Closure gate**: `lens_capability_register_zero_proxy_zero_stub` — register status updated to ZERO PROXY / ZERO STUB at R3 close.
15. **T-Tests-As-Data-Completeness (NEW 2026-05-02)** — close Category E test/verification surface gaps per user directive: every Rust test ports to `.dag` TestClaim or generated target-language test code (thesis facet 3); property-based testing surface (`ForAll` / `Exists` quantifiers + `ProgramGenerator` substrate carrier); cementing test discipline for `.dag` lenses. Verification Mgr; L sized. **Closure gates**: `every_rust_test_ports_to_dag_or_generated`, `forall_exists_quantifier_substrate_landed`, `program_generator_carrier_landed`.
16. **T-Lens-Application-Surface (NEW 2026-05-02; design doc landed 2026-05-02 [`docs/design-lens-application-surface.md`](design-lens-application-surface.md))** — first-class authoring surface for applying lenses to arbitrary `.dag` sections (function / module / expression / declaration scope). Per user reframe: lens application is a `.dag` declaration with configurable behavior — `apply_lens(lens, section, config)`. **Subsumes** prior T-Complexity-Contract-Compile-Error + T-User-Authored-Cost-Basis-Discipline as configurations of one mechanism. Substrate carriers (per design doc §2): **two separate top-level carriers** — `EnforcedApplication<Output, Budget>` (lens, enforcement, section, budget, severity, span) and `IntrospectApplication<Output>` (lens, section, span; no Budget axis). NOT a sum wrapping the two — v3 `.dag` substrate cannot currently express per-variant generic parameters; "SectionedLensApplication" is a collective noun for both carriers, not a sum-type declaration. Plus `SectionRef` (DeclarationScope/NodeScope disjoint sum) + `LensEnforcement<Output, Budget>` projection carrier (per-lens, e.g., complexity → AsymptoticClass projection) + `DiagnosticSeverity`. `CompileError | Warning | Silent` original user framing resolved to fail-closed-compatible binary per design doc §3 + INVARIANTS C-8. Demonstrations: complexity-contract-compile-error + CRDT cost basis + memory-peak cost basis + opt-in cross-iteration parallelism (4 worked examples; orthogonal axes per Director ratification). **Default policy for complexity contracts**: user-driven (per design doc §3.2 + §8.3 resolution at e9d67113e). Unannotated functions get synthesized `Introspect`-only applications — no implicit baseline, no inferred Enforce. Enforcement requires explicit user authoring of `apply_lens(complexity, fn, Enforce { ... })`. The original "opt-out" framing is reframed: the user can opt out (no Enforce or explicit Introspect), and compile errors fire when the user opts IN with a budget the function exceeds. `ComplexityBudgetWaiver` retains its purpose for accepting known violations of explicit user contracts (NOT an annotation per `feedback_no_annotations`). Substrate + Verification cross-program; L-XL sized. **Closure gates**: `lens_application_carrier_landed`, `section_ref_substrate_landed`, `lens_enforcement_carrier_landed` (per-lens `LensEnforcement<Output, Budget>` projection + violation-relation declarations co-located with each lens), `enforce_violation_routing_landed`, `complexity_violation_compile_error_demonstrated`, `crdt_cost_basis_demonstrated`, `memory_peak_cost_basis_demonstrated`, `opt_in_iteration_parallelism_via_lens_application_demonstrated`. Depends on T-Lens-Behavioral-Parity (lenses must be COMPLETE first).
17. **T-Workflow-As-Data (NEW 2026-05-04; per Director ratification at [gunbc#828 inbox-4374342708](https://github.com/gunb-ai/gunbc/issues/828))** — substrate work for modeling workflows as `.dag` data, including the **Shared External Attachment Pattern** (`WorkflowObservationAnchor` + observation/measurement carriers + report-not-scalar output distinguishing `Observed | Missing | Ambiguous | Stale`) per Substrate Mgr design stance at [gunbc#1130 comment-4374109666](https://github.com/gunb-ai/gunbc/issues/1130#issuecomment-4374109666). **First instance**: timing-lens substrate (`Lens<TimingMeasurement>` parallel to existing structural-static `Lens<C>` instances; observation-driven lens-shape class). **Substrate carriers**: `TimingMeasurement` + `TimingObservationSet` + `WorkflowObservationAnchor` (factored separately from timing as reusable external-data attachment primitive; serves coverage / logs / failures / artifacts beyond just timing) + `TimingBudget`. **Workflow grammar**: at least one workflow modeled as `.dag` data (CI workflow recommended as demonstration target). **Bidirectional case** (CI YAML emission + ingestion): coordinates with `gunb-ai/gunbc#1586` thread anchor 7 (workflow-timing as bidirectional architecture concern). Substrate Mgr ownership; M-L sized; absorbs into Substrate Mgr continuation per `r3-structure.md:187` standing protocol. **Closure gates**: `workflow_substrate_carriers_landed`, `timing_lens_carrier_landed` (per Substrate Mgr STOP+PING design receipt for `docs/design-timing-lens.md`), `ci_workflow_modeled_as_dag`, `shared_external_attachment_pattern_documented`. Depends on T-Lens-Behavioral-Parity COMPLETE (for lens consumption); R2-Evaluator (for runtime).
18. **T-Lens-Self-Application (NEW 2026-05-04; per Director ratification at [gunbc#828 inbox-4374342708](https://github.com/gunb-ai/gunbc/issues/828))** — demonstration work: gunbc applies its own lenses (cost / complexity / parallelism / timing) to gunbc's own build/CI workflow. Operationalizes the **recursive-flex thesis claim**: *"the compiler that compiles gunbc programs validates the workflow that produces gunbc itself."* Concrete first instance: timing-lens applied to CI workflow producing `DimensionReport<TimingMeasurement>`; either emit-back-to-CI-YAML (bidirectional case via T-Workflow-As-Data) OR direct execution. <1 min CI target as informational SLO; **not a closure gate** (per Director ratification — performance metric, not structural commitment). Verification Mgr ownership; M-L sized. **Closure gates**: `lens_self_application_demonstrated` (gunbc lenses applied to gunbc's own build/CI workflow producing `DimensionReport<C>`), `apply_lens_self_application_demonstrated` (`apply_lens(timing, ci_workflow, Enforce { budget })` enforced via existing T-Lens-Application-Surface carrier), `recursive_flex_demonstration_landed` (narrative-load-bearing claim cashes — "gunbc validates the workflows that produce gunbc"). Depends on T-Workflow-As-Data; T-Lens-Application-Surface; T-Lens-Behavioral-Parity COMPLETE; R2-Evaluator.

### NEW STANDING PROGRAM (added 2026-05-02)

**R3 Debt-Paydown Manager** — 9th standing R3 manager. Owned program: ROADMAP debt-row retirement + velocity-tripwire enforcement + closure-receipt cadence + per-PR discipline rule. Closure gate: `r3_debt_paydown_zero_remaining` — no tracked-debt rows survive R3 close. Per `feedback_standing_managers_need_owned_deliverables`: standing manager territory because Director ad-hoc would bottleneck. Hybrid mechanism: **(1)** per-PR discipline rule (every R3 PR includes "debt receipt" naming debt found + debt paid in this PR or routed to a paydown lane; vague deferrals rejected per existing INVARIANTS §P5 (Progress Is Dissolution) Dispatch-Discipline Mechanisms (b) per-PR gate — "Vague deferrals ('see ROADMAP', 'TBD', narrative without a cited row) do not satisfy the gate"); **(2)** windowed enforcement via INVARIANTS §P5 (Progress Is Dissolution) Dispatch-Discipline Mechanisms (c) Velocity tripwire (≥3:1 introduction:dissolution ratio in any 7-day window puts ad-hoc lane dispatch under Director review); **(3)** standing manager owns systemic debt that doesn't fit organic per-PR cleanup. (Outer enumeration uses (1)/(2)/(3) to avoid collision with INVARIANTS §P5 sub-mechanism labels (a)/(b)/(c) referenced inside.)

### Behavioral Expectations Documentation (parallel-dispatchable across features)

**T-Behavioral-Expectations-Documentation** — per-feature design docs covering 7 load-bearing features (per Director ratification 2026-05-02; expanded scope of 10-15 deferred to avoid template-grounding risk in saturated documentation space). Each doc: (1) names what the feature can do — concrete capabilities with examples; (2) names non-goals — what feature cannot do; (3) names guarantees — soundness/completeness/compiler-side promises; (4) confirms maintenance burden = 0 (no ongoing per-version maintenance; if it requires ongoing maintenance, that's a substrate gap); (5) lists worked examples (positive + negative cases); (6) lists test coverage (per-feature TestClaim suite). The 7 features:

1. Lens framework (foundational doc)
2-5. 4 lens instances (Complexity, Cost, Parallelism, Effect-Enumeration)
6. Complexity contract (lens application surface; compile-error default)
7. Cross-target consistency (L5)

Treated as parallel-dispatchable cross-Mgr work; any available worker can claim a feature doc. Not a separate lane (per-feature deliverables under their respective lane-owning Mgrs).

### Lane gating summary

**14 of 18 R3 lanes are gated on R2-Evaluator closing** (T-Tier3-Dissolution, T-LensProducer-Retirement, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition, T-V2-Retirement, T-Free-Consequences-Demonstration, **T-Lens-Behavioral-Parity** [Evaluator runtime needed for behavioral lens execution], **T-Lens-Application-Surface** [via T-Lens-Behavioral-Parity cascade], **T-Tests-As-Data-Completeness** [test-execution runtime needed for porting Rust tests to `.dag` TestClaim], **T-Workflow-As-Data** [via T-Lens-Behavioral-Parity cascade — workflow lens consumption needs lenses COMPLETE], **T-Lens-Self-Application** [via T-Workflow-As-Data + T-Lens-Application-Surface cascade]). The other 4 (T-Numeric-Construction, T-Anthropic-Wire, T-Bridge-Retirement, T-E-P-Producer-Broadening) are self-contained or substrate-completion work parallel to the Evaluator-gated lanes. **T-Numeric-Construction has its own internal cascade gate** (T-V2-Retirement landing first per path-(a) v2-refinement-syntax-blocker coordination). **T-Lens-Application-Surface depends on T-Lens-Behavioral-Parity** (lenses must be COMPLETE to produce useful structural facts on application sections). **T-Workflow-As-Data depends on T-Lens-Behavioral-Parity** (timing-lens carrier authoring uses the lens framework with parity-COMPLETE consumers). **T-Lens-Self-Application depends on T-Workflow-As-Data + T-Lens-Application-Surface** (demonstration consumes both substrate-extension work and the apply_lens carrier). Per-lane R2-close dependency is named in the §"Lane structure" table below; §"Dependency on R2" elaborates.

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
- **T-Numeric-Construction (REFRAMED 2026-05-01 from T-Int128).** Acceptance gates per [`docs/design-numeric-construction.md`](design-numeric-construction.md):
  - `numeric_abstract_carriers_landed` — Magnitude (terminal), Nat (Semiring<Magnitude>), Int (AbelianGroup<Nat>), Rational (Field<Int>), Real (ApproximateField<Rational>) declared in `dsl/std/integer.dag` + `dsl/std/float.dag`
  - `numeric_width_refinements_landed` — `Int<N>`, `Nat<N>`, `Real<N>` refinement chain resolves at compile time
  - `numeric_aliases_align_to_refinements` — `Int8`/.../`Int128`, `Float32`/`Float64`, `UInt8`/.../`UInt128` are refinements, not parallel substrate
  - `numeric_inherited_bake_ins_dissolved` — `Char`, `EpochMs`, `Duration`, `Milliseconds`, `Seconds` consume abstract `Int` (or appropriate refinement)
  - `int_refinement_overflow_proven_parametric` — replaces `tier2_int128_overflow_proven`; overflow caught structurally for any width refinement
  - `int_lit_full_magnitude_consumer` — replaces `int_lit_full_int128_word128_consumer`; IntLit accepts full magnitude range
  - `string_audit_receipt` — Substrate Mgr String audit landed (per Director scope-add 2026-05-01); either reframe applied OR documented-no-change
  - `numeric_reframe_no_parallel_authority` — old `Int = Int64` / `UInt = UInt64` / `Float = Float64` aliases removed; refinement chain is single authority
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
  - `bridge_source_span_file_participation_retired` — **Green predicate:** no production code path consults `SourceSpan.file` for participation/inclusion logic; participation is structural per declared facts. **Current state (R3-deferred; Director acceptance #1130 / dispatch #1139, 2026-04-29):** the gate is not satisfied; partial string-check retirement was rejected because parallel participation rules would remain. Production inclusion (distinct from diagnostics-only span use) is still keyed on path / `span.file` at: `src/v3/compiler/src/lens_apply.rs` (`behavior_source_file`, `reflect_program_dag_nodes_in_file` / `fold_lens_over_reflected_program`); `src/v3/compiler/src/lower.rs` (`lower_type_alias_refinements_phase` `dsl/std/types.dag` gate; `declaration_name_preference_rank` duplicate merge; `Dimension` + `DIMENSION_STD_AUTHORITY_FILE`); `src/v3/compiler/src/emit.rs` (`source_filtering.excludes` on declaration/bind spans). **Structural prerequisites:** module/compilation-unit identity for lens reflection; typed authority/emit-scope carriers for lower/emit; fold-shape / carrier work for remaining fold-path source-path semantics ([ROADMAP: *Lens fold execution: undeclared fallback structure + file-path semantics*](../ROADMAP.md#lens-fold-file-path-semantics)).
  - `bridge_mark_bootstrap_secret_nominal_opacity_retired` — name-keyed bootstrap bridge from #937 deleted; nominal-opacity authority lives in source-level declaration (PR A landed in R2)
  - `bridge_canonical_lens_name_dispatch_retired` — lens dispatch routes via `DeclarationRef`/typed identity, not canonical name strings
  - `bridge_include_str_side_channels_retired` — no `include_str!` macro reads source-substrate identity; substrate query surface used instead. **Open disposition (`pipeline_authority`, PR #1171, 2026-04-29):** `compile` remains `ArrowBody::Unparsed`, so compile-body stage order is not yet a structural Dag fact; runtime ordering reads `PipelineStageBinding` only — full gate for this site awaits derivation / lowered compile witness, not file IO.
  - `bridge_exact_string_patching_residual_retired` — umbrella row for exact-string patching scaffolds. **PB lower-helper slice (Tier-2 / #1014 lineage) is pinned at zero** in v3-compiler Rust: no `patch_lower_helpers*` code paths remain, and `bridge_lower_helpers_patch_zero_residual_test` ratchets reintroduction. **Other** exact-string patching classes (outside this retired lower-helper post-process bridge) remain **out of scope for this receipt** and keep their own dissolution triggers.
  - `bridge_retirement_ledger_zero` — unified ledger reports 0 named identity bridges remaining
- **T-V2-Retirement** (NEW 2026-04-30; pulled into R3 per user directive *"nothing can be deferred past R3"*).
  - `v2_oracle_no_remaining_test_consumers` — no `.rs` test file under the workspace consumes anything from `src/v2/`; v2-oracle and v2-using test scaffolds retired
  - `v2_directory_deleted` — `src/v2/` removed from the workspace; bootstrap routes through PB-Runtime trampoline only (per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) §`First-time bootstrap`); no v2 crate remains as a workspace member
- **T-Free-Consequences-Demonstration** (NEW 2026-04-30; operationalizes thesis "free consequences" framing). Loop-iteration parallelism: sequential default + opt-in via `Lens<Iteration-Independence>` (Director-ratified 2026-04-30; zero-heuristic — same shape as `Lens<Bind-Independence>`).
  - `auto_parallelism_independent_binds_emit_parallel` — `.dag` programs whose Bind sequence is provably bind-independent emit target code that schedules the binds in parallel
  - `auto_parallelism_dependent_binds_emit_sequential` — `.dag` programs with bind dependence emit serialized binds; no false parallelism
  - `auto_parallelism_branch_arms_serialize` — Branch arms are sequenced (one arm per execution); no spurious cross-arm parallelism in emitted target code
  - `auto_loop_parallelism_provable_independence_emits_parallel` — Loops carrying `Lens<Iteration-Independence>` opt-in emit parallel iteration
  - `auto_loop_parallelism_unproven_falls_back_sequential` — Loops without the opt-in lens fall back to sequential iteration; no heuristic auto-parallelization
  - `auto_loop_parallelism_dependence_emits_sequential` — Loops with provable cross-iteration dependence emit sequential iteration even if the opt-in lens is requested (lens read returns "not independent")
  - `auto_memoization_repeated_pure_call_cached` — repeated calls to a pure function with identical argument-value identity emit memoized target code (subsumes lens-fold caching as one instance)
  - `auto_memoization_no_caching_for_one_shot` — single-call sites do not emit memoization scaffolding; memoization predicates compose `Lens<Purity>·Lens<Cost>` rather than firing universally
  - `cross_target_optimization_constant_fold_consistent` — for the certification corpus, every emitted target program's `Lens<SymbolicCost>` reading post-emission equals its `Lens<SymbolicCost>` reading pre-emission minus the constant-folded subtree's algebra cost (same structural shrink applied across Rust/Python/Go via the LanguageSpec realization-cost composition; structural-fold equality, not byte/string match on emitted source)
  - `cross_target_optimization_cost_structurally_derived` — for the certification corpus, the cost-lens reading of the emitted target program (composed via `Lens<SymbolicCost>·LanguageSpec` per T-CostLens-Composition) equals the structural composition of (a) `.dag` algebra-level cost from `Lens<SymbolicCost>` + (b) per-primitive realization cost from the target's LanguageSpec — **no runtime measurement**; both readings are structural folds over substrate. Same shape as `coercion_cost_equals_complexity_by_construction` from T-CostLens-Composition; this gate restates that structural property over the certification corpus to operationalize the "cost lens drives lowering, not target-specific heuristics" free-consequence claim. Runtime perf measurement is post-R3 (per Design challenge #7 — measurable-or-deferred; R3 deliverable is structural close)
- **T-Workflow-As-Data** (NEW 2026-05-04; observation-driven lens-shape class).
  - `workflow_substrate_carriers_landed` — workflow grammar in `.dag` (`std.workflow` carriers); supports CI / build / internal-compiler workflows uniformly without per-shape special-casing
  - `timing_lens_carrier_landed` — `Lens<TimingMeasurement>` carrier authored per Substrate Mgr STOP+PING design receipt for `docs/design-timing-lens.md`; `TimingMeasurement` + `TimingObservationSet` + `TimingBudget` carriers live; `Output` is projection/report distinguishing `Observed | Missing | Ambiguous | Stale` (fail-closed enforcement on non-observed states per `feedback_fail_closed_discipline`)
  - `shared_external_attachment_pattern_documented` — `WorkflowObservationAnchor` factored separately from timing as reusable external-data attachment primitive (serves coverage / logs / failures / artifacts beyond timing); pattern documented in design-timing-lens doc with six invariants (stable subject identity not span; observed-artifact identity/digest; producer/observer/prover identity; attachment timestamp + run id; stale/ambiguous/missing/observed report states; fail-closed enforcement on non-observed/non-valid states); promotion to generic `ExternalDataAnchor<Subject, Source>` carrier deferred to second concrete consumer per Substrate Mgr design stance (`ProofReceipt` likely; per `gunb-ai/ctrl#369` reshaped scope)
  - `ci_workflow_modeled_as_dag` — at least one workflow modeled as `.dag` data (CI workflow recommended as demonstration target per Director scope); workflow-as-data thesis instantiated; `gunb-ai/gunb.ai` repo's existing CI infrastructure (substantial bazel-ci.yml at ~84KB) surveyed for portable patterns during carrier authoring (worker-level survey, not gating substrate-shape decision)
- **T-Lens-Self-Application** (NEW 2026-05-04; demonstrates recursive-flex thesis claim).
  - `lens_self_application_demonstrated` — gunbc lenses (cost / complexity / parallelism / timing) applied to gunbc's own build/CI workflow (modeled as `.dag` data per T-Workflow-As-Data); produces `DimensionReport<C>` for each lens; demonstrates that the same lens framework users get also validates the workflow that produces gunbc itself
  - `apply_lens_self_application_demonstrated` — `apply_lens(timing, ci_workflow, Enforce { budget: max_ns })` enforced via existing `EnforcedApplication<Output, Budget>` carrier (per T-Lens-Application-Surface §2 + §3.2); fail-closed when observation reports `Missing | Ambiguous | Stale` per timing-lens design; demonstrates that user-authored apply_lens substrate generalizes to project-self-validation use case
  - `recursive_flex_demonstration_landed` — narrative-load-bearing thesis claim cashes in worked instance: *"the compiler that compiles gunbc programs validates the workflow that produces gunbc itself."* Compiler-people read this as the categorical move that the lens framework isn't just for user code — it applies recursively to gunbc's own runtime behavior. Either emit-back-to-CI-YAML (bidirectional case via T-Workflow-As-Data) OR direct execution. <1 min CI target tracked as informational SLO; **not a closure gate** (per Director ratification 2026-05-04 — performance metric, not structural commitment)

## Lane structure

| Lane | Size | Manager | Covers | R2-close dependency |
|---|---|---|---|---|
| **T-Tier3-Dissolution** | M | **Tier 3 Manager** (or PB Manager continuing post-R2) | Four hand-Rust mirrors of `.dag` types retired (mirror bodies replaced by Evaluator-backed authority inside `dag.rs` / `dag/effects.rs` / `workflow_idempotency.rs`); **consumer count / mirror-symbol count reaches zero**. SG-0 delta is reported and **usually 0** because the hand-authored file remains on the census after mirror-block retirement — SG-0 reaches 0 through broader PB-Substrate / generated-file retirement + T-LensProducer-Retirement, not as a direct Tier 3 consequence (per PB Manager review 2026-04-28) | R2-Evaluator (executes std bodies); ValueBody::Map carrier (landed in R2 post-#1017; map read-path/API + arrow-body evaluation are the remaining substrate gaps for `kernel_algebra_profile`) |
| **T-LensProducer-Retirement** | XL | **PB Manager (post-R2 continuation)** | Three program-sized hand-Rust files retired via PB-Runtime + PB-1 patterns. **Internal sub-gates** (per Director directive 2026-04-28 — XL framing kept; sub-gate visibility for closure-ledger reporting): (i) `lens_apply.rs` retired (gated on PB-Runtime interpreter-as-data); (ii) `lens_testgen.rs` retired (same gate as `lens_apply.rs`); (iii) `regen_lens.rs` retired (gated on PB-1 bin-shim emit pattern — distinct gate). Closure ledger reports sub-gate progress so PB Manager can report sub-progress, but the lane is one program. **Plus advanced lifetime analyzer cases d/e/f** (closures, async lifetimes, self-referential/Pin) folded into this lane per design-emission-model.md Open call 2 — the lifetime analyzer is structurally what replaces `lens_apply.rs`'s reflection work, so advanced cases land alongside retirement | R2-Evaluator (interpreter-as-data); PB-1 generated bin-shim pattern (which itself depends on Evaluator); R2-T-Ground-Lifetime-Analyzer (a/b/c basic cases) |
| **T-Verification-L4-L7-Direct** | M | **Verification Manager** (new) | L4 emit/eval match harness + L7 algebraic-law witness construction. Evaluator-direct; can start as soon as R2-Evaluator + R2 close. **NOT a `Lens<C>` instance** (per codex BLOCKING `f5f63c7d9` — `Lens<C>.read: (Dag, Behavior) → Witness<C>` cannot read emitted target artifacts; L4/L7 are *runtime equivalence checks* that compare two computational results — emit-target output vs .dag eval result). **Consumes** `Lens<C>` instances as inputs where useful (e.g., `Lens<SymbolicCost>` for cost-related claims), but the lane itself is a corpus-driven runtime harness, not a structural fold. **Note:** L6's structural completeness check (in R2-T-Ground-CrossTarget-Meta) is also NOT a Lens<C> instance — different input space (per-(form × target) vs per-Behavior); L6 lives as its own substrate-load-time completeness primitive per codex BLOCKING `90220bd97`. | R2-Evaluator (witness construction) + **R2-T-Substrate-Lens-Primitive** (consumed as input substrate, not as L4-L7 framing) |
| **T-Verification-L5-Corpus** | M | **Verification Manager** | L5 cross-target equivalence only. Corpus-driven; needs (a) all 3 Shape A targets grounded, (b) L4 corpus from T-Verification-L4-L7-Direct existing first. (L6 form coverage moved to R2-T-Ground-CrossTarget-Meta as a structural cross-product fold; see §"Acceptance" note.) | R2-Grounding-Rust + R2-Grounding-Python + T-Verification-L4-L7-Direct |
| **T-FixedPoint** | M | **PB Manager** | `compiler.dag` compiles to bit-identical stage0 Rust + bit-identical emitted artifacts; R1's `pb_self_compile_fixed_point` gate closes under stronger interpretation | R2-Evaluator (executes compiler.dag); SG-0 zero from T-LensProducer-Retirement |
| **T-Numeric-Construction (REFRAMED 2026-05-01 from T-Int128)** | L-XL | **Substrate Manager (post-R2 continuation)** | Construction chain Magnitude → Nat → Int → Rational → Real + width refinements (`Int<N>`, etc.); **13 types in scope** (3 direct: `Int`, `UInt`, `Float`; 10 inherited via `Int` chain: `Char`, `EpochMs`, `Duration`, `Milliseconds`, `Seconds`, `RetryCount`, `HttpStatus`, `Port`, `PositiveInt`, `NonNegativeInt`); absorbs T-Int128 + post-R3 BigInt + Float/UInt widening + IntLit refinement; eager-ram's #1333 `Word128Carrier` preserved as storage refinement under `Int<128>`. Per Director ratification 2026-05-01 + design doc [`docs/design-numeric-construction.md`](design-numeric-construction.md). **Scope-count expansion 2026-05-02**: original count 8 (3 direct + 5 inherited; PR #1430 §A audit) extended to 13 after fresh PM sweep found 5 additional Int-inherited refinement types (`RetryCount` / `HttpStatus` / `Port` / `PositiveInt` / `NonNegativeInt`) at `dsl/std/types.dag:232-245`; Substrate Mgr acknowledgment at [gunbc#1130 comment 4360482400](https://github.com/gunb-ai/gunbc/issues/1130#issuecomment-4360482400) — folded into worker-brief planning (cascade auto-fixes once `Int` becomes abstract; `NonNegativeInt` / `PositiveInt` flagged for Slice 2 Nat-alignment migration; `RetryCount` / `HttpStatus` / `Port` flagged as bounded-range cost-lens candidates for `Nat<3>` / `Nat<10>` / `Nat<16>` width-refinement once refinement composition lands). Includes Substrate-Mgr String audit fold-in (per Director scope-add). | T-V2-Retirement landing first (path-(a) coordination for refinement syntax `Int<N>` / `where bits <= N` v2-blocker). Otherwise self-contained substrate work. |
| **T-Omni-Shape-B** | L | **Demo Manager** (or R3 Release Manager) | At least 2 Shape B omni-emission demos exercising the full-stack thesis claim | R2-Evaluator (Shape B emitters are `.dag` programs walking typed values via fold/match — needs runtime to demonstrate properly) |
| **T-Anthropic-Wire** | M | **Substrate Manager (post-R2 continuation)** | Anthropic provider request/response typed end-to-end | None (parallel; held in R2 pending OpenAI stabilize) |
| **T-Bridge-Retirement** | M | **Verification (ledger only); retirement work distributed per bridge map** (Director-locked 2026-04-28: distribute-work-centralize-ledger discipline match — bridges retire in PB/Substrate territory; Verification owns the unified `bridge_retirement_ledger_zero` audit gate) | **Bridge distribution map** (5 named bridges): (1) `SourceSpan.file` participation checks → **Substrate** (typed identity surface); (2) `mark_bootstrap_secret_nominal_opacity()` → **Substrate** (Secret PR A continuation lineage); (3) canonical lens-name dispatch → **PB Manager** (lens-producer-retirement adjacent); (4) `include_str!` side channels (e.g., pipeline_authority.rs) → **PB Manager** (compiler-internal bootstrap); (5) `patch_lower_helpers_*` residual → **PB Manager** (Tier 2 retirement lineage; #1014 was first slice). **Net: 3 Substrate-owned + 3 PB-owned + 1 Verification-owned ledger.** Verification's `bridge_retirement_ledger_zero` audit gate verifies cross-program coordination/reporting cadence; the actual retirement work absorbs into existing Substrate / PB scopes without spawning a parallel manager | R2 substrate carriers (typed identity surfaces); per-bridge gates depend on the natural-owner program's prerequisites |
| **T-CostLens-Composition** | M | **Substrate Manager (post-R2 continuation)** (Director-locked 2026-04-28: substrate-shape match — T-CostLens-Composition is substrate-authoring of cost facts (per-op algebra cost + per-primitive realization cost) + Lens<SymbolicCost> instance demonstration. Substrate authors; Verification asserts the gate. Different concerns shouldn't fold into one manager.) | Cost lens composes `.dag` algebra-level cost + target-primitive realization cost via the language spec; structural fold, not engine policy. **Instance of `Lens<C>`** (from R2-T-Substrate-Lens-Primitive) with `C = SymbolicCost`. Verifies "coercion cost = complexity" holds by construction. No "coercion cost" dimension. Per Modeling problem 8 in [`docs/design-emission-model.md`](design-emission-model.md). | R2-Evaluator (witness construction for cost claims) + **R2-T-Substrate-Lens-Primitive (the `Lens<C>` shape)** + R2-T-Substrate (per-operation cost on every algebra) + R2-T-Ground-LanguageSpec (per-primitive realization-cost declarations) |
| **T-V2-Retirement** (NEW 2026-04-30) | S-M | **PB Manager (post-R2 continuation)** | v2 retirement is largely a *consequence* of T-FixedPoint + T-LensProducer-Retirement closing; pulling it into R3 is structurally cheap; the post-R3 framing was coordination convenience, not technical blocker. **Scope:** ~79 `.rs` files + ~32 `.dag` files in `src/v2/`; ~13 v2-using test files; legacy emit chain (`rust_method_template_contracts.dag` header note); dual `verification.dag` convergence (per `design-test-infra.md:14`). **Gates:** `v2_oracle_no_remaining_test_consumers` (no test references `src/v2/`); `v2_directory_deleted` (workspace member removed; bootstrap routes through PB-Runtime trampoline only). Rationale: user directive 2026-04-30 — *"nothing can be deferred past R3."* | T-FixedPoint + T-LensProducer-Retirement |
| **T-Free-Consequences-Demonstration** (NEW 2026-04-30) | S-M | **Verification Manager** | Operationalizes thesis "free consequences" framing with structural test-claim suite. Per user directive 2026-04-30 — *"what guarantees does the compiler ACTUALLY provide - i expect a small doc/testcases."* **Deliverables:** (1) `docs/design-free-consequences.md` per-consequence guarantee analysis grounded in 5 substrate behaviors + Lens<C> framework. (2) **TestClaim suite (10 gates):** auto-parallelism × 3 + auto-loop-parallelism × 3 + auto-memoization × 2 + cross-target-optimization × 2. **Loop-iteration parallelism design call (Director-ratified 2026-04-30):** sequential default + opt-in via `Lens<Iteration-Independence>` (zero-heuristic; same shape as `Lens<Bind-Independence>`); aligns with `feedback_lenses_not_passes`. | R2-Evaluator (witness construction); R2-T-Substrate-Lens-Primitive (`Lens<C>` shape); T-CostLens-Composition (cost-related claims) |
| **T-E-P-Producer-Broadening** (NEW 2026-05-02) | M-L | **Substrate Manager (post-R2 continuation)** | Foundational. Broaden per-call `DescentEvidence` / `CallPattern` / `SubValueRelation` producer coverage from current first slice (recursive self-call + arithmetic-descent only) to full `ExprCall.descent_evidence` parity at live call sites. **Gates:** `e_p_per_call_descent_evidence_full_coverage`, `e_p_call_pattern_lookup_authoritative`, `e_p_sub_value_relation_per_call_landed`. Per Director ratification 2026-05-02 ([gunbc#828 comment 4362742638](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4362742638)). | R2 substrate carriers (already landed) + existing E-T/E-C/E-I vocabulary |
| **T-Lens-Behavioral-Parity** (NEW 2026-05-02) | L-XL | **Substrate Manager + Verification Manager (cross-program)** | Bring complexity / cost / parallelism / effect_enumeration lenses from PROXY/STUB/PARTIAL to BEHAVIORALLY COMPLETE per `docs/v3-lens-capability-register.md`. 4 sub-slices (parallel-dispatchable post-T-E-P-Producer-Broadening): (1) **complexity** — symbolic CostExpr full algebra (Sum/Mul/Log/Const) consumed by lens; work/span dimension split; asymptotic classification; cementing test against v2 oracle. (2) **cost** — same producer foundation as complexity; `SizeVar` value semantics; `Dimension<SymbolicCost>` wiring; cementing test. (3) **parallelism** — Stage 2e walk port from `src/v3/compiler/src/workflow_parallelism.rs` to `.dag`; rewire via `lane2_workflow_at` / `std.effects` (idempotency closure template). (4) **effect_enumeration** — resource-threading migration; ambient metadata removal; caller-side effect-set pinning; full `OperationEffect` retirement. **Gates:** `complexity_lens_behaviorally_complete`, `cost_lens_behaviorally_complete`, `parallelism_lens_behaviorally_complete`, `effect_enumeration_lens_behaviorally_complete`, `lens_capability_register_zero_proxy_zero_stub`. Per user directive 2026-05-02. | T-E-P-Producer-Broadening (foundational); R2-Evaluator (lens runtime execution); R2-T-Substrate-Lens-Primitive (Lens<C> shape) |
| **T-Tests-As-Data-Completeness** (NEW 2026-05-02) | L | **Verification Manager** | Close Category E test/verification surface gaps per user directive: (1) every Rust test ports to `.dag` TestClaim or generated target-language test code (thesis facet 3 — *"tests are data"* — full coverage); (2) property-based testing surface (`ForAll` / `Exists` quantifiers + `ProgramGenerator` substrate carrier; substrate-introduction); (3) cementing test discipline for `.dag` lenses (per-lens v2 oracle equivalence on same source). **Gates:** `every_rust_test_ports_to_dag_or_generated`, `forall_exists_quantifier_substrate_landed`, `program_generator_carrier_landed`, `lens_cementing_test_discipline_complete`. | R2-Evaluator (test execution runtime); existing TestClaim infrastructure (DB-15 R2) |
| **T-Lens-Application-Surface** (NEW 2026-05-02; design doc landed 2026-05-02 [`docs/design-lens-application-surface.md`](design-lens-application-surface.md)) | L-XL | **Substrate Manager + Verification Manager (cross-program)** | First-class authoring surface for applying lenses to arbitrary `.dag` sections (function / module / expression / declaration scope). Per user reframe 2026-05-02: lens application is a `.dag` declaration with configurable behavior — `apply_lens(lens, section, config)`. **Subsumes** prior T-Complexity-Contract-Compile-Error + T-User-Authored-Cost-Basis-Discipline as configurations of one mechanism. **Substrate carriers** (per design doc §2): **two separate top-level carriers** — `EnforcedApplication<Output, Budget>` and `IntrospectApplication<Output>`. NOT a sum wrapping the two — v3 `.dag` substrate cannot currently express per-variant generic parameters; "SectionedLensApplication" is a collective noun for both carriers. Plus `SectionRef` (DeclarationScope/NodeScope disjoint sum) + `LensEnforcement<Output, Budget>` projection carrier + `DiagnosticSeverity`. `CompileError | Warning | Silent` original user framing resolved to fail-closed-compatible binary per design doc §3 + INVARIANTS C-8. **Default policy for complexity contracts**: user-driven (per design doc §3.2 + §8.3 resolution at e9d67113e). Unannotated functions get synthesized `Introspect`-only applications — no implicit baseline, no inferred Enforce. Enforcement requires explicit user authoring of `apply_lens(complexity, fn, Enforce { ... })`. The original "opt-out" framing is reframed: the user can opt out (no Enforce or explicit Introspect), and compile errors fire when the user opts IN with a budget the function exceeds. `ComplexityBudgetWaiver` retains its purpose for accepting known violations of explicit user contracts (NOT an annotation per `feedback_no_annotations`). Demonstrations: complexity-contract-compile-error + CRDT cost basis + memory-peak cost basis + opt-in cross-iteration parallelism (4 worked examples; orthogonal axes per Director ratification; design doc §4). **Gates:** `lens_application_carrier_landed`, `section_ref_substrate_landed`, `lens_enforcement_carrier_landed` (per-lens `LensEnforcement<Output, Budget>` projection + violation-relation declarations), `enforce_violation_routing_landed`, `complexity_violation_compile_error_demonstrated`, `crdt_cost_basis_demonstrated`, `memory_peak_cost_basis_demonstrated`, `opt_in_iteration_parallelism_via_lens_application_demonstrated`. **Design doc §8 resolves all 5 originally-open questions** (module-scope semantics / multiple-applications / default-budget-inference / waiver lifecycle / cross-section composition); no Director ratification required before substrate authoring — only standard cascade-gate (T-Lens-Behavioral-Parity COMPLETE) + R2-Evaluator landed. | T-Lens-Behavioral-Parity (lenses must be COMPLETE first); R2-Evaluator |
| **T-Workflow-As-Data** (NEW 2026-05-04; per Director ratification at [gunbc#828 inbox-4374342708](https://github.com/gunb-ai/gunbc/issues/828); Substrate Mgr design stance at [gunbc#1130 comment-4374109666](https://github.com/gunb-ai/gunbc/issues/1130#issuecomment-4374109666)) | M-L | **Substrate Manager (post-R2 continuation)** | Substrate work for modeling workflows as `.dag` data; introduces observation-driven lens-shape class (parallel to existing structural-static `Lens<C>` instances). **First instance**: timing-lens substrate (`Lens<TimingMeasurement>`). **Substrate carriers**: `TimingMeasurement` + `TimingObservationSet` + `WorkflowObservationAnchor` (factored separately as reusable external-data attachment primitive — Shared External Attachment Pattern with six invariants per Substrate Mgr design stance) + `TimingBudget`. **Workflow grammar**: at least one workflow modeled as `.dag` data (CI workflow recommended). **Bidirectional case** coordinates with `gunb-ai/gunbc#1586` thread anchor 7 (workflow-timing as bidirectional architecture concern). **Gates:** `workflow_substrate_carriers_landed`, `timing_lens_carrier_landed` (per Substrate Mgr STOP+PING design receipt for `docs/design-timing-lens.md`), `shared_external_attachment_pattern_documented`, `ci_workflow_modeled_as_dag`. Substrate Mgr's design-doc-first cadence per `r3-structure.md:187` substrate-completion protocol — design receipt lands first, carrier authoring follows sign-off. | T-Lens-Behavioral-Parity COMPLETE (timing-lens uses lens framework with parity-COMPLETE consumers); R2-Evaluator |
| **T-Lens-Self-Application** (NEW 2026-05-04; per Director ratification at [gunbc#828 inbox-4374342708](https://github.com/gunb-ai/gunbc/issues/828)) | M-L | **Verification Manager** | Demonstration work: gunbc applies its own lenses (cost / complexity / parallelism / timing) to gunbc's own build/CI workflow. Operationalizes recursive-flex thesis claim: *"the compiler that compiles gunbc programs validates the workflow that produces gunbc itself."* Concrete first instance: timing-lens applied to CI workflow producing `DimensionReport<TimingMeasurement>`; `apply_lens(timing, ci_workflow, Enforce { budget: max_ns })` enforced via existing `EnforcedApplication<Output, Budget>` carrier. <1 min CI target as informational SLO; **NOT a closure gate** (per Director ratification 2026-05-04 — performance metric, not structural commitment). **Gates:** `lens_self_application_demonstrated`, `apply_lens_self_application_demonstrated`, `recursive_flex_demonstration_landed`. | T-Workflow-As-Data; T-Lens-Application-Surface; T-Lens-Behavioral-Parity COMPLETE; R2-Evaluator |

Critical path: **T-E-P-Producer-Broadening → T-Lens-Behavioral-Parity → T-Lens-Application-Surface → (T-Workflow-As-Data + T-Lens-Self-Application)** is the R3 critical-path chain (extended 2026-05-04 with recursive-flex commit). T-Verification-L4-L7-Direct → T-Verification-L5-Corpus remains the longest verification path. Other lanes parallel-dispatch after R2-Evaluator closes.

**R3 lane count: 18 lanes + 1 standing program** (12 original + T-E-P-Producer-Broadening + T-Lens-Behavioral-Parity + T-Tests-As-Data-Completeness + T-Lens-Application-Surface + T-Workflow-As-Data + T-Lens-Self-Application; standing R3 Debt-Paydown program; expansions 2026-05-02 + 2026-05-04 per user directives that all "accidentally deferred" gaps absorb into R3 + recursive-flex/self-application case commits to R3 scope).

Plus 3 fold-ins (no new lanes; Director-ratified 2026-05-02):
- **T-V-L4-L7-Direct exhaustive witness coverage** — per-(algebra, inhabitant, law) witness coverage ensures bug class like SymbolicCost product-zero (PR #1430 §A) is structurally caught by `l7_algebraic_laws_witnessed` gate at all inhabitants.
- **T-Ground-Diagnostic closed-axis enforcement** — replace `MissingEmissionPath { connective: String, behavior: String, target: String }` (Reflective Pattern from PR #1430 §F) with closed-axis enums; no String dispatch on closed sets.
- **T-LensProducer-Retirement d/e/f confirmation** — ownership analysis cases d (closures) + e (async lifetimes) + f (self-referential / Pin) confirmed delivered before R3 close, not separately deferred.

Plus 2 priority corrections folded into existing lanes (Director-ratified 2026-04-30):
- **C1 — Tier-3 mirror dissolution perf budget** sub-gate of T-Tier3-Dissolution: `tier3_mirror_dissolution_perf_within_budget` with thresholds **≤2× median, ≤5× p99**.
- **C2 — Provider generalization path (a)** folded into Substrate continuation (extends T-Anthropic-Wire scope): extract `ProviderTypedWire<P>` carrier.

## Standing program — R3 Debt-Paydown (NEW 2026-05-02)

**R3 Debt-Paydown Manager** is the **9th standing R3 manager** (added per Director ratification 2026-05-02 + user directive *"R3 to clean up any and all debt we come across WITHIN R3 - even if it means dedicated debt paydown lanes/managers"*).

**Owned program**:
- **ROADMAP debt-row retirement** — every tracked-debt row in ROADMAP `### Post-merge debt (...)` sections gets a named retirement PR or explicit closure receipt before R3 close
- **Velocity-tripwire enforcement** — per `INVARIANTS.md §P5` (Progress Is Dissolution) Dispatch-Discipline Mechanisms (c) Velocity tripwire (introduction:dissolution PR ratio ≥3:1 in any 7-day window); manager surfaces tripwire readings to Director on cadence
- **Per-PR discipline rule** — every R3 PR includes "debt receipt": names debt found in this PR, names debt paid in this PR, OR names a paydown-lane retirement PR for the debt; vague deferrals rejected
- **Closure-receipt cadence** — converts Director's reflective-analysis findings into per-PR retirement work; closes the gap between "tracked debt" and "retired debt"

**Closure gate**: `r3_debt_paydown_zero_remaining` — no tracked-debt rows survive R3 close. Every tracked-debt row retires with a PR receipt before R3 close — unconditional. Per user directive 2026-05-02 (*"all 'accidentally deferred to post R3' into R3 now"*): **there is no post-R3 deferral path for tracked debt**. If a row appears unretirable within R3 it surfaces as a substrate gap requiring a named R3 lane (the directive that motivated this manager's creation), not a deferral. Per `INVARIANTS.md §P5` (Progress Is Dissolution): a tracked-debt row deferred past R3 close is the bridge-as-steady-state pattern P5 explicitly forbids; an escape hatch for "rare structurally-justified deferrals" reintroduces that pattern at lower cadence and must not exist.

**Authority discipline**: Manager authors per-PR rule documentation + standing reporting cadence. Does not author lane-level structural-acceptance gates (those owned by lane-owning managers). Does not adjudicate cross-program scope conflicts (those route to Director). Does enforce: every tracked-debt row gets a retirement PR before R3 close (per closure-gate at line 173 — no post-R3 deferral path; if a row appears unretirable, manager surfaces it as a substrate gap requiring a named R3 lane, not a deferral).

**Hybrid mechanism**:
- **(a) Per-PR discipline rule** — every R3 PR includes debt receipt; vague deferrals rejected. Same shape as the per-PR gate from `INVARIANTS.md §P5` Dispatch-Discipline Mechanisms (b).
- **(b) Standing manager territory** — owns systemic debt that doesn't fit organic per-PR cleanup; spawns workers for retirement work as needed; reports cadence-aligned to Director's autonomous-loop pattern.

**Cross-program coordination**: Debt-Paydown Mgr coordinates with all 8 other R3 managers via the standard cross-manager queue + closure-ledger receipts.

## Manager structure

R3 inherits R2's manager structure with **four modifications** (originally 3; expanded 2026-05-02 to 4 with Debt-Paydown Mgr addition):

1. **R2 managers continue post-R2-close** rather than dissolving. **Substrate Manager** continues with **T-Numeric-Construction (REFRAMED 2026-05-01 from T-Int128) + T-Anthropic-Wire (incl. ProviderTypedWire<P> per C2 ratification 2026-04-30) + T-CostLens-Composition + T-E-P-Producer-Broadening (NEW 2026-05-02) + T-Lens-Behavioral-Parity (NEW 2026-05-02; cross-program with Verification) + T-Lens-Application-Surface (NEW 2026-05-02; cross-program with Verification) + T-Workflow-As-Data (NEW 2026-05-04)** (7 lanes total — expanded 2026-05-04 to absorb workflow-substrate work for recursive-flex commit); **PB Manager** continues with T-LensProducer-Retirement (incl. ownership d/e/f confirmation per 2026-05-02 fold-in) + T-FixedPoint + T-Tier3-Dissolution + **T-V2-Retirement (NEW 2026-04-30)** + 3 distributed bridge retirements (canonical lens-name dispatch / `include_str!` side channels / `patch_lower_helpers_*` residual — per T-Bridge-Retirement distribution map); Modeling/Impossible-Bugs Managers archive at R2 close. **Post-R2 emergent work disposition** (Director-locked 2026-04-28; carried forward through 2026-05-04 expansion): if `ctrl/` pressure-test or other post-R2 work surfaces new impossible-bug classes, modeling refinements, or substrate gaps, those are absorbed by **Substrate Manager continuation** as substrate-completion work — they're evidence of substrate gaps (per closed-system principle: enumerated bug classes are exhaustive over substrate; new classes = enumeration was wrong = substrate gap to fill), not new lanes spawning new managers.

2. **Verification Manager (new)** — owns T-Verification-L4-L7-Direct (incl. per-(algebra, inhabitant, law) exhaustive witness coverage per 2026-05-02 fold-in) + T-Verification-L5-Corpus + **T-Free-Consequences-Demonstration (NEW 2026-04-30)** + **T-Tests-As-Data-Completeness (NEW 2026-05-02)** + **T-Lens-Self-Application (NEW 2026-05-04)** + cross-program portion of T-Lens-Behavioral-Parity + cross-program portion of T-Lens-Application-Surface + the `bridge_retirement_ledger_zero` audit gate of T-Bridge-Retirement (Director-locked 2026-04-28: ledger-only ownership; retirement work distributes per bridge map — see Substrate/PB continuation above). Total: **6 lanes + 2 cross-program partners + 1 ledger gate** (expanded from 5 lanes + 2 cross-program partners + 1 ledger gate per 2026-05-04 directive committing recursive-flex demonstration to R3). Why a new manager: this cluster shouldn't fold into Substrate or PB; structural-acceptance-by-construction is its own discipline. **L6 is NOT in Verification Manager's scope** — it was reclassified out of R3 as a structural cross-product fold and lives in R2-T-Ground-CrossTarget-Meta (Grounding Manager's program). **T-CostLens-Composition is NOT in Verification Manager's scope** (Director-locked 2026-04-28, carried forward through 2026-05-04 expansion: substrate-authoring of cost facts + Lens<SymbolicCost> instance — under Substrate continuation; see lane table above).

3. **R3 Release Manager (new, may be R2 Release Manager continuation)** — owns T-Omni-Shape-B, R3 closure ledger, R3 demo coordination. Goal-6-equivalent for R3.

4. **R3 Debt-Paydown Manager (NEW 2026-05-02; 9th standing R3 manager)** — owns T-Debt-Paydown standing program (ROADMAP debt-row retirement + velocity-tripwire enforcement + closure-receipt cadence + per-PR discipline rule). Hybrid mechanism: per-PR rule + standing capacity. Closure gate: `r3_debt_paydown_zero_remaining`. Per `feedback_standing_managers_need_owned_deliverables`: standing manager territory because Director ad-hoc would bottleneck. See §"Standing program — R3 Debt-Paydown" above for full authority shape.

**T-Behavioral-Expectations-Documentation** is parallel-dispatchable across all R3 managers (per-feature deliverables under their respective lane-owning Mgrs). Not a separate manager territory; any available worker can claim a feature doc. 7 load-bearing features per Director ratification 2026-05-02.

Director's role unchanged: cross-program conflict resolution + scope-change escalation + weekly health check.

**Total R3 active surfaces**: 18 lanes + 1 standing program = **19 active surfaces** (expanded from 17 active surfaces at 2026-05-02 lock — added 2026-05-04 per user directive committing recursive-flex / self-application case to R3: 2 new lanes (T-Workflow-As-Data, T-Lens-Self-Application) = +2 surfaces; 17 + 2 = 19). T-Behavioral-Expectations-Documentation is parallel-dispatchable across existing lanes, not a separate surface (per §"Behavioral Expectations Documentation" above). Manager-structure modifications (4 total — Substrate continuation expansion, Verification Manager, R3 Release Manager, Debt-Paydown Manager) are organizational, not separately-counted surfaces — each distributes existing-or-new lane ownership rather than creating a new program surface. Manager count: **9 standing R3 managers** (8 + Debt-Paydown).

## Dependency DAG

**Authority note**: this DAG is **illustrative**; the canonical lane list with full per-lane R2-close dependencies lives in §"Lane structure" table. Counts/lane lists are not duplicated here — see §"Lane structure" + §61 lane-gating-summary + §"Dependency on R2" for the authoritative 18-lane / 14-gated state. This section visualizes critical-path shape only.

```
                                  R2 close
                                     │
                                     ▼
                              R2-Evaluator landed
                                     │
       ┌────────────┬────────────┬───┴────┬────────────┬────────────┐
       │            │            │        │            │            │
       ▼            ▼            ▼        ▼            ▼            ▼
T-Tier3-Diss  T-LensProducer  T-V-L4-L7-Direct  T-FixedPoint  T-Omni-Shape-B  (T-Numeric-Construction)
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
                                            wire stabilization;
                                            scope expanded 2026-04-30 per C2 ratification:
                                            +ProviderTypedWire<P> carrier + per-provider
                                            parameter rows)
                       T-Bridge-Retirement ◄── (parallel substrate-completion;
                                              partial side-effect from
                                              T-LensProducer-Retirement)
                       T-CostLens-Composition ◄── (Evaluator-gated; also
                                                  gated on R2-T-Substrate
                                                  per-operation algebra cost
                                                  + R2-T-Ground-LanguageSpec
                                                  per-primitive realization cost)
                       T-V2-Retirement ◄── (NEW 2026-04-30; cascade-gated:
                                            T-FixedPoint AND T-LensProducer-Retirement
                                            must close first — v2 retirement is largely
                                            a *consequence* of those two)
                       T-Free-Consequences-Demonstration ◄── (NEW 2026-04-30;
                                                              R2-Evaluator-gated for
                                                              witness construction +
                                                              R2-T-Substrate-Lens-Primitive
                                                              for Lens<C> instances;
                                                              T-CostLens-Composition for
                                                              cost-related claims)

  R3 critical-path chain (NEW 2026-05-02 + extended 2026-05-04):
                                  │
                                  ▼
                       T-E-P-Producer-Broadening (foundational substrate;
                                                  Substrate-Mgr; M-L)
                                  │
                                  ▼
                       T-Lens-Behavioral-Parity (L-XL; cross-program
                                                 Substrate + Verification;
                                                 4 sub-slices: complexity,
                                                 cost, parallelism,
                                                 effect_enumeration)
                                  │
                       ┌──────────┴──────────┐
                       ▼                     ▼
              T-Lens-Application-Surface   T-Tests-As-Data-Completeness
              (L-XL; Substrate + Verif      (L; Verification-Mgr)
               cross-program; carrier-       — parallel branch from
               general apply_lens)             T-LBP COMPLETE
                       │
                       ▼
              T-Workflow-As-Data (NEW 2026-05-04; Substrate-Mgr; M-L;
                                  observation-driven lens-shape class —
                                  workflow grammar + timing-lens substrate +
                                  Shared External Attachment Pattern)
                       │
                       ▼
              T-Lens-Self-Application (NEW 2026-05-04; Verification-Mgr; M-L;
                                       recursive-flex demonstration — gunbc
                                       applies own lenses to own build/CI workflow)
```

**Parallel-capable work at steady state**: 14 of 18 lanes are R2-Evaluator-gated; 4 of 18 are non-gated parallel substrate-completion (T-Numeric-Construction, T-Anthropic-Wire, T-Bridge-Retirement, T-E-P-Producer-Broadening). Once R2-Evaluator lands + the cascade prerequisites close, the 14 gated lanes parallel-dispatch with critical-path serialization on the chain shown above. Critical path is `R2-Evaluator → T-LensProducer-Retirement → T-FixedPoint → T-V2-Retirement` (fixed-point requires SG-0 = 0 which requires lens-producer retirement; v2 retirement cascades on fixed-point + lens-producer closing). Verification has its own internal critical path: `T-V-L4-L7-Direct → T-V-L5-Corpus` (because Corpus's L5 cross-target work consumes Direct's L4 corpus). The new R3 critical-path chain (T-E-P-Producer-Broadening → T-Lens-Behavioral-Parity → T-Lens-Application-Surface → T-Workflow-As-Data + T-Lens-Self-Application) parallels both. T-Free-Consequences-Demonstration parallels the Verification critical path post-R2-Evaluator.

## Compromises being made

R3 commits to closing the consequence layer of the thesis. The following are *not* in R3 scope:

| Excluded | Why | Where it lives instead |
|---|---|---|
| **Practical pressure-test of thesis on real programs** | Per [`docs/r2-structure.md`](r2-structure.md), the user's `../ctrl/` modeling work is the empirical pressure-test for whether the structural thesis holds on real programs. R3 is structural close; pressure-test is post-R3 external | Post-R3 stream (per existing r2-structure.md decision) |
| **Adoption tooling, ecosystem, community** | Not a thesis claim; downstream of structural close | Post-R3 external |
| ~~**v2 retirement**~~ | ~~Per r2-structure.md, v2 retirement is post-R3 operational cleanup, not on the release ledger~~ — **MOVED INTO R3 2026-04-30** as **T-V2-Retirement** lane (see Lane structure §11) per user directive *"nothing can be deferred past R3."* PM audit ratified pulling it in: v2 retirement is largely a consequence of T-FixedPoint + T-LensProducer-Retirement closing; pulling into R3 is structurally cheap | ~~Post-R3 operational cleanup~~ — **R3 lane (T-V2-Retirement)** |
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

**DESIGN LOCK introduced by PR-D:** [`docs/design-cross-target-equivalence.md`](design-cross-target-equivalence.md) defines semantic observations, corpus curation, oracle validity, float policy, side-effect normalization, and the R3 consumption gates for T-Verification-L5-Corpus / PR-E planning.

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
| **PR-D** | Cross-target equivalence semantics (algebraic-equal corpus) | Design challenge #3 | Design lock introduced at [`docs/design-cross-target-equivalence.md`](design-cross-target-equivalence.md) |
| **PR-E** | Evaluator dispatch brief (after PR-A through PR-D land) | Worker dispatch precondition | Authored at [`docs/briefs/r3-evaluator-dispatch.md`](briefs/r3-evaluator-dispatch.md) |

Workers cannot dispatch on under-specified scope, especially on multi-week T-Verification critical path. PR-B through PR-D are gates; PR-E starts Evaluator implementation work.

## Dependency on R2

R3 cannot start meaningful work until R2 closes. Specifically:

- **R2-Evaluator** is the upstream gate for **14 of 18 R3 lanes** (T-Tier3, T-LensProducer, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition, **T-V2-Retirement** [via T-FixedPoint + T-LensProducer-Retirement cascade], **T-Free-Consequences-Demonstration** [witness construction + lens-instance prerequisites], **T-Lens-Behavioral-Parity** [Evaluator runtime needed for behavioral lens execution], **T-Lens-Application-Surface** [via T-Lens-Behavioral-Parity cascade], **T-Tests-As-Data-Completeness** [test-execution runtime needed for porting Rust tests to `.dag` TestClaim], **T-Workflow-As-Data** [via T-Lens-Behavioral-Parity cascade — workflow lens consumption needs lenses COMPLETE], **T-Lens-Self-Application** [via T-Workflow-As-Data + T-Lens-Application-Surface cascade]). Without it, R3 dispatchers spin.
- **R2-Grounding-Rust + R2-Grounding-Python** are the upstream gate for T-Verification-L5-Corpus (specifically L5 cross-target).
- **R2 substrate carriers** (NominalOpacity, ValueBody::Map, parametric algebra) feed T-Numeric-Construction + T-Anthropic-Wire + T-Bridge-Retirement + T-E-P-Producer-Broadening as parallel substrate-completion work.

**R3 worker dispatch precondition** (Director-locked 2026-04-28; clarified 2026-04-28 per gpt-5-5-pro BLOCKING on `dbc48dc0` re P2 single-authority discipline; expanded 2026-04-30 for 12-lane structure; re-expanded 2026-05-02 for 16-lane structure; re-expanded 2026-05-04 for 18-lane structure with recursive-flex commit):

- **Applies to the 14 Evaluator-gated lanes** (T-Tier3-Dissolution, T-LensProducer-Retirement, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition, T-V2-Retirement, T-Free-Consequences-Demonstration, T-Lens-Behavioral-Parity, T-Lens-Application-Surface, T-Tests-As-Data-Completeness, T-Workflow-As-Data, T-Lens-Self-Application): R2-Evaluator landed AND R2-Grounding-Rust+Python landed. Pre-R3 *brief authoring* may begin during R2 final week (Director-discretionary, mirroring R2's pre-R1-close pattern), but worker dispatch waits for the joint precondition. This prevents R3 brief authoring from spawning drift if R2 close definition slips. **T-V2-Retirement** carries an additional internal cascade gate (T-FixedPoint + T-LensProducer-Retirement must close before v2 retirement work begins); pre-cascade brief authoring is still permitted under the same Director-discretionary rule. **T-Lens-Application-Surface** carries an additional internal cascade gate (T-Lens-Behavioral-Parity must reach BEHAVIORALLY COMPLETE before application-surface dispatch); pre-cascade design-doc work is permitted. **T-Workflow-As-Data + T-Lens-Self-Application** carry the same T-Lens-Behavioral-Parity COMPLETE cascade-gate as T-Lens-Application-Surface (workflow lens consumption + self-application demonstration both consume parity-COMPLETE lenses); pre-cascade design-doc + survey work permitted (e.g., gunb-ai/gunb.ai workflowgen survey for T-Workflow-As-Data).
- **Carve-out for the 4 non-Evaluator-gated lanes** (T-Numeric-Construction, T-Anthropic-Wire, T-Bridge-Retirement, T-E-P-Producer-Broadening): these are self-contained or substrate-completion work parallel to the Evaluator-gated lanes. They consume R2 substrate carriers but not the Evaluator itself, so they MAY dispatch pre-R3 (in parallel with R2-Evaluator work) per scheduling preference. The global R3 worker-dispatch precondition above applies ONLY to the 14 Evaluator-gated lanes; the 4 non-gated lanes operate as explicitly-scoped substrate-completion work outside that precondition. **T-Numeric-Construction has its own internal cascade gate** (T-V2-Retirement landing first per path-(a) v2-refinement-syntax-blocker coordination — see [`docs/design-numeric-construction.md`](design-numeric-construction.md)); pre-cascade design-doc and substrate-introduction-audit work is permitted under the same Director-discretionary rule.

This split resolves the prior single-authority drift between `:38` (non-gated lanes can dispatch in parallel) and the global precondition (worker dispatch waits): the precondition is now scoped to the 14 Evaluator-gated lanes only, with the 4 non-gated lanes explicitly carved out.

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
