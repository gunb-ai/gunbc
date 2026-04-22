# v3 Lens Capability Register

Canonical view of what every `.dag` lens under `src/v3/lenses/` actually computes, relative to its v2 counterpart (where one exists) and relative to the analysis its name implies. Load-bearing for direction review: any claim of the form "v3 replaces v2 X" must be checkable against a row in this table.

## Why this document exists

During the 2026-04-21 audit of the complexity lens we found that `src/v3/lenses/complexity.dag` (162 lines, marked 🟢 TERMINAL) does not subsume v2's complexity analysis. v2's `ComplexitySummary { work, span, output_size, certainty }` carries symbolic cost expressions, size variables, work/span separation, and asymptotic classification. v3's `cost_of(dag, port) -> CostLookup = FoundCost(Int) | MissingCost` carries a single integer depth per port and drops every other dimension. The 13× LOC reduction claim was comparing different functions of the program, not equivalent ones.

Extending the audit to the other nine lenses under `src/v3/lenses/` surfaced the same pattern in three more files (`cost.dag`, `idempotency.dag`, `parallelism.dag`). The common root cause is substrate and emit gaps — the v3 substrate does not yet carry `DescentEvidence`, `CallPattern`, `SubValueRelation`, or `MethodSemantics`, and the v3 emitter cannot yet lower `match` on user-defined sums. So lenses that depend on those facts either compute a structural proxy or forward to a Rust oracle.

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
| `complexity.dag` | TERMINAL | **PROXY** | `src/v2/complexity.dag` (5488L) | `CostLookup = FoundCost(Int) \| MissingCost` per port | `CostExpr` (Sum/Mul/Log/Const), `SizeExpr`, work/span split, `Certainty`, asymptotic classification, recurrence bounds |
| `cost.dag` | TERMINAL | **PROXY** | v2 `CostExpr` (embedded in `complexity.dag`) | `SymbolicCostLookup = FoundCost(SymbolicCost) \| MissingCost` per port | Named `SizeVar` with value semantics (v3's `SizeVariable` carries only `source_port: PortId`). `Dimension<SymbolicCost>` wiring deferred on grammar gaps. No cementing test. |
| `idempotency.dag` | PARTIAL | **STUB** | None (v3-native) | `WorkflowIdempotencyReport` via dispatch | — (analysis lives in `src/v3/compiler/src/workflow_idempotency.rs`; the `.dag` is a registration shim) |
| `parallelism.dag` | PARTIAL | **STUB** | None (v3-native) | Unconditional `report_parallelism_unsupported(LensSurfacePending, …)` | — (real analysis in `src/v3/compiler/src/workflow_parallelism.rs`; the `.dag` is a fail-closed placeholder) |
| `provenance.dag` | TERMINAL | COMPLETE | None (v3-native) | `Origin = NoProducer \| MissingPort \| MissingBehavior \| Source(NodeId) \| Computed(NodeId) \| Selected(NodeId) \| Accumulated(NodeId)` | N/A |
| `unused_parameters.dag` | TERMINAL | COMPLETE | None (v3-native) | `List<UnusedParameter>` | N/A |
| `variant_payload.dag` | TERMINAL | COMPLETE | None (v3-native) | `VariantPayloadShapeLookup` | N/A (see `DeclarationLookup` parallel-authority debt — ROADMAP 2026-04-21 wave, line 170 — which is an internal cleanup, not a behavioral gap) |
| `structural_resolution.dag` | TERMINAL | COMPLETE | None (v3-native) | `List<UnresolvedArrowBody>` (regression pin) | N/A |
| `infer_helpers.dag` | PARTIAL | N/A | None | Bounded Cat-1/Cat-2 infer helpers only | N/A (see file header for scope — `predicate_info`, `walk_to_optional_cardinality_decl`, `callable_template_arguments`, `is_retryable_generic_decl_walk`, `declaration_is_callable`, `template_arguments_match` stay in Rust pending substrate work) |
| `lower_helpers.dag` | PARTIAL | N/A | None | `expr_span(expr) -> SourceSpan` only | N/A (see file header — `SurfaceLiteral -> LiteralBits` and tuple-variant helper rejected on emit gaps; wire-in parked on parse/parse_surface convergence) |

## Common root causes

The four "not done" rows (complexity, cost, idempotency, parallelism) share two blockers. Fixing these unblocks multiple lenses at once; the register exists in part so that is visible.

### Missing substrate carriers (blocks genuine equivalence for complexity, cost)

v2's complexity analysis imports, from `dsl/std/`:

- `std.termination::DescentEvidence` (+ `RankingDimension`, `TerminationProof`, `ProofEdge`)
- `std.computation::CallPattern` (+ `IterationDimension`, `LoweringTarget`)
- `std.induction::SubValueRelation` (+ `CostBound`, `ShrinkFactor`, `PolynomialExponent`)
- `MethodSemantics` / `AlgebraMethodSemantics` (on `ExprMethodCall`)

The v3 substrate (`src/v3/std/`) does not carry these. Until it does, v3 lenses over cost or complexity cannot derive symbolic bounds from termination + induction proofs the way v2 does; they can only walk bare DAG topology. This is recorded in `ROADMAP.md` P2 as "four hand-rolled `BoundedLattice<T>` instances" but that entry flags the algebra-declaration gap, not the lens-consumer gap. Both are real; both are on the path to `BEHAVIORALLY COMPLETE` for complexity and cost.

### Missing emit capability (blocks `.dag` authority for idempotency, parallelism)

The v3 `emit_rust_module` cannot yet lower `match` on user-defined sums. That means a lens body that pattern-matches `WorkflowEffect` / `OperationEffect` variants cannot be the authoritative implementation — the emit would fail. So `idempotency.dag` is a thin dispatch-over-`lane2_workflow_at` shim and `parallelism.dag` is a hard `LensSurfacePending` stub. Both file headers name the specific blocker.

This is the same class of gap recorded in `ROADMAP.md` under the 2026-04-21 receipt-closure wave ("emit_rust_module gap: SurfaceLiteral → LiteralBits rename", "emit_rust_module gap: render_variant_constructor fails on external tuple variants"). Those are different emit gaps; the user-sum-match gap belongs alongside them.

## Discipline

The register is a tool, not a document. If it is stale, it is worse than not existing — it licenses the exact "weeks of execution without a clear goal" failure mode that motivated it.

1. **Every registry entry in `src/v3/compiler/regen.dag` requires a row here.** A `.dag` lens that is generated-and-shipped but absent from this table is unreviewable.
2. **Every claim of "v3 replaces v2 X" requires a behavioral cementing test.** A claim without a test is a hypothesis. The table column "What v2 has that v3 drops" must be empty (or filled with "N/A") before the behavioral status can read `COMPLETE`.
3. **Review at every ROADMAP dispatch.** Before queueing a brief that depends on a lens being equivalent to its v2 counterpart (e.g., "compare v3 complexity output against v2 complexity output"), check the row. If the row says `PROXY` or `STUB`, the brief is premature and must either wait for substrate/emit work or narrow its scope to what the lens actually computes.
4. **Downgrades are first-class.** A lens that regresses from `COMPLETE` to `PROXY` (e.g., because a cementing test failed and was weakened rather than fixed) belongs in this register immediately. Silent downgrade is the failure mode this document exists to catch.
5. **The two axes do not trade off.** `STRUCTURALLY TERMINAL` + `BEHAVIORALLY PROXY` is a valid, honest state — a well-formed file computing a weaker function than its name implies. It is not a contradiction. Claiming `COMPLETE` because the file is structurally clean is the mistake this register is named against.
6. **Cementing tests have a canonical home in-tree.** In-repo dispatch (fixture layout, v2-oracle vs minimal-`Dag` contract tests, and the `regen.dag` → register → test-module ratchet) lives in `TESTING.md` under *Cementing tests (Band C — lens subsumption)* and in `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs`. The ratchet checks both an on-disk `cementing/<stem>.rs` and a matching `#[path = ...]` line in `tests/integration.rs` for each escalated claim. Promoting a row to `BEHAVIORALLY COMPLETE` with a non-`N/A` v2 counterpart without landing the cementing module in the same PR is a process failure, not a modeling disagreement.

## Related docs

- `ROADMAP.md` — the scattered debt rows that share this root cause (2026-04-21 receipt-closure wave, P2 tracked debts on lattice instances, DB-20 workflow parallelism, complexity-receipt brief entry)
- `docs/briefs/complexity-v2-v3-comparison-receipt.md` — the in-flight brief that prompted this audit; to be rewritten against this register (scope widens from "complexity + cementing" to "Band C honesty pass")
- `INVARIANTS.md` — particularly P1 Modeling Faithfulness; a `PROXY` lens whose name implies `COMPLETE` is an authored faithfulness failure even if the file is structurally clean
