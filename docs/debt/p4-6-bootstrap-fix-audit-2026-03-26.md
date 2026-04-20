### P4.6 bootstrap fix audit (2026-03-26)

The `bootstrap_stage0_to_stage1` test was failing with 147 cargo check
errors in emitted stage1 Rust. All 147 were fixed in PR #212. This
audit classifies each fix as either a **true root-cause fix** or a
**workaround** that papers over a deeper invariant violation.

#### True root-cause fixes (no invariant debt)

These fixes corrected genuine bugs at the right layer. No follow-up needed.

| Fix | Why it's correct |
|-----|-----------------|
| Named record literals in `languages.dag` (`ReservedWords`, `ProjectScaffold`, `SerializationSpec`, `TestConventions`) | Anonymous `{ field: value }` syntax IS a tuple in the .dag language. Using `TypeName { field: value }` is the correct way to construct named structs. Source-code bug, not an invariant issue. |
| Missing imports (`UnaryOpKind` in `05_emit.dag`, `InterpPart` in `05_emit_rust.dag`, `is_typed_service_call_receiver`/`extract_typed_service_name` in `05_emit.dag`) | Imports were genuinely missing after file decomposition. Correct layer for the fix. |
| `map_expr_children` param name `node:` → `expr_node:` in `04_resolve.dag` | Call site used wrong parameter name, causing emitter to output arguments in wrong positional order. Naming bug at the call site. |
| `return;,` syntax → let+return pattern in `05_emit_rust.dag` | `.dag` `return` inside match arms generates `return expr;,` in Rust (semicolon + comma). Restructuring to `let result = match { ... }; return result` avoids the issue at the .dag source level. |
| `adjacency_add_edge` helper in `03_resolve.dag` | Extracts fold body into a function with explicit `Map<String, List<String>>` parameter types. Gives inference the information it needs without fabrication. Honest .dag-level fix. |

#### Workarounds (invariant debt — needs follow-up)

| # | Fix | Invariant violated | Root cause | Deletion point |
|---|-----|-------------------|------------|----------------|
| IV-6 | `empty_map()` → `BTreeMap::new()` in `emit_typed_call_expr` (`05_emit_rust.dag:1835`) | **No fallbacks that fabricate.** Emit silently drops the turbofish and hopes Rust's type inference recovers the value type from context. If Rust can't infer, this produces a different error (E0282) instead of the correct type. | Inference does not propagate expected parameter types to argument expressions. `empty_map()` as an argument to `f(rc_types: Map<String, Bool>)` should infer `Map<String, Bool>`, not `Map<String, Unit>`. **Bidirectional type inference is missing.** | Fix inference to propagate expected types from function signatures to argument expressions. Then emit can use the turbofish with the correct type. Extends IV-1/IV-2. |
| IV-7 | Fold init `empty_map()` with unit-child detection (`05_emit_rust.dag:2302-2310`) | **No fallbacks that fabricate** + **Heuristics indicate lost structure.** Emit inspects the acc type node's children for `"Unit"` or `""` names to decide whether to use turbofish or partial `<BTreeMap<String, _>>::new()`. This is a heuristic that compensates for inference producing incomplete types. | Same as IV-6: inference doesn't resolve fold accumulator type parameters from the fold body. The `acc_type_node` carries `Map<String, Map<String, Unit>>` when the fold body clearly produces `Map<String, List<String>>`. | Fix inference to propagate fold body return type back to the accumulator type. Then emit receives complete types and the heuristic is unnecessary. |
| IV-8 | Fold acc type resolution with unit-child fallback to contextual type (`05_emit_rust.dag:2277-2284`) | **Heuristics indicate lost structure.** Emit checks `acc_type.children |> any(c => c.name == "Unit")` to decide whether to use the contextual (method result) type instead of the inferred accumulator type. | Same root cause as IV-6/IV-7. The emit layer is doing type resolution work that belongs in inference. | Same deletion point as IV-7. |
| IV-9 | `go_source_extension` → inline literal `".go"` in `languages.dag:163` | **No duplicate representations.** The value `".go"` is now defined in both `dsl/extdeps/languages/go/emit.dag:65` (as `data go_source_extension`) and inline in `languages.dag`. They will diverge if either changes. | The emitter inconsistently transforms `data` constant names to SCREAMING_SNAKE_CASE in import `use` statements. 6/7 Go extdep data constants are correctly uppercased; `go_source_extension` is not. Import emission doesn't distinguish function imports (stay snake_case) from data constant imports (should be SCREAMING_SNAKE). | Fix the import emission in `05_emit_rust.dag` to consistently apply SCREAMING_SNAKE_CASE for `data` constant imports. Then restore the import in `languages.dag` and delete the inline literal. |

#### Underlying root cause: no bidirectional type inference

IV-6, IV-7, and IV-8 all trace to the same root cause: **inference is
top-down only.** It resolves types from declarations and expressions
forward, but does not propagate expected types backward from:

- Function parameter signatures to argument expressions
- Fold accumulator usage in the body back to the init expression
- Let-binding type annotations back to the initializer

This is not a new finding — IV-1/IV-2 (2026-03-25) already identified
the incomplete container types. The P4.6 fixes expose the same root
cause at 124+ additional sites (every `empty_map()` call where the
value type is unresolved).

**Scope:** This is a Phase 5+ fix (inference architecture). The current
workarounds are viable because Rust's own type inference recovers the
correct types in all 124+ sites. But they are fabrications: emit
produces `BTreeMap::new()` instead of `<BTreeMap<String, bool>>::new()`,
relying on a downstream system (rustc) to compensate for information
the pipeline lost.

#### Return-in-match-arm emitter bug (not fixed, worked around)

The `return;,` fix restructured the .dag source to avoid `return` in
match arms, but the underlying emitter bug remains: when a `.dag`
`return` statement appears as a match arm body, the emitter generates
`return expr;,` (semicolon from statement termination + comma from match
arm separation). Any future .dag code using `return` inside match arms
will hit the same issue.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-10 | LOW | Emitter match-arm rendering | `return` in match arm body emits `return expr;,` — Rust syntax error. Workaround: use let+return pattern. Fix: emitter should suppress trailing `;` when the match arm body is a return/break/continue. |

---

