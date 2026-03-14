# v2 Compiler Postmortem — Bootstrap to cargo check (2026-03-14)

This document records the full state of the v2 self-hosted compiler at
the point where the generated Rust crate passes `cargo check`, `cargo
build`, and runtime smoke tests. It catalogs every workaround, hack,
and shortcut taken during bootstrap, and maps out all remaining work.

## What exists

**v2 .dag source:** 7 modules, 7,292 lines.

| Module | Lines | Purpose |
|--------|-------|---------|
| 00_core.dag | 341 | Shared types (Token, TokenKind, AST nodes, TypeExpr, IR) |
| 01_tokenize.dag | 471 | Tokenizer (string scanning, keyword recognition) |
| 02_parse.dag | 3,313 | Recursive descent parser with Pratt precedence |
| 03_resolve.dag | 465 | Module resolution (Kahn's topological sort) |
| 04_typecheck.dag | 1,041 | Type checking (structural, recursive type detection) |
| 05_emit.dag | 1,528 | Rust code emission from typed IR |
| 06_pipeline.dag | 133 | Pipeline orchestration (tokenize → parse → resolve → typecheck → emit) |

**Generated Rust crate:** ~10,500 lines, 9 source files (7 modules + lib.rs + v2_rt.rs).

**No `extern func` declarations.** The v2 compiler is pure .dag — no
Rust-backed extern functions in the .dag source.

## What's proven

| Layer | Evidence | Status |
|-------|----------|--------|
| Syntactic correctness | 94 parser tests (daglang-syntax), 315 codegen unit tests (daglang-emit) | CI gate |
| Type correctness | `cargo check` passes on generated crate | CI gate (v2_crate_cargo_check) |
| Link correctness | `cargo build` succeeds | Test (--ignored) |
| Semantic correctness | Phase 3 interpreter tests: tokenize → parse → resolve → typecheck → emit on real input | CI gate |
| Runtime correctness | 3 smoke tests: tokenizer produces tokens, ends with Eof, recognizes `fn` as KwFn | Test (--ignored) |

**Test counts:** 60 pass, 0 fail, 3 ignored (OOM on large files × 2, cargo build + smoke).

---

## Debt catalog

### Category 0: Confirmed invariant violations from follow-up scan

These are not generic bootstrap rough edges; they are places where the
current .dag pipeline claims one invariant and implements another.

**Invariant violation: `merge_envs()` is first-writer-wins, not last-writer-wins.**
`04_typecheck.dag` says later bindings shadow earlier ones, and the
comments explicitly describe a right-to-left dedupe. The implementation
at lines 186-197 does not reverse the bindings before folding, so it
keeps the first occurrence of each name. That means kernel or imported
bindings can incorrectly shadow later local bindings, violating the
documented environment layering (`kernel < imports < local`).

**Invariant violation: `build_type_env()` loses import provenance.**
`ResolvedImport` carries the target module for each import, but
`build_type_env()` ignores it. At lines 213-224, each imported name is
looked up in one merged parent environment built from *all* parent
modules, not in the specific module named by `imp.module_path`. If two
dependencies export the same type name, `import a { Foo }` can bind
`Foo` from `b` instead of `a`. This launders module identity across the
resolve/typecheck boundary.

**Invariant violation: the pipeline emits after resolve/typecheck failure.**
`06_pipeline.dag` only gates on parse errors. Once parsing succeeds,
`compile_sources()` always calls `resolve_modules()`, `typecheck()`, and
`emit_rust()` (lines 99-136), even if resolver or typechecker
diagnostics contain errors. That violates the emitter precondition in
`05_emit.dag` lines 1-11, which says emit receives a fully resolved,
fully typed graph with no remaining ambiguity.

**Concrete bad case: cycle modules get `dep_order = -1` and still flow downstream.**
When topological sort cannot place a module, `find_index_in_list()`
returns `-1` (03_resolve.dag lines 439-444). `resolve_modules()` still
attaches that `dep_order` and sorts the full module list by it (lines
90-104). Combined with the pipeline behavior above, cyclic or otherwise
unsorted modules are pushed to the front of the resolver output and then
typechecked/emitted anyway instead of being quarantined behind the
diagnostic.

**Invariant checker exists but the pipeline does not use it.**
`04_typecheck.dag` defines `typecheck_and_validate()` plus
`validate_no_unresolved()` / `typecheck_ok()` (lines 992-1017), but
`06_pipeline.dag` calls plain `typecheck()`. So even the compiler's own
post-typecheck invariant audit is bypassed on the main pipeline path.

### Category 1: Hardcoded bootstrap scaffolding in v2_crate_emit.rs

These exist because the v1 emitter pipeline works on individual parsed
modules without cross-module knowledge. The v2 compiler's resolve phase
handles this properly — all of these die with self-hosting.

**S78: Materialized types in `std_types_prelude()`.**
Types imported from `std.types` (`SourceSpan`, `FilePath`, `NonEmptyStr`,
`BindingPower`) are hand-written as Rust struct definitions. In the v2
compiler, these come from the .dag type definitions via resolved imports.

**S79: Hardcoded cross-module imports in `module_prelude()`.**
A match statement maps each .dag stem to its `use crate::` statements
(e.g., `02_parse` → `use crate::tokenize::*`). Should be derived from
`import` declarations in each .dag file.

**`V2_MODULE_MAP` constant.**
Hardcoded mapping from .dag file stems to Rust module names (7 entries).
Should be derived from module declarations.

**`struct_field_types` manual entries.**
`BindingPower` and `SourceSpan` field maps are manually inserted into
the struct field type registry. Should come from the type definitions.

**S81: Duplicate type suppression.**
Downstream modules that re-declare structurally identical types get their
definitions suppressed so cross-module references use the upstream type
via `use crate::upstream::*`. Correct but positional — depends on module
processing order matching the dependency graph.

### Category 2: Runtime shim functions (v2_rt.rs)

Every function in the runtime shim represents something the generated v2
crate can't express in pure generated Rust. Each must either become a
proper stdlib module or be eliminated by better codegen.

**String operations** (needed because .dag treats strings as opaque values):
- `char_at(s, pos)` — character at position
- `string_length(s)` — character count
- `substring(s, start, end)` — character range extraction
- `str_eq(a, b)` — string equality
- `code_point(c)` / `from_code_point(cp)` — Unicode conversion
- `process_escapes(raw)` — escape sequence handling (currently in .dag, shim for compiled path)

**Scanner operations** (tokenizer-specific character scanning):
- `scan_while(s, start, pred)` — scan while predicate holds
- `skip_horizontal_ws(s, start)` — skip spaces/tabs
- `scan_to_eol(s, start)` — scan to end of line
- `scan_string_end(s, start)` — scan string literal with escape handling

**Collection operations:**
- `concat<T: Concat>(a, b)` — polymorphic string/list concatenation
- `lookup<V>(table, key)` — HashMap lookup with clone
- `list_concat<T>(a, b)` — deprecated list concatenation

**Filesystem:**
- `filesystem_read(path)` — file read (panics on error)

### Category 3: Type-unaware codegen heuristics in fn_codegen.rs (S81)

The v1 fn_codegen pipeline compiles .dag function bodies to Rust without
type information. Every decision requiring types is heuristic. All of
these are eliminated by the v2 compiler's typed emitter.

**S76: `clone_if_needed()` — blind ownership.**
Adds `.clone()` to all variable/field expressions passed as arguments or
struct fields. ~300 unnecessary clones in the generated crate. Correct
but inefficient. v2 fix: ownership/liveness tracking.

**S77: `infer_struct_name()` — field-name guessing.**
Anonymous records `{ field: value }` mapped to named Rust structs by
matching field names against known definitions. Wrong when multiple
structs share field names. v2 fix: typechecker resolves anonymous
records to their structural type.

**`infer_scrutinee_type()` / `infer_type_from_arms()`.**
Infers match scrutinee enum type from parameter types, ir_scope, and arm
variant names. Picks enum with best variant overlap; returns None on ties.
v2 fix: typechecker knows the scrutinee type.

**`is_likely_option_receiver()` / `is_likely_option_receiver_ctx()`.**
Detects method chains returning Option (`.last()`, `.first()`, `.get()`,
`.find()`) to convert `.value` field access to `.unwrap()`. v2 fix:
type system tracks optionality.

**`is_already_optional_expr()` / `is_null_ast_expr()`.**
Prevents double-wrapping in `Some()` by detecting expressions that
already produce optional values. v2 fix: type annotations on IR nodes.

**`needs_box_wrapping()`.**
Checks if a field needs `Box<>` wrapping across three naming patterns
(direct, variant-qualified, enum-qualified). v2 fix: recursive type
detection in typechecker.

**`compile_expr_in_field_context()`.**
Compiles expressions with expected field type for variant qualification.
Uses `enum_variants` map to resolve ambiguous variant names. v2 fix:
typechecker resolves variants from context type.

**`infer_collection_element_struct()` / `infer_ast_expr_type()`.**
Core workaround: infers types from ir_scope, struct_field_types, and
function return types without actual type information. v2 fix: typed IR.

**`escape_rust_keyword()`.**
Hardcoded list of ~40 Rust keywords, prefixed with `r#`. v2 fix:
the v2 emitter already has this list in 05_emit.dag line 59-68, but
derived from .dag source.

### Category 4: .dag source workarounds

**C4: Forward declarations in 04_typecheck.dag.**
Lines 33-41 forward-declare types (`ResolvedImport`, `ResolvedModule`,
`ModuleGraph`) that should come from `import v2.compiler.resolve`.
Status: awaiting proper cross-module import in compiled path.

**C5: Forward declarations in 05_emit.dag.**
Lines 29-33 forward-declare types (`TypedGraph`, `TypedModule`,
`TypeEnv`, `TypeBinding`) that should come from
`import v2.compiler.typecheck`. Same root cause as C4.

**S56: Parse error check before module extraction in 06_pipeline.dag.**
Lines 101-106 check for parse errors before extracting modules because
`parse()` returns `{ module: none }` on failure. Extracting before
checking would produce `List<Module?>` — a type violation the evaluator
can't catch. v2 fix: typed Result returns.

**Optional type handling limitation in 04_typecheck.dag.**
Lines 505-518: Uses field access `.value` instead of pattern matching on
`TypeExpr?` because the evaluator represents Optional as
`Map({"value": ...})` without a `_variant` field. Workaround: checks
`expr == none` instead of matching `Some { value: te }`. v2 fix:
compiled code has proper Option types.

**S54: Service parameter forwarding in 05_emit.dag.**
Line 86: builds an item registry mapping item names to their kind and
service dependencies. Used by `emit_call` to forward service params to
callees. This is correct design but could be cleaner with a proper
module-level analysis pass.

### Category 5: code_ir target leakage (S81 — CRITICAL)

The code_ir layer was designed as target-agnostic IR that all backends
(Rust, Go, C, Verilog) can render. During v2 bootstrap, fn_codegen has
injected ~15 Rust-specific constructs directly into the IR:

- `clone_if_needed()` → `.clone()` method calls
- `Box::new()` wrapping → Rust heap boxing
- `Some()`/`None` injection → Rust `Option<T>`
- `.as_str()` insertion → Rust `String` vs `&str`
- `..Default::default()` → Rust struct update syntax
- `LazyLock` for Map data → Rust static initialization
- `Deref`/`*` for Box unwrapping → Rust-specific dereference

**Why this matters:** If the IR contains `"clone"`, `"Box::new"`,
`"Some"`, then every non-Rust backend must enumerate and strip Rust
idioms. The IR has become a Rust AST with extra steps.

**Fix:** The v2 emitter reads computation facts (types, cardinality,
recursion, optionality) and applies rendering facts per-backend. The
code_ir should represent "this value is used here and here" — the
backend decides what that means (Rust: clone, C: nothing, Go: nothing,
Verilog: fan-out wire).

### Category 6: Evaluator limitations (Branch 7)

The v1 evaluator was built for simple fn bodies. Using it for the v2
compiler's 80+ mutually-recursive functions, deep self-recursion, and
multi-stage pipeline contracts exposed:

- **Stack amplification:** Each DSL call pushes ~1.6KB of native Rust
  frames vs ~100-200 bytes compiled. Tests need 16-32MB stacks.
- **No type safety:** `Value::Unit` flows where `Value::Map` (Module)
  is expected with no error until downstream fails (S57, now mitigated).
- **Performance:** `Env::from_inputs` clones on every non-self call.
  Map field flattening clones every field.

**Status:** Mitigated (S55-S61 fixes), not solved. Self-hosting
eliminates this entire category.

---

## Design decisions (locked in 2026-03-14)

### D1: TypedGraph is the compiler boundary — emit is per-backend

`04_typecheck.dag` outputs `TypedGraph` (fully resolved, no unresolved
Named references). `05_emit.dag` = `emit_rust()` — explicitly a Rust
backend. `06_pipeline.dag` has `type Backend = Rust | Python` with a
match dispatch.

Adding a backend means adding one `.dag` module + one match arm. The
typecheck output is the contract. Code upstream of emit is
target-agnostic; code in emit is target-specific. No exceptions.

**Implication for S81:** The code_ir target leakage in v1 is moot — it
dies with fn_codegen. The v2 emitter is *correctly* Rust-specific
because it IS the Rust backend. We don't need to clean up code_ir; we
need to delete it.

### D2: v1 is frozen — no parallel implementations

v1's fn_codegen is a parallel implementation of v2's 05_emit.dag. The
only justified v1 changes are those that unblock the self-hosting
fixed point. No heuristic improvements, no new workarounds. Once
self-hosting lands, delete entirely.

### D3: Self-hosting equivalence is behavioral, not textual

The v2 emitter may produce intentionally different (better) Rust than
v1. The fixed-point comparison happens at the *pipeline output* level:

1. Both compile (`cargo check` passes)
2. Same observable behavior (same tokens/AST/types on same .dag input)
3. NOT character-identical source

This frees the v2 emitter to improve codegen quality without
artificial constraints.

### D4: Runtime shims → first-class language features, not annotations

**Principle:** Solve through first-class features before reaching for
annotations or `extern func` declarations. Each annotation pattern must
be justified by a concrete failure of first-class features.

String operations map to language features with existing analogs:

| Current shim call | First-class feature | Rationale |
|---|---|---|
| `char_at(s, pos)` | `s[pos]` — indexing syntax | Universal; renders per-backend |
| `string_length(s)` | `count(s)` — extend existing intrinsic | `count` already works on lists |
| `substring(s, start, end)` | `s[start..end]` — slice syntax | Natural extension of indexing |
| `lookup(table, key)` | `table[key]` — indexing syntax | Same syntax as strings |
| `code_point(ch)` / `from_code_point(cp)` | Cast syntax or known intrinsic | Type conversion |
| `concat` | Already first-class | No change |
| `scan_while`, `skip_horizontal_ws`, etc. | Already .dag functions | Shim exists only because v1 can't compile the .dag versions |

Indexing and slicing are computation facts ("access element at
position"), not rendering decisions. Each backend renders in its idiom.

### D5: Forward declarations are temporary — imports are the only mechanism

C4 (typecheck.dag) and C5 (emit.dag) have forward-declared types that
are exact copies of types in resolve.dag and typecheck.dag. This is a
duplicate representation. Once the compiled v2 crate has working
cross-module imports, the .dag source switches to `import` and the
duplicated types are deleted. No formalization of forward declarations
as a language feature.

### D6: `data` definitions → backend renders per-idiom

`data` in the typed IR is a constant definition. The Rust emitter
renders as `lazy_static!` / `const` / `static`. A Go emitter renders
as package-level `var`. 05_emit.dag already handles this correctly via
`emit_data_def()`. No backport to v1 — let it die.

---

## Follow-on work (ordered by dependency)

### Phase 1: Self-hosting fixed point

**Goal:** The v2 compiler compiles itself. Behavioral equivalence with
v1 output (D3).

1. **Run v2 pipeline end-to-end on a trivial .dag file.** The smoke
   test proves `tokenize` works. Next: prove `parse`, `resolve`,
   `typecheck`, and `emit` work by feeding the tokenizer output through
   each stage and checking the output.

2. **Run v2 pipeline on v2 source.** Feed the 7 .dag files through the
   compiled v2 pipeline. Compare pipeline output (tokens, AST, types)
   with v1's output. Both should produce working code.

3. **Fixed-point test.** Compile v2 with v1 → get binary B1. Compile v2
   with B1 → get binary B2. B1 and B2 should produce identical output
   on the same input.

### Phase 2: Eliminate runtime shims via language features (D4)

Add first-class language features to replace v2_rt.rs:

1. **Add Index expression** to AST/parser/typechecker/emitter —
   `s[pos]` for strings, `table[key]` for maps.
2. **Add Slice expression** — `s[start..end]` for substring.
3. **Extend `count()` intrinsic** to strings (already works on lists).
4. **Handle `code_point`/`from_code_point`** — cast syntax or known
   intrinsic (decide when the need arises).
5. **Migrate tokenizer** — `char_at(s, pos)` → `s[pos]`,
   `string_length(s)` → `count(s)`, `substring(s, start, end)` →
   `s[start..end]`, `lookup(table, key)` → `table[key]`.
6. **Scanner functions** (`scan_while`, `skip_horizontal_ws`, etc.)
   are already .dag — they'll just compile correctly once self-hosting
   works. No language feature needed.
7. **`filesystem_read`** → I/O transport (existing mechanism).
8. **Delete v2_rt.rs.**

### Phase 3: Eliminate bootstrap scaffolding (S78, S79)

1. **Derive `module_prelude()` from .dag imports.** Read `import`
   declarations from each module's AST, map to `use crate::` statements.
   Delete the hardcoded match statement.

2. **Derive `std_types_prelude()` from std .dag files.** Parse the std
   type definitions and emit them. Delete hand-written struct defs.

3. **Derive `V2_MODULE_MAP` from module declarations.** Read `module`
   declarations and derive stem→name mappings.

4. **Delete manual `struct_field_types` entries.** All field type maps
   should come from `build_struct_field_types()` over parsed type defs.

### Phase 4: Resolve .dag source forward declarations (C4, C5)

1. **Wire cross-module imports in the compiled v2 crate.** When the
   compiled crate's modules can import from each other, replace forward
   declarations in 04_typecheck.dag and 05_emit.dag with actual imports.

2. **Delete the forward-declared types.** Once imports work, the
   duplicated type definitions in typecheck and emit modules can be
   removed.

### Phase 5: Clean up code_ir target leakage (S81)

This is the architectural cleanup. Every Rust-specific construct
currently injected into code_ir by fn_codegen must be moved to
`render_rust.rs` (or deleted when v2 emitter replaces fn_codegen).

1. **Audit all `code_ir::Expr` and `code_ir::Stmt` variants** for
   target-specific content.
2. **Add target-agnostic IR nodes** for ownership, optionality,
   recursion where needed.
3. **Move rendering decisions to backends.** Each backend interprets
   the target-agnostic nodes in its own idiom.

**Note:** This is largely moot if the v2 emitter replaces fn_codegen
entirely. The v2 emitter in 05_emit.dag already reads typed IR and
makes per-target decisions. The cleanup is only needed if fn_codegen
persists beyond bootstrap.

### Phase 6: Delete v1 bootstrap code

Once self-hosting is proven and tested:

- `fn_codegen.rs` — entire file
- `v2_crate_emit.rs` — entire file
- `v2_runtime_shim.rs` — entire file
- All heuristic functions: `infer_struct_name`, `clone_if_needed`,
  `is_option_expr`, `is_none_expr`, `is_likely_option_receiver`,
  `needs_box_wrapping`, `is_already_optional_expr`,
  `compile_expr_in_field_context`, `infer_ast_expr_type`,
  `escape_rust_keyword`, `infer_scrutinee_type`, `infer_type_from_arms`,
  `synthesize_anonymous_structs`
- `std_types_prelude()`, `module_prelude()`, `V2_MODULE_MAP`
- Phase 3 interpreter tests (replaced by compiled tests)
- `with_parser_stack(16MB)` scaffolding

### Phase 7: Capabilities that would further reduce debt

From SUSTAINABILITY.md — not blockers, but would accelerate:

- **Language model serialization to JSON IR** — backend type mappings
  as data, not code
- **`behavior` DSL construct** — algebraic property test enumeration
- **Structural coercion paths** — eliminate `is_compatible` case
  enumeration

---

## Sustainability ledger cross-reference

| S-ID | Status | Description |
|------|--------|-------------|
| S76 | OPEN | `clone_if_needed()` blind ownership — dies with self-hosting |
| S77 | OPEN | `infer_struct_name()` field-name guessing — dies with self-hosting |
| S78 | OPEN | Materialized types in `std_types_prelude()` — Phase 3 above |
| S79 | OPEN | Hardcoded `module_prelude()` imports — Phase 3 above |
| S81 | OPEN | fn_codegen emits Rust, not code_ir — Phase 5 above / dies with self-hosting |
| S80 | DONE | Untyped `PR.val: Map` → 45 typed result types |
| S75 | DONE | `+` operator overloaded → `concat()` intrinsic |
| S54 | DONE | Service param forwarding → item registry |
| S55 | DONE | TCO for self-recursive tail calls |
| S56 | DONE | Parse error laundering → explicit error gates |
| S57 | DONE | No runtime type enforcement → `check_call_inputs()` |
| S58-S61 | DONE | Evaluator performance fixes |
| C4 | OPEN | Forward declarations in 04_typecheck.dag — Phase 4 above |
| C5 | OPEN | Forward declarations in 05_emit.dag — Phase 4 above |

## Error count progression

```
2204 → 829 → 279 → 258 → 231 → 223 → 115 → 36 → 32 → 10 → 7 → 0
```

Key inflection points:
- 2204 → 829: Optionality tracking (check `T?` annotations, prevent double-wrapping)
- 829 → 115: Box wrapping, runtime shim imports, string match, variant disambiguation
- 115 → 36: S76 clone_if_needed, S80 typed parse results, anonymous struct naming
- 36 → 0: Optionality mismatches, ServiceCall lowering, parser bug (`{ "string" }`
  parsed as empty record), fold Vec inference, .dag source fixes
