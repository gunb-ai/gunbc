# v2 Review Follow-up Plan

Triage of review feedback from the v2 grinding phase. Organized into
waves by dependency order and blast radius.

## Current Status (2026-03-11)

**48/48 tests passing. 0 clippy warnings. Full workspace green.**

Completed waves/items this batch:
- **Wave 2** (AST model): DONE — TypeApp, RecordLit type_name, FieldBinding,
  NullCoalesce in BinOpKind, PipeArrow in TokenKind (Units 1-2)
- **Wave 3** (typechecker bugs): DONE — mutual recursion cycle detection
  via `resolving` parameter (Unit 3)
- **Wave 4** (service/mock_response): PARTIALLY DONE — response/mock_response
  block parsing implemented (Unit 2). Emitted test invocation still deferred.
- **Wave 5** (docs): updating now
- **Wave 8** (pipeline completeness): PARTIALLY DONE — resolver diagnostic
  threading verified correct (Unit 5), Cargo.toml emission added (Unit 4)
- **List concat bug**: FIXED (Unit 6)
- **Bootstrap driver**: created at src/v2/bootstrap/ (Unit 7)

Deferred: `provides` clause, `from "key"` extraction, Wave 7 Option
normalization (post-merge), Phase 1c native bootstrap, Phase 2/3.

## Previous state (post-grinding-phase)

Working end-to-end chain through the v1 evaluator:
- Tokenizer: full E2E (22 tests)
- Parser: tokenize → parse chain on `"module test"` (Phase 3 test)
- Evaluator: enumerate first/second, first/last Option wrapping,
  structural Option matching, map/filter fn refs, fold named args

Not yet exercised: resolver → typechecker → emitter chain through
the evaluator. The .dag files exist and parse, but haven't been
evaluated as a pipeline.

---

## Wave 1: Unify cross-stage types

**Problem:** The three inner stages define incompatible versions of
their shared types:

| Type | resolve.dag | typecheck.dag | emit.dag |
|------|------------|---------------|----------|
| ModuleGraph | `{ modules: List<ResolvedModule>, diagnostics }` | `{ modules: List<ResolvedModule>, dep_order: List<String> }` | — |
| TypedGraph | — | `{ modules: List<TypedModule>, diagnostics }` | `{ modules: List<TypedModule>, diagnostics }` |
| TypedModule | — | `{ module, type_env: TypeEnv }` | `{ module, resolved_types: Map<String, TypeExpr> }` |
| ResolvedModule | `{ module, resolved_imports, dep_order: Int }` | `{ module, resolved_imports }` (no dep_order) | — |

**Fix:** Pick one canonical definition per type. The resolve.dag
definitions are the most complete. Typecheck and emit should import
from resolve (once cross-module evaluation works) or duplicate the
exact same structure.

Concrete changes:
1. typecheck.dag: change `ModuleGraph` to match resolve.dag's
   `{ modules, diagnostics }` + per-module `dep_order`
2. emit.dag: change `TypedModule.resolved_types` → `type_env: TypeEnv`
   to match typecheck.dag
3. typecheck.dag `typecheck()`: iterate `graph.modules` sorted by
   `dep_order` (per-module field) instead of `graph.dep_order` list
4. pipeline.dag: verify it can wire resolve → typecheck → emit with
   aligned types

**Risk:** Low — these are type structure changes in .dag files only.
No evaluator or v1 code changes needed.

**Blocked by:** Nothing. Can start now.

---

## Wave 2: Strengthen core.dag AST model — DONE

**Problem:** The v2 core AST can't faithfully represent constructs
that the v2 compiler sources actually use.

### 2a: Named record/variant constructors in expressions

`RecordLit { fields }` has no type name, so `TokenizerState { ... }`
and `Unknown { char: ch }` lose their constructor identity. The v1
parser produces these as `Expr::Record(Some("TokenizerState"), fields)`
but the v2 AST has no slot for the name.

**Fix:** Add `type_name: String?` to `RecordLit`:
```dag
| RecordLit { type_name: String?, fields: List<FieldInit>, span }
```

### 2b: Named field bindings in variant patterns

`VariantPattern { name, bindings: List<String> }` loses field names.
`Some { value: kind }` becomes `["kind"]` — works only because the
binding name equals the field name. Fails for renaming or wildcards.

**Fix:** Change to `List<FieldBinding>`:
```dag
type FieldBinding { field_name: String, binding: MatchPattern }
| VariantPattern { name: String, field_bindings: List<FieldBinding> }
```

### 2c: Block expressions and statement layer

`Expr` has no `Return` variant or real statement layer. `parse_return`
returns the inner expression, losing early return semantics. Multi-
statement blocks are flat expression lists.

**Fix:** Add `Return` and `Stmt` to the AST:
```dag
type Stmt
  = LetStmt { name: String, value: Expr, span: SourceSpan }
  | ExprStmt { expr: Expr, span: SourceSpan }
  | ReturnStmt { value: Expr, span: SourceSpan }

// Block already exists — just needs to use Stmt:
| Block { stmts: List<Stmt>, span: SourceSpan }
```

### 2d: Type application node

`TypeExpr` has no general type application — only hard-coded
`Container` and `MapType`. Can't represent `Foo<A>` or `Foo<A,B>`.

**Fix:** Add `TypeApp`:
```dag
| TypeApp { name: String, args: List<TypeExpr>, span: SourceSpan }
```
Container and MapType become sugar over TypeApp.

### 2e: Domain predicate

DIRECTION.md C3 says `Domain(String)` must stay open. But `Predicate`
in core.dag omits `Domain` entirely.

**Fix:** Add to Predicate:
```dag
| Domain { name: String }
```

**Risk:** Medium — touches the central type definitions. All parser
code that constructs these types must be updated simultaneously.

**Blocked by:** Nothing, but should be done after Wave 1 to avoid
conflicting changes.

---

## Wave 3: Fix typechecker internal bugs — DONE

**Problem:** Two correctness issues in typecheck.dag:

1. `typecheck_module` computes item-level diagnostics from
   `resolve_item_types` but drops them — only env-resolution
   diagnostics are threaded to the output.

2. `validate_no_unresolved` flags ALL surviving `Named` as errors,
   but the comments say recursive types retain Named cycle-breakers.
   Current cycle detection only handles trivial `type Foo = Foo`,
   not recursive records/sums through fields.

**Fix:**
1. Thread `all_diags` from item resolution into the TypedModule or
   accumulate them in the fold.
2. Track "being resolved" set during type resolution. Named refs
   to types in the being-resolved set are cycle-breakers and should
   be excluded from validate_no_unresolved. **DONE (Unit 3)** —
   implemented via `resolving` parameter for mutual recursion cycle
   detection.

**Risk:** Low — contained within typecheck.dag.

**Blocked by:** Wave 1 (type alignment).

---

## Wave 4: Close parser-to-emitter for service/mock_response — PARTIALLY DONE

**Problem:** The design and data model declare mock_response support,
but `parse_operation_def` leaves response/mock_response lists empty.
`emit_operation_test` only binds status/body and inserts a comment —
never invokes the operation or asserts.

**Fix:**
1. `parse_operation_def`: parse `response { ... }` and
   `mock_response { ... }` blocks, populating the OperationDef fields.
   **DONE (Unit 2).**
2. `emit_operation_test`: generate an actual test body that constructs
   mock data, calls the operation, and asserts on the response.
   Still deferred.

**Risk:** Medium — requires parser extensions and emitter logic.

**Blocked by:** Wave 2a (named constructors needed for mock data).

---

## Wave 5: Documentation reconciliation

**Problem:** Project docs are stale relative to the code:

1. `v2-project-plan.md` says parser/resolve/typecheck/emit are
   "not yet created" or "not started" — all exist now.
2. C1/C2 gap lists include items already implemented (Primitive,
   string interpolation, escape sequences, kernel intrinsics).
3. `v2-self-hosted-compiler.md` still presents `Backend = Rust | Python`
   and an older `compile(root, backend)` example, even though the
   "resolved" section says to split driver vs pure compiler.

**Fix:** Update both docs to reflect current state. Mark completed
items, remove stale gap lists, align examples with actual code.

**Risk:** Zero — documentation only.

**Blocked by:** Nothing, but best done after Waves 1-2 so the docs
reflect the final type structure.

---

## Wave 6: Test narrative accuracy

**Problem:**
1. `phase1_all_v2_modules_compile` only compiles tokenizer-related
   files — doesn't exercise resolve/typecheck/emit/pipeline.
2. `emitted_binary_smoke.rs` runs existing workspace bins, not fresh
   emitted Rust from the v2 compiler.

**Fix:**
1. Rename `phase1_all_v2_modules_compile` to
   `phase1_tokenizer_module_compiles` (it uses `compile_tokenizer_module`).
   The new `phase3_compile_all_modules` test now covers all modules.
2. Defer the "build fresh emitted Rust" smoke test until the emitter
   actually produces compilable output (post-Wave 4).

**Risk:** Zero.

**Blocked by:** Nothing.

---

## Wave 7: Normalize option/diagnostic spelling

**Problem:** Mixed styles across .dag files: `Some { value: x }`,
`Some(x)`, `none`, `None`. Optional Diagnostic fields sometimes
wrapped, sometimes raw.

**Fix:** Establish convention and enforce:
- Construction: `Some { value: x }` (record-style, consistent with
  match patterns)
- None: lowercase `none` in expression position (matches existing
  evaluator behavior where `none` → `Value::Unit`)
- Match: `Some { value: x } => ...` / `None => ...`
- Audit all .dag files for consistency

**Risk:** Low — mechanical changes.

**Blocked by:** Wave 2b (pattern model must support field bindings
before the convention is fully enforceable).

---

## Wave 8: Pipeline completeness — PARTIALLY DONE

**Problem:** Several smaller gaps in the pipeline wiring:
1. `pipeline.dag` ignores `backend` parameter (always emits Rust)
2. ~~Resolver diagnostics not threaded through pipeline~~ **DONE (Unit 5)**
   — verified already correct.
3. ~~No `Cargo.toml` or dependency manifest in emitted output~~ **DONE
   (Unit 4)** — Cargo.toml emission added.
4. Emitted integration tests use `use super::*;` (unit-test style)

**Fix:** Address individually. Items 1 and 4 remain.

**Risk:** Low per item.

**Blocked by:** Wave 4 (emitter needs to be working first).

---

## Recommended execution order

```
Wave 1 (type alignment) ──┐
                           ├── Wave 3 (typechecker bugs)
Wave 2 (AST model)     ───┤
                           ├── Wave 4 (service/mock) ──→ Wave 8 (pipeline)
Wave 5 (docs)          ───┘
Wave 6 (test naming)
Wave 7 (option spelling) ──→ depends on Wave 2b
```

Waves 1, 2, 5, 6 can start in parallel. Wave 3 depends on Wave 1.
Wave 4 depends on Wave 2a. Wave 7 depends on Wave 2b. Wave 8
depends on Wave 4.

Estimated total: ~5-7 sessions. Waves 1+5+6 are one session each.
Wave 2 is 2-3 sessions (touches the most code). Waves 3, 4, 7, 8
are one session each.
