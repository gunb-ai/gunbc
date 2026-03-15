# WS-G: Runtime Shim Elimination

## Scope

Replace hardcoded Rust runtime functions with first-class .dag language features.
This is the longest-running stream and depends on language extensions from other
streams.

## Files

- `src/v1/07_emit/daglang-emit/src/v2_runtime_shim.rs` (186 lines)
- Potentially `.dag` syntax extensions in `src/v2/00_core.dag`

## Key invariant

> **Domain lives in the DSL, not in Rust.** If something can be expressed in `.dag`
> files, it must not be hardcoded in Rust.

## Current runtime shim functions

Read `v2_runtime_shim.rs` to get the current list. The known shim functions
(as of plan creation) include:
- `char_at`, `substring` — character/substring access
- `string_length` — string length
- `code_point`, `from_code_point` — char ↔ int conversion
- `filesystem_read` — file I/O
- Scanner helper functions used by the tokenizer

## Items (in dependency order)

1. **Add Index/Slice expressions to .dag** (`s[pos]`, `s[start..end]`)
   - Replaces `char_at` and `substring` runtime shims
   - Requires parser changes (new expression type) — coordinate with WS-B if active
   - Must update `00_core.dag` (new `Expr` variant), `02_parse.dag`, `05_emit.dag`

2. **Extend `count()` to work on strings**
   - Replaces `string_length` runtime shim
   - `count` already works on `List<T>` — extend to `String`
   - May require emitter change only (emit `.len()` for strings)

3. **Handle `code_point`/`from_code_point` as casts**
   - Character ↔ integer conversion should be a cast expression, not a function
   - `ch as Int` and `n as String` (or a `Char` type)

4. **Migrate tokenizer scanner functions to .dag control flow**
   - Functions like `scan_while`, `is_digit`, `is_ident_start` are currently in the
     runtime shim
   - With index/slice expressions (item 1), these can be rewritten in .dag

5. **Replace `filesystem_read` with proper I/O transport**
   - File reading should go through `gunbc-lib-transport`, not a runtime shim
   - This is the most architecturally significant change

6. **Delete `v2_runtime_shim.rs`**
   - Final cleanup after all shims are migrated
   - Only possible when all above items are done and tests pass

## Verification

```bash
# Re-emit crate
cargo test -p v2-compiler-tests v2_crate_emit_to_target -- --ignored

# Build and test generated crate (self-parse must still pass)
cd target/v2-compiler && cargo test

# Full workspace (ensure no regressions)
cargo test --workspace --exclude gunbc-dag-tests --exclude gunbc-codegen
```

## Run order

**Run this workstream last.** It depends on:
- WS-B (parser) for new expression syntax
- WS-E (core) for new `Expr` variants in `00_core.dag`
- Language-level features that may emerge from other streams

Each item can be landed independently — there's no requirement to do all 6 at once.
The verification step (self-parse) catches regressions immediately.

## Working notes

- Read `v2_runtime_shim.rs` first — it's only 186 lines
- Each shim function has callers in the generated Rust crate. Use `grep` in
  `target/v2-compiler/src/` to find all callsites before removing a shim.
- The tokenizer (`01_tokenize.dag`) is the heaviest consumer of runtime shims.
  Items 1 and 4 together should eliminate most of its shim dependencies.
