# v3 Lens Capability Register

Canonical view of what every `.dag` lens under `src/v3/lenses/` actually computes, relative to its v2 counterpart (where one exists) and relative to the analysis its name implies. Load-bearing for direction review: any claim of the form "v3 replaces v2 X" must be checkable against a row in this table.

## Why this document exists

During the 2026-04-21 audit of the complexity lens we found that `src/v3/lenses/complexity.dag` (structurally TERMINAL) does not subsume v2's complexity analysis. v2's `ComplexitySummary { work, span, output_size, certainty }` carries symbolic cost expressions, size variables, work/span separation, and asymptotic classification. v3's `cost_of(dag, port) -> v3.std.lookup::Lookup<Int> = Miss | Hit(Int)` (imported from `v3.std.lookup` — the shared std carrier) carries a single integer depth per port and drops every other dimension. The 13× LOC reduction claim was comparing different functions of the program, not equivalent ones.

Extending the audit to the other nine lenses under `src/v3/lenses/` surfaced the same pattern in `cost.dag` and (historically) the Lane-2 workflow lenses. The common root cause was substrate gaps: v3 did not carry v2's termination/computation/induction vocabulary. **Today** `DescentEvidence`, `CallPattern`, and `SubValueRelation` (+ `CostBound`, etc.) **are staged** on `src/v3/std/termination.dag`, `src/v3/std/computation.dag`, and `src/v3/std/induction.dag` (lanes E-T / E-C / E-I). **Lane E-M closed as structural subsumption:** v3 does not need a ported `MethodSemantics` carrier because method-like dispatch no longer has an `ExprMethodCall` attachment point; it is represented structurally by `TransformTarget::{Callable, FieldProject, Operator}` plus typed declaration/effect facts (see "Method semantics receipt" below). **Lane E-P now adds the first derived per-call induction-evidence producer** as a side table (`v3_compiler::dag::per_call_descent_evidence`, implemented in `src/v3/compiler/src/dag.rs`), while `src/v3/std/substrate.dag` remains unwidened. This is a first narrow slice for recursive self-call / arithmetic-descent evidence, not full v2 `ExprCall.descent_evidence` parity. Remaining gaps for cost/complexity **behavioral** parity therefore include **producer coverage beyond this first slice, lens consumers, and cementing** (same-source v2/v3 oracle comparison, grammar/emit items below), plus (for some lanes) emit or analysis-porting gaps. Lenses that need live per-call facts still compute a structural proxy or forward to a Rust oracle until those wires land.

The pattern was invisible at the file level because "🟢 TERMINAL" reads as both structural ("this file is a pure catamorphism, fail-closed, bounded") and behavioral ("this lens does the job its name implies") — but only the first claim was ever load-bearing for the marker. This register separates the two.

## Status markers — two axes

Every `.dag` lens declares both axes in its file header.

**Structural status** — is the `.dag` file well-formed under the compiler's rules?

| Marker | Meaning |
|---|---|
| `STRUCTURALLY TERMINAL` | Pure fold / catamorphism, fail-closed on malformed refs, no fuel, no mutual recursion, no heuristics |
| `STRUCTURALLY PARTIAL` | Intentional prototype or stub; scope explicitly bounded in the file header |
| `STRUCTURALLY UNSOUND` | Known-broken; exists as a placeholder or for regression observation only |

**Behavioral status** — does the lens subsume the analysis its name implies?

| Marker | Meaning |
|---|---|
| `BEHAVIORALLY COMPLETE` | Subsumes its v2 counterpart (or, if none, genuinely computes the named analysis). Has a behavioral cementing test. |
| `BEHAVIORALLY PROXY` | Computes a weaker function of the named analysis. The name overclaims relative to what the file produces. |
| `BEHAVIORALLY STUB` | The `.dag` file dispatches to a Rust oracle; the real analysis lives elsewhere. |
| `BEHAVIORALLY N/A` | v3-native analysis with no v2 counterpart; not scoped to subsume anything. |

A lens is only "done" when **both** axes are at their strongest grade for the scope the lens claims. `STRUCTURALLY TERMINAL` alone is not done.

## Capability table

| Lens | Structural | Behavioral | v2 counterpart | v3 output | What v2 has that v3 drops |
|---|---|---|---|---|---|
| `complexity.dag` | TERMINAL | **PROXY** | `src/v2/complexity.dag` (5488L) | `v3.std.lookup::Lookup<Int> = Miss \| Hit(Int)` per port (generated consumer: `src/v3/compiler/src/lens_cost_generated.rs` via `regen_lens` / `emit_rust_module`); `src/v3/std/{termination,computation,induction}.dag` stage E-T/E-C/E-I vocabulary; E-P side-table producer `v3_compiler::dag::per_call_descent_evidence` exists for the first recursive self-call / arithmetic-descent slice; E-M method carrier parity is closed by structural subsumption — lens output unchanged until cost/complexity consumers read per-call facts | `CostExpr` (Sum/Mul/Log/Const), `SizeExpr`, work/span split, `Certainty`, asymptotic classification, recurrence bounds |
| `cost.dag` | TERMINAL | **PROXY** | v2 `CostExpr` (embedded in `complexity.dag`) | `v3.std.lookup::Lookup<SymbolicCost> = Miss \| Hit(SymbolicCost)` per port (generated consumer: `src/v3/compiler/src/lens_cost_symbolic_generated.rs` via `regen_lens` / `emit_rust_module`; Rust surface re-exports `SymbolicCostLookup` as a type alias for `Lookup<SymbolicCost>`); E-C stages `std.computation::CallPattern` / `LoweringTarget`; E-I stages `std.induction::SubValueRelation` / `CostBound`; E-P adds the first side-table `v3_compiler::dag::per_call_descent_evidence` producer; E-M method carrier parity is closed by structural subsumption. **T-CostLens-Composition α-narrow disposition** (Director-ratified at gunb-ai/gunbc#828 #issuecomment-4400772335 via PR #2171): §1.8 gates **#38** `coercion_cost_equals_complexity_by_construction` + **#39** `no_coercion_cost_dimension` are structurally satisfied at HEAD by construction (`SymbolicCost` algebra at `src/v3/std/algebra.dag:181-188` is the sole cost dimension; `Semiring<SymbolicCost>` `sequential` / `iterate` composition is the only authority — no parallel cost dimension exists). Gates **#37** `cost_lens_reads_target_realization`, **#40** `symbolic_cost_expr_equals_executable`, **#70** `cost_lens_demonstration` deferred to follow-on substrate canvas at gunb-ai/gunbc#2175 (Behavior→primitive-identity wiring; cross-cutting with T-LBP cementing tests #1950 / #1951). | Named `SizeVar` with value semantics (v3's `SizeVariable` carries only `source_port: PortId`). `Dimension<SymbolicCost>` wiring deferred on grammar gaps. **E-I carriers are present and E-P has a first narrow producer slice;** broader producer coverage, cost/complexity lens consumption, and the same-source v2/v3 cementing test remain pending. |
| `effect_enumeration.dag` | TERMINAL | **PARTIAL** | None (v3-native) | `EffectEnumerationReport { facts, coverage_gaps, redundant_reads, transaction }` derived from the five `Behavior` variants and callable type-signature shape; generated consumer target: `src/v3/compiler/src/lens_effect_enumeration_generated.rs` via `regen_lens` / `emit_rust_module` | Audit verdict is path (ii): live primitives still require ambient resource/transport metadata rather than returned-modified-resource signatures (`dsl/std/resources.dag`, `dsl/std/primitives.dag`, `dsl/extdeps/shell.dag`, `dsl/extdeps/github/auth.dag`). Full `OperationEffect` retirement, resource-threading migration, and caller-side effect-set pinning are follow-up work. |
| `idempotency.dag` | TERMINAL | **COMPLETE** | None (v3-native) | `WorkflowIdempotencyReport` via `lane2_workflow_at` + `std.effects::lane2_workflow_idempotency_report` | — (behavioral authority for the `WorkflowEffect` walk is `lane2_workflow_idempotency_report` in `src/v3/std/effects.dag`. `workflow_idempotency.rs` is still **parallel staging** for native `Dag` entry — same projection, hand mirror until an explicit dissolution removes it or routes through generated-only code; INVARIANTS P2 cleanup, not “two competing semantics.”) |
| `parallelism.dag` | PARTIAL | **STUB** | None (v3-native) | Unconditional `report_parallelism_unsupported(LensSurfacePending, …)` | — (real analysis in `src/v3/compiler/src/workflow_parallelism.rs`. **STUB reason:** Stage 2e parallelism walk not yet ported to `.dag` / `lane2_workflow_at` / `std.effects` — the `.dag` stub’s pending reason matches that gap, not the old “cannot emit `match` on imported user sums” era. `emit_rust_module` already lowers that match path; parallelism stays pending until the lens is rewired like `idempotency.dag`.) |
| `provenance.dag` | TERMINAL | COMPLETE | None (v3-native) | `Origin = NoProducer \| MissingPort \| MissingBehavior \| Source(NodeId) \| Computed(NodeId) \| Selected(NodeId) \| Accumulated(NodeId)` | N/A |
| `unused_parameters.dag` | TERMINAL | COMPLETE | None (v3-native) | `List<UnusedParameter>` | N/A |
| `variant_payload.dag` | TERMINAL | COMPLETE | None (v3-native) | `VariantPayloadShapeLookup` | N/A (see `DeclarationLookup` parallel-authority debt — ROADMAP 2026-04-21 wave, line 170 — which is an internal cleanup, not a behavioral gap) |
| `structural_resolution.dag` | TERMINAL | COMPLETE | None (v3-native) | `List<UnresolvedArrowBody>` (regression pin) | N/A |
| `infer_helpers.dag` | PARTIAL | N/A | None | `v3.std.lookup::Lookup<DeclarationId>` for `template_argument_value` (`std.substrate::{miss_declaration_id_lookup, hit_declaration_id_lookup}`; generated `infer_helpers_generated.rs` via `regen_lens`); other surface bounded Cat-1/Cat-2 (includes `template_arguments_match`, `normalize_instantiation_arguments`, etc.) | N/A (see file header for scope — `predicate_info`, `walk_to_optional_cardinality_decl`, `callable_template_arguments`, `is_retryable_generic_decl_walk`, `declaration_is_callable` stay in Rust pending substrate work) |
| `lower_helpers.dag` | PARTIAL | N/A | None | `expr_span(expr) -> SourceSpan` only | N/A (see file header — `SurfaceLiteral -> LiteralBits` and tuple-variant helper rejected on emit gaps; wire-in parked on parse/parse_surface convergence) |
| `named_function_count.dag` | TERMINAL | **N/A** | None (demo / Day-1 gate fixture) | `Int` (count of `Bind` nodes with non-empty `name`) | N/A (not in `Dag::new()` bootstrap; `compile_to_dag` integration receipt — not `regen.dag`) |

## Common root causes

The remaining “not done” rows (**complexity**, **cost**) still share **producer / analysis-porting** gaps (not missing E-I types and not missing a v3 `MethodSemantics` carrier); **parallelism** stays **STUB** until the Stage 2e `.dag` surface is wired (see table — distinct from the resolved imported-sum `match` emit lane). **idempotency** is now **COMPLETE** (see table). Fixing the shared blockers unblocks multiple lenses at once; the register exists in part so that is visible.

### Substrate carriers vs per-call producers (complexity, cost)

v2's complexity analysis imports carriers from `dsl/std/`. **E-T, E-C, and E-I have staged the mirrored families in `src/v3/std/`** — the **types** below are present for bootstrap and tests; **what still blocks genuine equivalence** is broadening E-P beyond the first self-call/arithmetic slice and wiring those facts into **live cost/complexity consumers**, plus the non-carrier items in the table rows above.

- `std.termination::DescentEvidence` (+ `RankingDimension`, `TerminationProof`, `ProofEdge`) — staged by E-T (`src/v3/std/termination.dag`)
- `std.computation::CallPattern` (+ `SizeBound`, `ShrinkFactor`, `IterationDimension`, `LoweringTarget`) — staged by E-C (`src/v3/std/computation.dag`)
- `std.induction::SubValueRelation` (+ `CostBound`, `PolynomialExponent`, …) — staged by E-I (`src/v3/std/induction.dag`)
- **E-P partial receipt:** first derived side-table producer (`v3_compiler::dag::per_call_descent_evidence`, `src/v3/compiler/src/dag.rs`) exists for the recursive self-call / arithmetic-descent slice; `TransformNode` remains unwidened pending consumer-fit evidence.

Until E-P producer coverage broadens, cost/complexity lenses consume that per-call evidence, and the result is compared against the v2 oracle on the same source, they still cannot derive symbolic bounds from termination + induction proofs **at call sites** the way v2 does, even though the **E-I vocabulary** is now load-bearing in std and the first producer exists. This is recorded in `ROADMAP.md` P2 as "four hand-rolled `BoundedLattice<T>` instances" but that entry flags the algebra-declaration gap, not the lens-consumer gap. Both are real; both are on the path to `BEHAVIORALLY COMPLETE` for complexity and cost.

### Method semantics receipt (E-M closed by structural subsumption)

v2's `MethodSemantics` was necessary because `ExprMethodCall { method_semantics }` carried a post-lookup side fact for downstream consumers:

- `PlainMethodSemantics` meant no structural algebra/service fact was found; emit fell back to receiver-method syntax.
- `AlgebraMethodSemantics { method_def, fold_accumulator_type, size_effect, cost_shape, algebra_template }` carried the resolved algebra field, fold accumulator type, and the algebra template facts complexity used for iteration, size preservation, callback element position, and method cost.
- `ServiceMethodSemantics { service_name, op_params }` carried transport/service operation identity and operation parameters.

v3 has no `ExprMethodCall` and no side field to populate. The replacement facts live at their structural authorities:

- v2 `PlainMethodSemantics` maps to ordinary `TransformTarget::Callable(DeclarationId)` dispatch or `TransformTarget::FieldProject { field_label, field_child }` when the surface form is a projection. The declaration id, input ports, and resolved child declaration are the dispatch evidence; there is no separate "plain" marker to query.
- v2 `AlgebraMethodSemantics.method_def` maps to the callable declaration id or, for operators still in the surface scaffold, `TransformTarget::Operator(OperatorKind)` plus the infer-time algebra walk that resolves the actual `std/algebra.dag` field. `fold_accumulator_type` maps to callable/lambda signature facts: v3 resolves the callable's `Arrow { inputs, output, body }`, binds callback arguments through `BindNode.params` and port states, and substitutes callable template arguments during `resolve_callable_target` instead of storing a method-call side override. `size_effect`, `cost_shape`, and `algebra_template` are algebra-declaration facts, not call-node facts; cost/complexity should read them from the resolved algebra/template declarations when the richer analysis is ported.
- v2 `ServiceMethodSemantics` maps to typed service/effect declarations and operation metadata in the substrate/effects path. Service behavior is not recovered by matching an `ExprMethodCall` tag.

Therefore a v3 `MethodSemantics` carrier would duplicate structural facts and reintroduce a side channel beside the declaration graph. E-M uses option **M-b**: no `src/v3/std/method_semantics.dag`, no Rust mirror, and no method-carrier cementing test. Cost/complexity remain **PROXY** for the output-dimension reasons in the table and for E-P producer wiring, not because `MethodSemantics` is missing.

### ~~Missing emit capability~~ (resolved for imported user-defined sums)

The v3 `emit_rust_module` path historically failed `match` on some user-defined sums when lowering introduced **anonymous** specialized `Disj` copies (`name: None` so `Dag::declaration_by_name` stays single-authority), so Rust emission could not recover the template enum label. **Receipt:** `specialize_decl_for_lowering` sets **`Declaration::specialization_parent: Some(template_disj_id)`** on those anonymous sums (`src/v3/compiler/src/dag.rs`, `src/v3/compiler/src/lower.rs`) — a typed lowering edge distinct from `meta_tag` (realization / data-type rows). Rust emit walks it via **`named_disj_enum_name_for_rust_match_emit`** (`src/v3/compiler/src/emit/rust_target.rs`). **Proof:** `m1_3_emit_rust_test::emit_rust_module_match_on_imported_workflow_effect_sum` (end-to-end `match` on an imported user sum); `lower::tests::specialize_decl_for_lowering_anonymous_disj_sets_specialization_parent` (real `specialize_decl_for_lowering` path); and `named_disj_enum_name_for_rust_match_emit_follows_specialization_parent_to_named_template` in `src/v3/compiler/src/emit/rust_target.rs` (walker-only regression on a synthetic anonymous specialized `Disj` — not `meta_tag`).

`idempotency.dag` was not blocked on this emit gap in practice (it matches `Option` and delegates `WorkflowEffect` work to `std.effects::lane2_workflow_idempotency_report`, which already lowered). The register row is updated to **COMPLETE** on that basis — meaning the **shipped lens + std.effects** path is emit-faithful for the named analysis, not that parallel Rust staging has already been deleted (see table note).

`parallelism.dag` remains a **STUB** until the Stage 2e walk is authored under `std.effects` (or the lens) and the lens stops returning `LensSurfacePending` — **not** because match-on-`WorkflowEffect` cannot emit anymore.

This is the same class of gap recorded in `ROADMAP.md` under the 2026-04-21 receipt-closure wave ("emit_rust_module gap: SurfaceLiteral → LiteralBits rename", "emit_rust_module gap: render_variant_constructor fails on external tuple variants"). Those are different emit gaps; the user-sum-match gap belonged alongside them and is now closed for the imported-sum / specialized-disj case above.

## Discipline

The register is a tool, not a document. If it is stale, it is worse than not existing — it licenses the exact "weeks of execution without a clear goal" failure mode that motivated it.

1. **Every registry entry in `src/v3/compiler/regen.dag` requires a row here.** A `.dag` lens that is generated-and-shipped but absent from this table is unreviewable.
2. **Every claim of "v3 replaces v2 X" requires a behavioral cementing test.** A claim without a test is a hypothesis. The table column "What v2 has that v3 drops" must be empty (or filled with "N/A") before the behavioral status can read `COMPLETE`.
3. **Review at every ROADMAP dispatch.** Before queueing a brief that depends on a lens being equivalent to its v2 counterpart (e.g., "compare v3 complexity output against v2 complexity output"), check the row. If the row says `PROXY` or `STUB`, the brief is premature and must either wait for substrate/emit work or narrow its scope to what the lens actually computes.
4. **Downgrades are first-class.** A lens that regresses from `COMPLETE` to `PROXY` (e.g., because a cementing test failed and was weakened rather than fixed) belongs in this register immediately. Silent downgrade is the failure mode this document exists to catch.
5. **The two axes do not trade off.** `STRUCTURALLY TERMINAL` + `BEHAVIORALLY PROXY` is a valid, honest state — a well-formed file computing a weaker function than its name implies. It is not a contradiction. Claiming `COMPLETE` because the file is structurally clean is the mistake this register is named against.
6. **Cementing tests have a canonical home in-tree.** In-repo dispatch (fixture layout, v2-oracle vs minimal-`Dag` contract tests, and the `regen.dag` → register → test-module ratchet) lives in `TESTING.md` under *Cementing tests (Band C — lens subsumption)* and in `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs`. `cementing_escalation_slice_matches_capability_register` mechanically aligns `CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS` with this table plus `regen.dag` (the slice is a projection, not a second authority). The follow-on ratchet checks both an on-disk `cementing/<stem>.rs` and a matching `#[path = ...]` line in `tests/integration.rs` for each escalated claim. Promoting a row to `BEHAVIORALLY COMPLETE` with a non-`N/A` v2 counterpart without landing the cementing module in the same PR is a process failure, not a modeling disagreement.

## Related docs

- `ROADMAP.md` — the scattered debt rows that share this root cause (2026-04-21 receipt-closure wave, P2 tracked debts on lattice instances, DB-20 workflow parallelism, complexity-receipt brief entry)
- `docs/briefs/complexity-v2-v3-comparison-receipt.md` — the in-flight brief that prompted this audit; to be rewritten against this register (scope widens from "complexity + cementing" to "Band C honesty pass")
- `INVARIANTS.md` — particularly P1 Modeling Faithfulness; a `PROXY` lens whose name implies `COMPLETE` is an authored faithfulness failure even if the file is structurally clean
