# WS-B: Parser & Tokenizer Hardening

## Scope

Replace fabrication fallbacks with proper errors, fix silent drops and semantic
erasure in the tokenizer and parser. These are "fail-open" behaviors that make
invalid programs look valid.

## Files

- `src/v2/01_tokenize.dag`
- `src/v2/02_parse.dag`

## Key invariant

> **No fallbacks that fabricate.** Every code path succeeds fully or fails with a
> clear error. No fallback defaults producing valid-looking but wrong output.

## Priority 1: Fabrication fallbacks (highest impact)

Invalid inputs produce valid-looking output. Each of these constructs a
"successful" result from nothing when the input is actually malformed.

1. **`scan_number` zero default** (`01_tokenize.dag`, fn `scan_number`)
   - When number parsing fails, falls back to `0` instead of erroring
   - Fix: return a tokenizer error diagnostic

2. **`parse_config_fields` empty-string fabrication** (`02_parse.dag`, fn `parse_config_fields`)
   - Fabricates `""` for missing config values
   - Fix: return parse error when expected field value is missing

3. **`parse_pattern` placeholder wildcards** (`02_parse.dag`, fn `parse_pattern`)
   - Constructs synthetic wildcard patterns when pattern parsing fails
   - Fix: return error result

4. **`parse_status_pattern` synthetic literal** (`02_parse.dag`, fn `parse_status_pattern`)
   - Fabricates `"_"` literal on failure
   - Fix: propagate parse error

5. **`token_to_binop` wildcard → Add** (`02_parse.dag`, fn `token_to_binop`)
   - Unknown tokens map to `Add` via wildcard match
   - Fix: return `None` or error for unrecognized tokens

6. **~28 dummy-node error paths** (`02_parse.dag`)
   - Scattered throughout: construct nodes with `name: ""`, `span: {0, 0}` on error
   - Fix: each error path should return a proper error result, not a fabricated AST node
   - Search for: `name: ""` and `SourceSpan { start: 0, end: 0 }` to find all instances

## Priority 2: Silent drops (data loss)

Data is silently discarded without error or warning.

1. **`parse_op_body_entries` skips unknown entries** (`02_parse.dag`, fn `parse_op_body_entries`)
   - Entries that don't match known patterns are silently skipped
   - Fix: emit diagnostic for unrecognized entries

2. **`parse_capability` empty fabrication** (`02_parse.dag`, fn `parse_capability`)
   - Fabricates empty capability on parse failure
   - Fix: return error

3. **`make_call_expr` callee rewriting** (`02_parse.dag`, fn `make_call_expr`)
   - Rewrites callee expressions silently
   - Fix: preserve original callee or emit diagnostic

4. **Unterminated string laundering** (`01_tokenize.dag`)
   - Unterminated strings are accepted without error
   - Fix: emit diagnostic and mark token as erroneous

## Priority 3: Semantic erasure (information loss)

Distinct inputs collapse to the same representation, losing semantic meaning.

1. **`parse_return` vs bare expr indistinguishable** (`02_parse.dag`, fn `parse_return`)
   - `return x` and `x` produce identical AST nodes
   - Fix: preserve `is_return` flag or distinct node type

2. **`parse_paren_expr` `()` → RecordLit** (`02_parse.dag`, fn `parse_paren_expr`)
   - Empty parens `()` become a `RecordLit` instead of unit
   - Fix: emit distinct unit expression

3. **`parse_brace_expr` unwrap** (`02_parse.dag`, fn `parse_brace_expr`)
   - Single-element brace blocks silently unwrap
   - Fix: preserve block structure

4. **`parse_interp_parts` malformed → valid** (`02_parse.dag`, fn `parse_interp_parts`)
   - Malformed interpolation parts become valid string literals
   - Fix: emit diagnostic

5. **`parse_import` collapse** (`02_parse.dag`, fn `parse_import`)
   - Different import forms collapse to same representation
   - Fix: preserve import style distinction

## Priority 4: Structural cleanup

1. **`expect_ident` vs `expect_name` near-duplicate** (`02_parse.dag`, lines ~318-361)
   - Two almost-identical functions; `expect_name` also accepts keywords
   - Fix: unify into one function with a parameter, or document distinction clearly

2. **`parse_io_blocks_acc` copy-paste** (`02_parse.dag`, fn `parse_io_blocks_acc`)
   - Contains duplicated parsing logic
   - Fix: extract shared logic

3. **`pattern`/`interface`/`func` coercion** (`02_parse.dag`)
   - Different declaration kinds coerced into the same AST node type
   - Fix: use distinct Item variants per declaration kind

## Verification

```bash
# Unit + integration tests
cargo test -p v2-compiler-tests

# Re-emit and check the generated crate still compiles
cargo test -p v2-compiler-tests v2_crate_emit_to_target -- --ignored
cd target/v2-compiler && cargo check
```

## Working rules

- Every fabrication you remove must be replaced with a diagnostic-producing error path
- After each fix, run `cargo test -p v2-compiler-tests` to confirm no regressions
- If a fabrication is load-bearing (tests fail when removed), that's a bug in
  the caller — fix the caller, don't keep the fabrication
- Prefer fixing items within the same priority band before moving to the next
