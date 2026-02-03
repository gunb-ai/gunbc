# Codegen-on-DAG Migration

**Status**: Design
**Date**: 2026-02-03

## Problem

`codegen.rs` (2500 lines) converts structured obligation/analysis data into
generated test code via raw `format!()` string interpolation. This is where
every testgen hack lives: `value_to_rust_literal`'s catch-all arm, the
`NonEmpty` codegen bug, the `Satisfies` comment-not-assertion, the `List`
`filter_map` that drops non-string elements. The compiler can't enforce
exhaustiveness on string templates, and the entire emission layer is
hardcoded to Rust syntax.

## Observation

The codebase already models types as DAGs (`Dag<TypeOp>`), patterns as DAGs
(`Dag<PatternOp>`), and languages as DAGs (`Dag<LanguageOp>`). Code emission
is the one pipeline stage that breaks out of this model into ad-hoc strings.

## Design: `Dag<CodeOp>`

Introduce `CodeOp` as a new operation type. A generated test is a
`Dag<CodeOp>` — nodes are simple operations, edges are data flow. Rendering
to a target language is a backend pass over the DAG, not interleaved with
logic.

### CodeOp variants (minimal set)

```
CodeOp::Literal(Value)        — emit a value as a target-language literal
CodeOp::VarBind(name)         — bind input to a named variable
CodeOp::FieldAccess(name)     — access a field/key on input struct/map
CodeOp::Call(fn_name)         — call a function with input ports as args
CodeOp::AssertEq              — two inputs, emit equality assertion
CodeOp::AssertTrue            — one input, emit truthiness assertion
CodeOp::Construct(kind)       — build a List/Map/Struct from inputs
CodeOp::TestBlock(name)       — wrap body in a test function/method
```

Each variant maps to a simple, well-defined operation that every language
can express. No variant requires language-specific knowledge. The ports on
each node carry `TypeId` and `Cardinality` as they do today, so the
existing type/cardinality checking infrastructure applies.

### Backend trait

```
trait CodeRenderer {
    fn render_literal(&self, value: &Value) -> String;
    fn render_assert_eq(&self, left: &str, right: &str) -> String;
    fn render_assert_true(&self, expr: &str) -> String;
    fn render_test_block(&self, name: &str, body: &str) -> String;
    fn render_var_bind(&self, name: &str, expr: &str) -> String;
    fn render_call(&self, fn_name: &str, args: &[String]) -> String;
}
```

Rust, Python, TypeScript each implement this. `render_literal` replaces
`value_to_rust_literal` — and since it takes `&Value` (not `_`), the
compiler forces exhaustive handling per backend. Adding a new `Value`
variant breaks all backends at compile time.

### Migration path

The existing phase generators (`generate_execution_tests`,
`generate_contract_tests`, etc.) currently return `String`. Migration
changes their return type to `Dag<CodeOp>`. A final `render(dag, backend)`
pass produces the source text.

This can be done phase-by-phase:

1. **Phase 0**: Implement `CodeOp`, `CodeRenderer`, Rust backend.
2. **Phase 1**: Migrate `value_to_rust_literal` → `CodeOp::Literal` +
   `render_literal`. This immediately fixes Hacks 3, 4, and the List bug.
3. **Phase 2**: Migrate assertion emission (`to_check_code`) →
   `CodeOp::AssertEq` / `CodeOp::AssertTrue`. Fixes Hack 1 (Satisfies).
4. **Phase 3**: Migrate test function scaffolding → `CodeOp::TestBlock`.
5. **Phase 4**: Add Python/TypeScript backends.

Each phase is independently shippable. After Phase 1, the catch-all hack
is structurally impossible. After Phase 3, `codegen.rs` contains zero
`format!()` calls for code emission.

## What this replaces

- `value_to_rust_literal()` → `CodeOp::Literal` + `CodeRenderer::render_literal`
- `OutputMatcher::to_check_code()` → `CodeOp::AssertEq/AssertTrue` + renderer
- `cardinality_case_mock_value()` → `CodeOp::Literal` (same path, no special case)
- All `format!("assert_eq!(...)")` in phase generators → DAG construction

## What this does NOT replace

The obligation collector (`collect_obligations`) and DAG analysis
(`analyze_dag`) remain unchanged — they produce structured data that feeds
into DAG construction. The change is purely in the emission layer.
