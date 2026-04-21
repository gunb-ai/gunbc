# 1e-2b — Lane 1e-2 retry: Class 5 Gap 1 OR different verified cluster `(M-XL, decision-first)`

## Context

PR #610 (Lane 1e-2 first attempt) was **closed without merge**. Cluster F ("logical operators via `LogicalOperatorCarrier`") was misclassified in `#608`'s Phase 1 audit — the proposed carrier would have been parallel realization schema over the existing `OperatorRealization` + `BooleanAlgebra.meet/join` authority, papering over a real modeling gap rather than closing it.

Per the thread verdict on #610:

> "LogicalOperatorCarrier is a parallel realization schema that papers over **Class 5 Gap 1** (Bool → BooleanAlgebra grounding via `resolve_operator_arrow` + `inhabits` edge) rather than closing it. Correct path is to reclassify Cluster F as semantically-already-covered by OperatorRealization + BooleanAlgebra.meet/join, gated on Class 5 Gap 1."

1e-2b is the retry. The worker has **a decision to make before implementing**: which of two paths.

## Decision required in the first PR commit message / body (not mid-implementation)

### Path A — Class 5 Gap 1 (Bool → BooleanAlgebra grounding) `(XL)`

Land the structural grounding: `Bool` inhabits `BooleanAlgebra<Bool>` via substrate-level `inhabits` edge; `resolve_operator_arrow` walks that edge for logical operators and resolves through `BooleanAlgebra.meet/join` / `complement`.

After Class 5 Gap 1 lands, logical-operator emission becomes trivially spec-driven via existing `OperatorRealization` rows (the ones that already handle arithmetic operators for `OrderedRing<Int>` etc.). No new carrier needed.

**This is thesis-aligned**: grounds a previously-ungrounded Bool-logical-op dispatch in the existing algebra framework, closing the modeling gap rather than routing around it.

### Path B — Different verified Lane 1e-2 cluster `(M)`

Pick a different Lane 1e-2 cluster from `docs/emit-target-spec-gaps.md` that (1) survives Phase 0 verification (existing authorities do NOT already cover it) and (2) is a pure spec-data gap, not a modeling gap.

**Known mis-classifications to skip** (6 confirmed audit errors in #608):
- Cluster A — Go type recursion (already covered by `TypeInstantiationRealization.carrier` + `TypeApplicationSyntax`)
- Cluster B — execution model (already covered by `TargetExecutionModel.memory`)
- Cluster C — bootstrap filtering (already covered by `is_bootstrap_file` behavior)
- **Cluster E — optional type wrappers (already covered by `TypeApplicationSyntax.optional` in `emit_model.dag`; e.g., `go_type_applications.optional: "*{element}"`)**
- Cluster F — logical ops (needs Class 5 Gap 1 / Bool grounding, not a new carrier)
- Cluster H — unused pattern binding (already covered by `CleanEmissionContract.pattern_bindings: PatternBindingRule`)

**Not yet contested** (candidates — worker verifies before committing): none from the current `#608` list are high-confidence. May require reading `#608` critically + re-checking each cluster against live authorities.

Phase 0 verification is **mandatory** — grep every proposed row against `src/v3/spec/*.dag` + `src/v3/std/*.dag` before implementing.

## Read first

- My closing comment on #610 — the reclassification rationale
- `docs/emit-target-spec-gaps.md` — treat as flawed audit; verify each classification before use
- `dsl/std/algebra.dag:230-236` — `BooleanAlgebra<T>` definition; comments at 44-51 say explicitly *"Bool inhabits BooleanAlgebra"*
- `src/v3/compiler/src/infer.rs` — `resolve_operator_arrow` (grep for it)
- `src/v3/std/substrate.dag` — `AtomPayload::ResolvedByStructure(DeclarationId)` and `inhabits` edge plumbing
- `src/v3/compiler/operators.dag` — `OperatorKind::Logical(LogicalOp)` definition
- `src/v3/compiler/src/emit/rust_target.rs:3312` + `emit/python_target.rs:783` — current Logical-op dispatch (hardcoded `if let OperatorKind::Logical(_)` branches that bypass algebra-field resolution)

## Work (Path A)

1. **Add `inhabits` edge** from `Bool` to `BooleanAlgebra<Bool>` in `dsl/std/bool.dag` (or wherever `Bool` is declared). Match the pattern used for `Int inhabits OrderedRing<Int>` (if such pattern exists — verify first).

2. **Extend `resolve_operator_arrow`** in `infer.rs` to walk the inhabits edge for `OperatorKind::Logical(_)` and resolve through `BooleanAlgebra.meet` / `BooleanAlgebra.join` / `BooleanAlgebra.complement` declarations.

3. **Verify `OperatorRealization`** (in `emit_model.dag` / per-target spec) has rows covering the BooleanAlgebra.* field-declaration ids for all three targets. If missing, add them — but this is a small data addition, not a new carrier.

4. **Delete the hardcoded `if let OperatorKind::Logical(_)` branches** in `emit/rust_target.rs` and `emit/python_target.rs` and (if applicable) the Go inline emitter in `emit.rs`. Logical operators now dispatch through the same `OperatorRealization` pathway as arithmetic operators.

5. **Regression tests**: add an emit test per target asserting `&&` / `and` / `&&` rendering correctness. Existing tests should pass unchanged.

## Work (Path B)

1. **Phase 0 verification** — grep every proposed Category 2 spec row from `#608` against live authority files. For each, verify it's genuinely uncovered. Discard any that are covered.
2. Pick ONE remaining verified cluster with smallest scope.
3. Follow the Lane 1e-2 template from the original brief: spec row + current consumer site + matching test.
4. Do not exceed one cluster.

## Acceptance

### Path A
- `Bool inhabits BooleanAlgebra<Bool>` is a substrate-level fact (declaration + edge)
- `resolve_operator_arrow` resolves `OperatorKind::Logical(And/Or/Not)` through `BooleanAlgebra.meet` / `.join` / `.complement` declarations
- Hardcoded `OperatorKind::Logical` branches in the three emitters **deleted** — logical ops dispatch through the shared `OperatorRealization` path
- Emit test per target for `&&` / `and` / `&&` confirms correct rendering
- `#610`'s original goal (data-driven logical-op rendering) achieved via existing authority instead of new carrier

### Path B
- PR body declares "Path B selected; Class 5 Gap 1 deferred" and names the alternate cluster chosen
- Phase 0 verification report in PR body (every candidate cluster checked against live spec)
- One cluster's spec rows landed with current consumer wiring
- No new parallel authority introduced

## STOP-AND-ESCALATE

### Path A
- **If `inhabits`-edge plumbing doesn't exist for any type currently** — STOP. This becomes a substrate-extension lane, not 1e-2b. Surface and name the substrate work.
- **If `resolve_operator_arrow` can't cleanly walk the inhabits edge without major inference restructuring** — STOP. May overlap with SG-4b scope.
- **If BooleanAlgebra.meet/join/complement aren't declared at the algebra level** — STOP. May need algebra-side work first.

### Path B
- **If Phase 0 verification finds ALL remaining Category 2 clusters are already covered or misclassified** — STOP and report. Means `#608` is more broken than thought; 1e-2b is blocked until a re-audit happens.
- **If the chosen cluster requires a modeling change (not just a data row)** — STOP. 1e-2b is spec-data extension; modeling changes belong in other lanes.

## Non-goals

- **Not both paths** — pick one per PR
- **Not rewriting `#608`** — Path B gets verification within 1e-2b; full re-audit is separate
- **Not walker implementation** — 1e-2b is still Phase 2 (spec gap closure); walker work is Lane 1e Phase 3+
- **Not Class 5 Gaps 2-N** — if Path A, scope is Gap 1 only

## Size

- **Path A**: XL. Substrate `inhabits` edge + inference resolution + emitter simplification + tests. Thesis-aligned; larger surface.
- **Path B**: M. Single-cluster data extension; Phase 0 verification is read-heavy but implementation is small.

## Dispatch note

**Worker must declare path in first commit message.** Director reviews the decision artifact before implementation proceeds far.

**Director preference**: Path A if possible — Bool → BooleanAlgebra grounding is a load-bearing thesis move ("every concept grounds in primitives"). Path B is acceptable as an incremental alternative if Path A reveals substrate-extension scope that warrants a dedicated lane.

Either path satisfies the 1e-2b slot in the next-wave queue. The wrong outcome is another misclassified carrier like #610 — Phase 0 verification is non-negotiable regardless of path.

## Tracked debt update on merge

On landing (either path):
- Update ROADMAP Class 5 Gap 1 entry (open one if Path B is chosen and Gap 1 is deferred)
- Update `docs/emit-target-spec-gaps.md` — flag Cluster F as reclassified to Category 1 (if Path A) or note the picked cluster and any verification findings (if Path B)
