# Codegen Quality: IR Completeness & Language Idioms

**Status**: Active (ongoing concern)
**Date**: 2026-02-05
**DSL Alignment**: Backend/codegen quality required for confident DSL emission parity
**Track**: D — Runtime/Test Hardening

## Problem Statement

Code generation produces source code that must pass linters (clippy, pylint, eslint).
When linters catch issues in generated code, we need to determine the root cause:

| Concern Type | Where to Fix | Example |
|--------------|--------------|---------|
| **Semantic bug** | Generator logic | Wrong assertion, missing edge case |
| **IR modeling gap** | IR type definitions | Can't express idiomatic pattern |
| **Project preference** | Linter config | Line length, naming conventions |

**Key insight**: Style lints that fire on generated code often reveal IR modeling gaps.
If the IR can only express the non-idiomatic form, we're forced into either:
1. Lint violations (bad)
2. Manual post-processing (fragile)
3. `#[allow(...)]` attributes (hides real issues)

**Principle**: The IR should be complete enough that generated code passes all linters
with no allows. If a linter fires, it's either a real bug or an IR gap to fix.

## Affected IRs

The codebase has multiple code generation IRs:

| IR | Location | Generates |
|----|----------|-----------|
| Test IR | `core/ir/src/code_ir.rs` | `generated_tests.rs` files |
| CLI IR | `core/codegen/src/cli_gen.rs` | CLI `main.rs` entrypoints |
| DAG IR | `core/codegen/src/dag_gen.rs` | `graph.rs` DAG builders |

Each must model idiomatic patterns for its target language.

## Case Study: `needless_return` (2026-02-05)

**Symptom**: Clippy error on all `generated_tests.rs` files:
```rust
fn mock_spec() -> MockSpec {
    return crate::graph_mock::ci_mock_spec();  // ← clippy: needless_return
}
```

**Root cause**: `Stmt` enum had `Return(Expr)` but no way to express Rust's
idiomatic tail expression (final expression without `return` keyword).

**Fix**: Added `Stmt::TailExpr(Expr)` variant and `Stmt::tail()` constructor.

**Files changed**:
- `core/codegen/src/testgen/test_ir.rs` — added `TailExpr` variant
- `core/codegen/src/testgen/render_rust.rs` — render without `return` keyword
- `core/codegen/src/testgen/codegen.rs` — use `tail()` for helper bodies

**Lesson**: The IR was syntactically complete (could generate valid Rust) but
idiomatically incomplete (couldn't generate *idiomatic* Rust).

## IR Completeness Checklists

### Test IR (`test_ir.rs`)

Rust idioms the test IR should support:

| Idiom | IR Support | Status |
|-------|-----------|--------|
| Implicit return (tail expr) | `Stmt::TailExpr` | ✅ |
| Explicit return | `Stmt::Return` | ✅ |
| Let bindings | `Stmt::Let` | ✅ |
| Mutable let | `Stmt::Let { mutable: true }` | ✅ |
| Expression statements | `Stmt::Expr` | ✅ |
| Comments | `Stmt::Comment` | ✅ |
| Method chaining | `Expr::method()` | ✅ |
| Closures | `Expr::Closure` | ✅ |
| Pattern matching | — | ❌ (not needed yet) |
| `if let` / `while let` | — | ❌ (not needed yet) |
| `?` operator | — | ❌ (not needed yet) |
| `match` expressions | — | ❌ (not needed yet) |

### CLI IR (`cli_gen.rs`)

| Idiom | Status | Notes |
|-------|--------|-------|
| Main function | ✅ | `fn main()` |
| Imports | ✅ | `use` statements |
| Error handling | ✅ | `?` via template |
| Match on enum | ✅ | hardcoded template |

### DAG IR (`dag_gen.rs`)

| Idiom | Status | Notes |
|-------|--------|-------|
| Function definitions | ✅ | `pub fn build_*()` |
| Struct construction | ✅ | `Node::opaque(...)` |
| Method chaining | ✅ | `.add_node().add_edge()` |
| Closures | ✅ | executor closures |

## Process: Handling New Lint Violations

When a lint fires on generated code:

1. **Identify the lint category**:
   - `clippy::correctness` → likely a real bug in generator logic
   - `clippy::style` / `clippy::pedantic` → likely an IR gap
   - `clippy::restriction` → likely a project preference (configure clippy)

2. **For IR gaps**:
   - Add the missing construct to the relevant IR
   - Add rendering support in the backend
   - Update the generator to use the new construct
   - Add to the completeness checklist above

3. **For project preferences**:
   - Configure in `clippy.toml` or crate-level attributes
   - Document the decision

4. **Never**:
   - Add `#[allow(...)]` to generated code (hides future issues)
   - Manually post-process generated files (fragile)

## Open TODOs

### ~~TODO: Audit all generated code for lint violations~~ DONE (2026-02-07)
- [x] Run `cargo clippy` on fresh `make codegen && make testgen` output
- [x] Categorize each lint: IR gap vs config vs bug
- [x] Fix IR gaps, configure preferences, fix bugs

**Results**: All 27 generated files (19 `generated_tests*.rs` + 8 CLI entrypoints) are clean:
- Zero `#[allow(...)]` annotations
- Zero `needless_return` — all `return;` are legitimate early-exit guards
- Zero other clippy-triggering patterns (no redundant clones, unused vars properly `_`-prefixed)
- Pre-existing clippy issues in hand-written code were fixed: `type_registry.rs` (question_mark, implicit_saturating_sub), `preflight.rs` (useless_vec ×4), `gcp-ops/ops.rs` (should_implement_trait, manual_is_multiple_of), `cloud-ops/env_status.rs` (new_without_default), `codegen_cli.rs` (needless_return restructured into per-platform helpers)

### TODO: Add lint CI check for generated code
- [x] CI step that regenerates all code and runs clippy (gunbc-ci runs codegen/testgen/pragma + clippy)
- [x] Fails if any lints fire (ensures IR stays complete)

### TODO: Cross-language idiom coverage
- [ ] When Python/TypeScript backends are implemented, audit for their idioms
- [ ] Same principle: IR should produce idiomatic output per language

## References

- Test IR implementation: `core/ir/src/code_ir.rs`
- Test IR renderer: `core/codegen/src/testgen/render_rust.rs`
- Original codegen-on-DAG design: `TODO/TODONE/TODO_codegen_dag.md`
- Testgen improvements: `TODO/TODONE/testgen-improvements.md` (Phase 11 references this doc)
