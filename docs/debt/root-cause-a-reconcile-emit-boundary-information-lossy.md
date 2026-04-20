### Root Cause A: Reconcile→Emit Boundary is Information-Lossy (ADDRESSED)

**Status:** Design decision made, infrastructure landed. Gradual migration underway.

**Design decision (2026-03-21):** Split into two categories:

1. **Reconcile resolution bugs (A-4, A-5, A-8, A-9):** Reconcile fails to resolve
   facts it should. Fix: improve resolution, add `RefKind` and `ParamSource` types.

2. **Emit rendering decisions (A-1, A-2, A-3, A-6, A-7, A-10):** Emit owns these
   decisions but must compute them efficiently. Fix: `EmitContext` struct with 6
   cached indexes built once per emit call, O(1) lookups per expression. No
   precomputation in reconcile — rendering decisions stay with the renderer.

**Infrastructure landed:**
- `EmitContext` type + `build_emit_context` + `ctx_*` helpers in `05_emit.dag`
- `RefKind`, `ParamSource` types in `04_reconcile.dag`
- `build_intrinsic_index`, `build_primitive_set` pre-built at emit entry
- EmitContext wired into `emit_rust` entry point

**Remaining:** Migrate emit functions from individual map params to `EmitContext`
lookups. Mechanical — each function gets `ctx: EmitContext` parameter, replaces
ad-hoc scans with `ctx_*` helpers.

| # | What reconcile computes | Where it's lost | How emit compensates |
|---|------------------------|-----------------|---------------------|
| A-1 | Field access style (StoredField / EnumAccessor / OptionalUnwrap) — `build_field_summaries_*` at `04_reconcile.dag:1070-1175` | Not attached to ExprFieldAccess nodes | `emit_typed_field_access` calls `lookup_emit_field_summary_in_scope` at codegen time (redundant); `is_likely_optional_receiver` scans all type_summaries; `is_optional_field_in_any_type` / `is_enum_accessor_in_any_type` do global sweeps (`05_emit_rust.dag:1576-1601`) |
| A-2 | Known-method classification + result type — `resolve_known_method_node` in `04_reconcile.dag` | `ExprMethodCall` now carries `method_semantics`; remaining loss is that renderer leaf helpers still branch on `method` strings for target syntax | Complexity no longer compensates. Emit still has per-target method-name ladders and runtime helper tables. |
| A-3 | Call→MethodCall bridging — ExprCall handler rewrites bridged calls to `ExprMethodCall` | No longer lost after reconcile; bridged calls remain structurally distinct downstream | Emit no longer needs to rediscover bridged method shape, but Rust still carries target-specific runtime helper maps for ownership/rendering. |
| A-4 | Function-as-value reference — `lookup_in_scope` fallback to `lookup_func_sig` at `04_reconcile.dag:751-754` | ExprVar node gets return type only; callable-vs-value distinction lost | Emit cannot distinguish function reference from local binding (SB-1). Fabricates value type from callable's return type. |
| A-5 | Fold accumulator type — computed during method resolution | No longer lost on typed method nodes; carried in `IntrinsicMethodSemantics.fold_accumulator_type` | Downstream consumers can read it from `method_semantics`; remaining work is deleting renderer-local fallbacks. |
| A-6 | Rc-wrapping requirement — derivable from type summaries and scope types | Not attached per expression; Rust emit still re-derives it from a module-local `rc_types` map plus Rust-local match analysis | Emit now centralizes match probing through `RcPatternAnalysis`/`RcMatchAnalysis`; lookup-specific wrapping on data maps remains separate |
| A-7 | Variant→parent enum mapping — resolved during type resolution | Only available via global `vtoe` map, not per-expression | Emit builds module-local vtoe disambiguation (`05_emit_rust.dag:430-467`); `emit_var_ref` does fallback lookup (line 1508) |
| A-8 | Dynamic/error type propagation — `node_is_dynamic` at `04_reconcile.dag:900` | Error state encoded as `string_contains("<error:")` in type name | Emit replicates check at `05_emit_rust.dag:1473`; `node_type_equals` treats Dynamic as universally compatible (SB-2) |
| A-9 | Lambda parameter types — unresolved when collection type is Dynamic | Bound to `Dynamic` in `extend_scope_for_lambda` (`05_emit_rust.dag:1959`) | Auto-wrap disabled entirely (`let needs_wrap = false` at line 2445) because `is_already_optional` can't detect Optional inside Dynamic-typed lambdas |
| A-10 | Primitive/collection type identity — structurally known | Only available as type name strings | Emit hardcodes `"Int"`, `"Bool"`, `"Float"`, `"List"`, `"Map"`, `"Set"`, `"String"` in name-matching functions (`05_emit_rust.dag:1145-1150`, `882-908`, `1488-1494`) |

**Previously tracked as:** F6, F7, SB-1, SB-2

---

