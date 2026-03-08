# TODO

Technical debt identified during the generated test fix-up (2026-03-08).
Each item has enough context to be picked up cold.

---

## T1: Deterministic fan-in ordering in executor

**Priority: High**
**Files:** `src/core/exec/src/execute/mod.rs`

The executor processes fan-in edges (multiple edges targeting the same list port) in topological-sort order. The topological sort is non-deterministic for nodes at the same depth — it depends on HashMap iteration order and node insertion order. This means `Value::List` outputs built from fan-in have unstable element ordering across runs or between a full DAG and a subDAG extraction.

**Current workaround:** `assert_window_outputs` in `src/core/test/src/window.rs` accepts any matching-length list instead of checking value equality. This is extremely permissive.

**Proper fix:** When the executor collects fan-in values for a list port, sort by a deterministic key before constructing the `Value::List`. Candidates:
- Edge index (already stored in `Edge.index`, but currently defaulting to 0 for all edges)
- `(from_node, from_port)` lexicographic order
- Assign monotonic indices during lowering so edges carry stable ordering

After fixing, revert the matching-length comparison in `window.rs` back to exact equality (or multiset comparison).

**Test:** The 32 window tests in `tools_build` and `workflows_build_all` that go through `expr_compute_*_stages_arg_*` nodes — these currently pass only because of the lenient comparison.

---

## T2: Secret as first-class ValueBacking

**Priority: Medium**
**Files:** `src/core/ir/src/types.rs`, `src/core/test/src/auto_mock.rs`, `src/core/test/src/mock_spec.rs`, `src/core/codegen/src/testgen/codegen.rs`

`ValueBacking` has no `Secret` variant. `value_backing_for_type_id("Secret")` returns `ValueBacking::String`, so the auto-mock generates `OutputMatcher::IsString` for Secret-typed ports. But the runtime value is `Value::Secret(...)`, not `Value::Str(...)`.

**Current workaround:** Two places paper over this:
1. `OutputMatcher::IsString` check in `mock_spec.rs` accepts `Value::Secret(_)` alongside `Value::Str(_)`
2. The codegen renders `matches!(*v, Value::Skipped | Value::Secret(_))` in the IsString assertion

**Proper fix:**
1. Add `ValueBacking::Secret` to the enum
2. Register it in the type registry for `TypeId("Secret")`
3. Add `OutputMatcher::IsSecret` variant
4. Auto-mock emits `IsSecret` for Secret-typed ports
5. Remove the Secret accommodations from IsString
6. Consider whether the DSL coercion system should handle String->Secret upcast (safe) and Secret->String downcast (explicit only)

---

## T3: Testgen-side filter for unevaluable fn bodies

**Priority: Medium**
**Files:** `src/core/codegen/src/testgen/codegen.rs`, `src/core/resolve/src/resolve.rs`

Some fn bodies in `.dag` files contain service calls (e.g., `llm.Anthropic.Messages()` in `design.dag`). The `ExprComputeOp` evaluator can't execute these — it hits "unknown function" errors.

**Current workaround:** `ExprComputeOp::execute()` in `resolve.rs` catches errors containing "unknown function" and degrades to `Value::Skipped` instead of returning an error. This silently swallows real errors alongside the expected ones.

**Proper fix:** Detect unevaluable fn bodies at testgen time and skip generating example tests for those nodes. Detection approach:
- During lowering, `ExprComputeOp` already stores the fn body AST
- Walk the AST for service call expressions (identifiable by module-qualified names like `llm.Anthropic.Messages`)
- If found, mark the node as unevaluable in the `DagAnalysis`
- Testgen skips example/chain tests for unevaluable nodes

Then remove the silent "unknown function" fallback in `ExprComputeOp::execute()` so the executor stays strict.

---

## T4: Scalar fan-in semantics for conditional branches

**Priority: Low**
**Files:** `src/core/exec/src/execute/mod.rs`

When conditional branches (if/match lowered to CondBranch) produce edges to the same scalar port, the executor now accepts multiple upstream edges and takes the first non-Skipped value. Previously it rejected this as a "duplicate scalar port" error.

**Current behavior:** In both sequential and parallel executors (~line 655 and ~line 1250), when a scalar port already has a source and a new edge arrives:
- If the new value is non-Skipped, it overwrites the existing value
- If the new value is Skipped, the existing value is kept

This works for the current pattern (exactly one branch fires, others produce Skipped) but is ad-hoc.

**Proper fix:** Introduce a formal `ConditionalMerge` node kind in the IR that explicitly merges conditional branch outputs. The lowerer would emit this node at the branch join point. The executor would have well-defined semantics: exactly one non-Skipped input expected, error if zero or 2+. This makes the behavior visible in the DAG structure rather than implicit in the executor's edge processing.

---

## T5: Literal source terminal example generation

**Priority: Low**
**Files:** `src/core/test/src/auto_mock.rs`

`literal_source_*` nodes (created by the lowerer for fn body literal expressions) are skipped from terminal example generation because their fn body evaluation can produce empty values that fail the `NonEmpty` fallback matcher.

**Current workaround:** `auto_mock_spec` skips all nodes whose ID starts with `literal_source_` when generating terminal node examples.

**Proper fix:** Instead of skipping by name prefix, the auto-mock should trial-execute the node with its example inputs and verify matchers pass before emitting the example. The previous attempt at this (blanket trial validation) was too aggressive — it removed ~360 valid examples because many trial executions fail for unrelated reasons (missing context, upstream dependencies). A more targeted approach:
- Only trial-validate when the fallback `NonEmpty` matcher is used (not typed matchers like IsBool/IsInt)
- Accept the example if the trial execution fails (the test may work differently)
- Only skip if the trial succeeds AND the matcher explicitly fails on the produced value
