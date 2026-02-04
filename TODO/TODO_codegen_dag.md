# Codegen-on-DAG Migration

**Status**: Design
**Date**: 2026-02-03

## Problem

The current pipeline is structured everywhere except the last step:

```
DAG definition  (structured, typed)
  → DagAnalysis   (structured — ports, edges, cardinalities)
  → ObligationSet (structured — 4 buckets, proof statuses)
  → Phase generators (structured — 7 phases, clear responsibilities)
  → format!() strings    ← cliff
  → Generated Rust source
```

`codegen.rs` (2500 lines) converts structured obligation/analysis data
into generated test code via raw `format!()` string interpolation. This
is where every testgen hack lives: `value_to_rust_literal`'s catch-all
arm, the `Satisfies` comment-not-assertion, the `List` `filter_map` that
drops non-string elements. The compiler can't enforce exhaustiveness on
string templates, and the entire emission layer is hardcoded to Rust
syntax.

Additionally, there are two parallel functions doing the same thing:
`value_to_rust_literal()` in codegen.rs and `value_to_code()` in
mock_spec.rs. Both had silent catch-all arms. Both need the same
fix. This duplication is a symptom of having no shared value-rendering
abstraction.

## Observation

The codebase already models types as DAGs (`Dag<TypeOp>`), patterns as
DAGs (`Dag<PatternOp>`), and languages as DAGs (`Dag<LanguageOp>`). Code
emission is the one pipeline stage that breaks out of this model into
ad-hoc strings.

## Design: a minimal cross-language expression language

The goal is not just a codegen refactor — it's to define a small language
of operations that every target language (Rust, Python, TypeScript, etc.)
can express. Each operation should be simple enough that its semantics are
unambiguous across languages. Operations that require careful per-language
negotiation (e.g., integer overflow, string encoding, struct construction)
should be explicit so they can be litigated once per backend.

### Type language (proto-like, minimal)

The type language is the intersection of what every target language can
express without ambiguity. Think protobuf — deliberately small, maps
cleanly everywhere. Complexity goes into backends, not into the core.

**In scope:**

| Type | Rationale |
|------|-----------|
| `bool` | Universal. |
| `i64` | Single integer type. No unsigned — it's implementable per-backend but adds complexity with zero benefit to test code. |
| `string` | Universal. |
| `json` | Escape hatch for unstructured data. Render as `serde_json::json!()` / dict literal / plain object. Keep as explicit variant rather than decomposing into primitives. |
| `list(T)` | Homogeneous ordered collection. |
| `map(string, T)` | Dynamic string-keyed collection (proto `map<string, T>`). |
| `struct { name, fields }` | Named product type with statically known fields. Distinct from map — backends render as Rust struct / Python dataclass / TS interface. |
| `unit` | Absence of data — null/None/undefined/Unit. |

**Out of scope (and why):**

- **Unsigned integers** — Python doesn't distinguish, JS numbers are
  floats. Implementable per-backend but doesn't help test code.
- **Pointers/references** — no target language needs them for test
  assertions.
- **Enums/sum types** — would need per-language pattern matching. Model
  as struct + tag field if needed.
- **Generics** — proto doesn't have them. The cardinality system handles
  "zero or more of T."

### Cardinality as the typing system

Cardinality is not part of the type — it's metadata on the *port*. A
port has a base type (string, bool, struct Foo) and a cardinality
interval ([0,1], [1,∞), etc.). Whether something is "optional" or
"repeatable" falls out of the interval:

- `[0,0]` — absent (Hack #5: this is what Empty should test)
- `[0,1]` — optional: Rust `Option<T>`, Python `Optional[T]`, TS `T | undefined`
- `[1,1]` — required scalar (default)
- `[0,∞)` — optional repeatable: Rust `Vec<T>`, Python `list[T]`, TS `T[]`
- `[1,∞)` — required repeatable

The backend picks the idiomatic representation based on the interval,
not based on the type being called "List". This is already how
`DagAnalysis` works internally (via `allows_empty()`, `test_cases()`) —
it just isn't wired through to emission. Struct/object types in each
language can leverage cardinality to decide field optionality, collection
wrapping, etc.

### Two layers: ValueExpr and CodeOp

Separate value representation from code structure. Values are data
(literals, collections, structs). Code is control flow and assertions.

**ValueExpr** — what a value looks like in the target language:

```
ValueExpr::Unit                       — null/None/undefined/Unit
ValueExpr::Bool(bool)                 — true/false/True/False
ValueExpr::Str(String)                — "hello" / 'hello'
ValueExpr::Int(i64)                   — 42
ValueExpr::List(Vec<ValueExpr>)       — [a, b, c] / vec![a, b, c]
ValueExpr::Map(Vec<(String, ValueExpr)>) — {k: v} / BTreeMap::from(...)
ValueExpr::Json(serde_json::Value)    — serde_json::json!() / JSON.parse()
ValueExpr::Struct { name, fields }    — MyStruct { a, b } / { a, b }
```

Converting `Value → ValueExpr` is a total function — the compiler forces
exhaustive handling. No catch-all possible. `Request`/`Response` must be
explicitly modeled as structs (they have known fields) rather than
silently degraded to `"<MOCK>"`.

**CodeOp** — what a test does, independent of language:

```
CodeOp::Let(name, ValueExpr)          — bind a value to a name
CodeOp::FieldAccess(name)             — access a field/key
CodeOp::Call(fn_name, Vec<CodeOp>)    — call a function with args
CodeOp::AssertEq(Box<CodeOp>, Box<CodeOp>)  — equality assertion
CodeOp::AssertTrue(Box<CodeOp>)       — truthiness assertion
CodeOp::AssertNonEmpty(Box<CodeOp>)   — non-emptiness assertion
CodeOp::Construct(kind, Vec<CodeOp>)  — build a List/Map/Struct
CodeOp::TestBlock(name, Vec<CodeOp>)  — wrap body in a test fn
CodeOp::Comment(String)               — emit a comment
```

Each variant maps to a simple, well-defined operation that every language
can express. No variant requires language-specific knowledge. The ports on
each node carry `TypeId` and `Cardinality` as they do today, so the
existing type/cardinality checking infrastructure applies.

### Cross-language semantics to litigate

These are operations where languages differ and the backend must make
explicit choices:

| Operation | Rust | Python | TypeScript |
|---|---|---|---|
| Integer literal | `42_i64` | `42` | `42` |
| Empty string | `String::new()` | `""` | `""` |
| List construction | `vec![...]` | `[...]` | `[...]` |
| Map construction | `BTreeMap::from([...])` | `{...}` | `new Map([...])` |
| Struct construction | `Foo { a, b }` | `Foo(a=.., b=..)` | `{ a, b }` |
| Equality assertion | `assert_eq!(a, b)` | `assert a == b` | `expect(a).toEqual(b)` |
| Non-empty assertion | `assert!(!x.is_empty())` | `assert x` or `assert len(x)` | `expect(x).toBeTruthy()` |
| Test function | `#[test] fn name()` | `def test_name():` | `test('name', () =>)` |

The backend trait makes these choices explicit:

```
trait CodeRenderer {
    fn render_value(&self, expr: &ValueExpr) -> String;
    fn render_assert_eq(&self, left: &str, right: &str) -> String;
    fn render_assert_true(&self, expr: &str) -> String;
    fn render_assert_non_empty(&self, expr: &str) -> String;
    fn render_test_block(&self, name: &str, body: &str) -> String;
    fn render_var_bind(&self, name: &str, expr: &str) -> String;
    fn render_call(&self, fn_name: &str, args: &[String]) -> String;
    fn render_field_access(&self, expr: &str, field: &str) -> String;
    fn render_imports(&self) -> String;
}
```

`render_value` replaces both `value_to_rust_literal` and `value_to_code`.
Adding a new `Value` variant breaks all backends at compile time.

### Migration path

Language stubs (Python, TypeScript) should exist from Phase 0 — they
validate the abstraction. If a CodeOp variant can't be cleanly expressed
in a stub backend, the abstraction is wrong and we find out immediately,
not at Phase 4.

1. **Phase 0**: Define `ValueExpr`, `CodeOp`, `CodeRenderer` trait.
   Implement Rust backend. Add Python and TypeScript stubs that panic
   with `todo!("not yet implemented")` on every method — but the type
   signatures must compile. This validates the trait surface.
2. **Phase 1**: Migrate `value_to_rust_literal` and `value_to_code` →
   `Value → ValueExpr → render_value`. Consolidates the two duplicated
   functions into one path. Immediately fixes Hacks 3, 4, and the
   List bug.
3. **Phase 2**: Migrate assertion emission (`to_check_code`) →
   `CodeOp::AssertEq` / `CodeOp::AssertTrue`. Fixes Hack 1 (Satisfies).
4. **Phase 3**: Migrate test function scaffolding → `CodeOp::TestBlock`.
   After this, `codegen.rs` contains zero `format!()` calls for code
   emission.

Each phase is independently shippable. After Phase 1, the catch-all hack
is structurally impossible. After Phase 3, adding a new test phase or a
new target language is a matter of implementing a trait, not writing a
template engine.

## What this replaces

- `value_to_rust_literal()` + `value_to_code()` → `ValueExpr` +
  `CodeRenderer::render_value` (consolidates two duplicated functions)
- `OutputMatcher::to_check_code()` → `CodeOp::AssertEq/AssertTrue` +
  renderer
- `cardinality_case_mock_value()` → `ValueExpr` (same path, no special
  case)
- All `format!("assert_eq!(...)")` in phase generators → CodeOp
  construction

## Analysis data is currently severed from emission

Five phase generators accept `&DagAnalysis` and immediately discard it
(`_analysis`). The cardinality algebra exists — `allows_empty()`,
`test_cases()`, `satisfies()`, the full lattice — but `codegen.rs` never
consults it. Instead, `cardinality_case_mock_value()` reconstructs
cardinality behavior from ad-hoc `type_id` string matching (`"String"`,
`"Bool"`, `"Int"`), ignoring the port's actual `Cardinality` interval.

The migration must wire `DagAnalysis` into the `CodeOp` construction
phase so that cardinality-driven decisions (empty case generation, list
vs scalar emission, optional handling) come from the structured analysis,
not from string heuristics. Concretely:

- `ValueExpr` nodes should carry the port's `Cardinality`, not just a
  `type_id` string.
- Mock value generation should use `cardinality.allows_empty()` and
  `cardinality.test_cases()` instead of type-name pattern matching.
- The `_analysis` parameters should become live inputs to CodeOp
  construction, not dead signatures.

## What this does NOT replace

The obligation collector (`collect_obligations`) and DAG analysis
(`analyze_dag`) remain unchanged — they produce structured data that
feeds into CodeOp construction. The change is purely in the emission
layer, but the emission layer must actually *consume* the analysis data
it receives.
