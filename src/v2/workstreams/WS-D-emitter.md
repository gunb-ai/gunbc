# WS-D: Emitter Correctness

## Scope

Fix miscompilations in the Rust emitter, add diagnostic channel, consume
`TypeEnv` properly instead of re-deriving type information from raw AST.

## Files

- `src/v2/05_emit.dag`

## Key invariant

> **DAG nodes are facts, rendering is separate.** How to express truths in a target
> language is a rendering decision that lives in the backend, never in the IR.

## Priority 1: Miscompilations (wrong Rust output)

These produce Rust code that compiles but has wrong semantics, or produces
Rust code that doesn't compile.

1. **for-loop desugaring** (`05_emit.dag`, fn `emit_expr`, for-loop branch)
   - Changes return type from `()` to `Vec<T>` — a `for` loop that should be
     effectful instead collects into a vector
   - Fix: emit `for` as `for x in iter { body; }` (unit return), not `.map().collect()`

2. **`??` associativity** (`05_emit.dag` + `02_parse.dag`)
   - Right-associative instead of left-associative
   - `a ?? b ?? c` parses as `a ?? (b ?? c)` instead of `(a ?? b) ?? c`
   - Fix: parse `??` as left-associative (this also involves WS-B's `02_parse.dag`)

3. **NonEmptyList downgrade** (`05_emit.dag`)
   - Silently drops non-emptiness guarantee: `NonEmptyList<T>` emits as `Vec<T>`
   - Fix: emit as a newtype wrapper or document the intentional downgrade

4. **Type alias identity loss** (`05_emit.dag` + `04_typecheck.dag`)
   - `UserId = String` emits as bare `String`, losing the alias
   - Root cause is in typechecker (WS-C `type_body_to_expr`), but emitter compounds it
   - Fix: if alias survives to emit, emit `type UserId = String;`

5. **`from_key` serde loss** (`05_emit.dag`)
   - Fields with `from_key` don't get `#[serde(rename = "...")]` in output
   - Fix: emit serde rename attribute when `from_key` is present

6. **Match pattern capitalization** (`05_emit.dag`)
   - Uppercase identifiers in match patterns parsed as `VariantPattern` even when
     they should be variable bindings
   - Fix: distinguish variant references from bindings using type context

7. **`emit_func_def` signature mismatch** (`05_emit.dag`, fn `emit_func_def` ~line 502)
   - Declares `Result<T>` return type but emits raw body without wrapping in `Ok()`
   - Fix: emit `Ok(body)` wrapper when return type is `Result`

8. **`emit_record_lit` invalid syntax** (`05_emit.dag`, fn `emit_record_lit` ~line 889)
   - Bare `{ field: expr }` without type name — invalid Rust
   - Fix: always emit `TypeName { field: expr }` or use anonymous struct

9. **`emit_prelude` invalid syntax** (`05_emit.dag`, fn `emit_prelude` ~line 241)
   - Escaped braces in use statement produce invalid Rust
   - Fix: emit valid `use` statements

10. **`emit_cast` arbitrary targets** (`05_emit.dag`, fn `emit_cast` ~line 1040)
    - Emits `as T` for any target type without validating it's a valid Rust cast
    - Fix: restrict to known-valid cast pairs or use `.into()`/`From`

11. **`emit_call` drops arg names** (`05_emit.dag`, fn `emit_call` ~line 691)
    - Named arguments in .dag become positional in Rust — argument names are lost
    - Fix: emit arguments in the correct positional order based on function signature

## Priority 2: Architectural gaps (boundary violations)

1. **Gap 10: Emitter ignores `TypeEnv`** (`05_emit.dag`, fn `emit_module` ~line 200)
   - The emitter receives a `TypedGraph` from the typechecker but re-derives type
     information from the raw AST instead of using the resolved `TypeEnv`
   - Fix: thread `TypeEnv` through emit functions and use resolved types

2. **Gap 11: Emitter has no diagnostic channel**
   - Emitter silently swallows problems or fabricates output instead of reporting
   - Fix: add `diagnostics: List<Diagnostic>` to `EmitResult` and thread through

3. **Gap 12: Emitter redefines typechecker output types** (`05_emit.dag` ~lines 36-53)
   - Emitter defines its own copies of types that should come from `04_typecheck.dag`
   - Fix: import the canonical types from `04_typecheck`

## Priority 3: Fabrication residuals

1. **`emit_first_arg` empty literal** (`05_emit.dag`, fn `emit_first_arg` ~line 799)
   - Returns empty string `""` when no args present
   - Fix: return proper error or `None`

2. **`extract_service_name` "Unknown" fallback** (`05_emit.dag`, fn `extract_service_name` ~line 1518)
   - Returns `"Unknown"` when service name can't be extracted
   - Fix: propagate error

3. **`emit_data_value_json` var → quoted, other → "null"** (`05_emit.dag`, fn `emit_data_value_json` ~line 1632)
   - Variables become quoted strings, unknown expressions become `"null"`
   - Fix: emit proper JSON or error

4. **Anonymous type erasure** (`05_emit.dag`)
   - Anonymous record types silently erased during emission
   - Fix: generate named wrapper types or error

## Verification

```bash
# Emitter tests
cargo test -p v2-compiler-tests

# Re-emit and verify generated crate compiles
cargo test -p v2-compiler-tests v2_crate_emit_to_target -- --ignored
cd target/v2-compiler && cargo check
```

## Coordination notes

- **WS-C dependency for Gap 10/12**: If WS-C changes the `TypeEnv` type, this
  stream needs to consume the new shape. Coordinate on the boundary type.
- **WS-B dependency for `??`**: The associativity fix spans parser and emitter.
  Either fix both in one stream or coordinate the parse-side change with WS-B.
- **Item 3 (NonEmptyList)**: If this is an intentional downgrade for bootstrap,
  document it rather than fixing. Check with user.
