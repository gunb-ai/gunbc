# WS-F: Rust Codegen Determinism & Scaffolding

## Scope

Fix non-deterministic output in the v1 Rust codegen that compiles .dag to Rust,
reduce hardcoded bootstrap scaffolding, clean up code_ir.

## Files

- `src/v1/07_emit/daglang-emit/src/fn_codegen.rs`
- `src/v1/07_emit/daglang-emit/src/type_codegen.rs`
- `src/v1/07_emit/daglang-emit/src/v2_crate_emit.rs`
- `src/v1/00_foundation/ir/src/code_ir/mod.rs`

## Key invariant

> **No parallel implementations.** If the same computation exists in two forms, one
> must be deleted.

## Priority 1: Non-determinism (different output on each run)

These are HashMap/HashSet iteration-order dependencies that cause the generated
Rust crate to differ between runs, even with identical input.

1. **`infer_struct_name` HashMap iteration tiebreaker** (`fn_codegen.rs` ~line 434)
   - When multiple structs match the same field set, the winner depends on HashMap
     iteration order
   - Fix: use a deterministic tiebreaker (e.g., lexicographic sort on struct name)

2. **`compute_recursive_fields` HashSet DFS roots** (`type_codegen.rs` ~line 1191)
   - DFS traversal order depends on HashSet iteration
   - Fix: sort the DFS roots before traversal

3. **`fill_missing_fields` HashMap iteration** (`fn_codegen.rs` ~line 341)
   - Missing field defaults are filled in HashMap iteration order
   - Fix: iterate fields in a deterministic order (e.g., sorted by field name,
     or match the struct definition order)

4. **`build_variant_to_enum` order-dependent** (`v2_crate_emit.rs` ~line 523)
   - HashMap from variant name to enum name — when a variant belongs to multiple
     enums, the mapping depends on insertion order
   - Fix: detect ambiguity and error, or use deterministic resolution

5. **`TypeDefSignature` field-order sensitive** (`v2_crate_emit.rs` ~line 611)
   - Signature comparison is sensitive to field order in records
   - Fix: normalize field order before comparing (sort by field name)

## Priority 2: Bootstrap scaffolding reduction

1. **Derive `V2_MODULE_MAP` from .dag module declarations** (`v2_crate_emit.rs` ~line 48)
   - Currently hardcoded: `("00_core", "v2_core")`, `("01_tokenize", "v2_tokenize")`, etc.
   - Fix: derive from the `module` declarations in the .dag files

2. **Derive `MODULE_PATH_TO_RUST_MOD` from import paths** (`v2_crate_emit.rs` ~line 61)
   - Currently hardcoded import path → Rust module mapping
   - Fix: derive from the actual import graph

3. **Remove `opt-level=1` workaround** (`v2_crate_emit.rs` ~line 298-304)
   - Was needed pre-S76 for stack overflow prevention; S76 is now fixed with Rc
   - Fix: remove the `opt-level = 1` line from generated Cargo.toml, verify the
     generated crate still builds without it

4. **Derive `std_types_prelude()` from std type .dag files** (`v2_crate_emit.rs` ~line 343)
   - Currently a hardcoded string block of materialized types
   - Fix: generate from the `dsl/std/` type definitions

## Priority 3: code_ir target leakage (lower priority)

1. **`FnDef` params/return_type as String** (`code_ir/mod.rs` ~line 409-410)
   - Function parameters and return types are `String` instead of `IrType`
   - This means the IR contains rendered Rust fragments, not target-agnostic types
   - Fix: use `IrType` enum for params and return type

2. **`StructDef` fields as String** (`code_ir/mod.rs` ~line 447)
   - Struct field types are `String` instead of `IrType`
   - Same issue: IR contains rendered Rust, not abstract types
   - Fix: use `IrType` for field types

## Verification

```bash
# Emitter unit tests
cargo test -p daglang-emit

# V2 compiler tests
cargo test -p v2-compiler-tests

# Full round-trip: re-emit + check generated crate
cargo test -p v2-compiler-tests v2_crate_emit_to_target -- --ignored
cd target/v2-compiler && cargo check
```

## Working notes

- Priority 1 (non-determinism) is the most impactful — it makes diffs noisy and
  CI unreliable. Each fix is mechanical: replace HashMap/HashSet iteration with
  sorted iteration.
- Priority 2 (scaffolding) requires understanding how the v1 compiler discovers
  and processes .dag modules. Read `v2_crate_emit.rs` top-to-bottom before starting.
- Priority 3 (code_ir) is a larger refactor that affects many callsites. Defer
  unless the other priorities are done.
- For the `opt-level=1` removal: test carefully. If the generated crate still
  hits stack overflows, the fix should stay (and document why).
