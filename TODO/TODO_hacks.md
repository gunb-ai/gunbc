# Testgen Codegen Hacks

**Status**: In Progress
**Date**: 2026-02-03

Hacks and fallbacks discovered during node_example enforcement coverage.
Each item was introduced because the codegen can't serialize certain Rust
constructs to generated test code. They're not blocking — tests compile
and run — but they reduce confidence in what's actually being verified.

---

## 1. Satisfies matcher emits a comment, not an assertion

**Where**: `core/test/src/mock_spec.rs` `OutputMatcher::Satisfies` arm of
`to_check_code()`

**What happens**: `Satisfies` carries a `predicate: fn(&Value) -> bool`
closure, but closures can't be serialized to Rust source. So `to_check_code()`
emits:

```rust
// Custom assertion: skip is a boolean
```

The generated test binds the output variable and checks it exists (via
`.expect()`), but **never asserts anything about the value**. There are
~8 CI node examples and ~2 makegen examples using `Satisfies` today.
They provide false confidence — they look like they test something, but
the actual predicate is never evaluated.

**Why it matters**: Someone reading the generated test sees `output_skip`
is bound and assumes it's checked. It isn't. If the node starts returning
the wrong type, nothing catches it.

**Root cause**: Codegen operates at the source-text level. It can't
embed a closure reference into generated Rust code without some form of
indirection.

**Possible approaches**:

1. **Runtime callback**: Instead of inlining the predicate, emit code
   that calls back into the MockSpec at test time:
   ```rust
   let spec = ci_mock_spec();
   spec.node_examples[3].outputs["skip"].check(output_skip);
   ```
   This requires `OutputMatcher::check(&self, &Value) -> Result<(), String>`
   (which already exists as `OutputMatcher::matches()`) and generating
   the index/lookup code. Downside: generated tests depend on MockSpec
   construction being deterministic and index-stable.

2. **First-class matcher variants**: The common patterns (`is_bool`,
   `is_non_negative_int`, `matches_type`) could be `OutputMatcher`
   variants with known codegen. `Satisfies` would remain as an escape
   hatch for truly custom predicates, but most uses would migrate to
   the typed variants:
   ```rust
   OutputMatcher::IsType(ValueType::Bool)
   OutputMatcher::IntRange { min: 0, max: None }
   ```
   These can be serialized to `assert!(matches!(output, Value::Bool(_)))`.

3. **Hybrid**: Emit a `todo!("custom assertion: ...")` so the test
   explicitly fails/panics rather than silently passing. Forces the
   author to either use a serializable matcher or acknowledge the gap.

**Affected files**:
- `ci/graph_mock.rs` — `prepare_codegen_cmd`, `prepare_build`,
  `prepare_test`, `parse_clippy_lint` (2 outputs each)
- `makegen/graph_mock.rs` — `load_registry` `tool_count`

---

## 2. Parse nodes tested via Value::Skipped only verify skip propagation

**Where**: `bootstrap/graph_mock.rs` (`parse_scan_result`),
`ci/graph_mock.rs` (`parse_deps_exists`, `parse_codegen_exists`),
`lib/llm-ops/src/graph_mock.rs` (all 4 `parse` examples)

**What happens**: These node examples provide `input("response",
Value::Skipped)` and check `OutputMatcher::Any`. The node's skip-handling
path runs (`if matches!(input, Value::Skipped) { return all-Skipped }`),
but the **actual parsing logic is never exercised** at the integration
test level.

The parsing logic IS tested in unit tests in `ops.rs` for each tool.
So this isn't untested code — it's untested *at the generated-test layer*.

**Why it matters**: The whole point of node examples is to verify nodes
work in DAG context with realistic I/O. Testing only the skip path
doesn't do that.

**Root cause**: `value_to_rust_literal()` can't serialize
`Value::Response(TransportResponse::Shell(ShellResponse { ... }))` to
Rust source. The catch-all arm maps unknown variants to
`Value::Str("<MOCK>")` (see hack #4). So there's no way to provide a
realistic transport response as an example input in generated code.

**Possible approaches**:

1. **Extend `value_to_rust_literal()`** to handle `Value::Response` and
   `Value::Request` — emit the full constructor chain:
   ```rust
   Value::Response(gunbc_ir::transport::TransportResponse::Shell(
       gunbc_ir::transport::ShellResponse {
           exit_code: 0,
           stdout: "crates/foo\ncrates/bar\n".to_string(),
           stderr: String::new(),
       }
   ))
   ```
   This is verbose but mechanical. Each `TransportResponse` variant
   (Shell, File, Rest) needs a serializer. Once this works, examples
   can provide real responses and assert real parse outputs.

2. **Mock response builder in codegen preamble**: Emit helper functions
   at the top of the generated test module that build mock responses,
   then reference them by name in examples. This keeps the test body
   readable.

**Affected nodes** (7 total):
- `bootstrap::parse_scan_result`
- `ci::parse_deps_exists`, `ci::parse_codegen_exists`
- `llm::parse` (openai, anthropic, code_review, secrets)

---

## 3. NonEmpty matcher vacuously passes on non-string Values

**Where**: `core/test/src/mock_spec.rs` `OutputMatcher::NonEmpty` arm of
`to_check_code()`

**What happens**: Generated code is:

```rust
assert!(!output_request.as_str().map(|s| s.is_empty()).unwrap_or(false), "expected non-empty");
```

`as_str()` returns `None` for `Value::Request(...)`, `Value::Int(...)`,
`Value::Bool(...)`, etc. When it returns `None`, `.unwrap_or(false)` makes
the inner expression `false`, then `!false` is `true`, and the assertion
passes. So `NonEmpty` on a non-string value **always succeeds** without
actually checking anything.

**Where it's used on non-string outputs**:
- `bootstrap/graph_mock.rs` — `prepare_scan_workspace` output `request`
  (this is a `Value::Request`, not a string)
- `makegen/graph_mock.rs` — `load_registry` output `tool_names`
  (this is a `Value::StrList`)
- `ci/graph_mock.rs` — `prepare_codegen_exists` output `request`
- `lib/llm-ops/src/graph_mock.rs` — all `prepare` outputs `request`

For `Value::StrList`, `as_str()` also returns `None`, so even list
non-emptiness isn't checked.

**Possible fix**: Generate a type-aware non-empty check:

```rust
assert!(
    match output_request {
        Value::Str(s) => !s.is_empty(),
        Value::StrList(v) => !v.is_empty(),
        Value::Unit => false,
        Value::Skipped => false,
        _ => true, // Request, Response, Int, Bool, Json, etc. are non-empty by existence
    },
    "expected non-empty value"
);
```

Or add `Value::is_empty() -> bool` to the IR and emit
`assert!(!output.is_empty())`.

**Note**: The runtime `OutputMatcher::check()` method (mock_spec.rs:778)
already handles this correctly — it matches on `Value::Str`, `Value::List`,
and treats other types as non-empty by existence. The fix for codegen is
just mirroring that logic in `to_check_code()`.

---

## 4. value_to_rust_literal catch-all silently degrades unknown variants

**Where**: `core/codegen/src/testgen/codegen.rs:1782`

```rust
_ => "Value::Str(\"<MOCK>\".to_string())".to_string(),
```

**What happens**: Any `Value` variant not explicitly handled (`Request`,
`Response`, `Map`, `MapStrStr`, etc.) becomes `Value::Str("<MOCK>")`
in generated code. If someone writes:

```rust
NodeExample::new("my_node")
    .input("request", Value::Request(TransportRequest::Shell(cmd)))
```

The generated test would pass `Value::Str("<MOCK>")` as input. The node
would fail with a confusing "missing request" error, not a clear
"unsupported Value variant in codegen" error.

**Why it matters**: This is a silent data corruption path. The codegen
should fail loudly when it encounters a variant it can't serialize, not
silently substitute a string placeholder.

**Possible fix**: Replace the catch-all with a `panic!()`:

```rust
other => panic!(
    "value_to_rust_literal: unsupported Value variant {:?}. \
     Add serialization support or use Value::Skipped as a placeholder.",
    std::mem::discriminant(other)
),
```

Or, if we want to keep codegen infallible, emit code that fails at
test compile time:

```rust
_ => "compile_error!(\"unsupported Value variant in node example\")".to_string(),
```

This is the simplest fix and could be done independently of the others.

**Related**: `Value::List` serialization has a separate silent corruption
path. The `Value::List(list)` arm uses `filter_map(|v| v.as_str())`,
which silently drops any non-string elements. `Value::List(vec![Value::Int(1)])`
would serialize to `Value::str_list(vec![])` — empty list, no error.
Fix: either assert all elements are strings, or serialize recursively
over `Value` variants.

---

## 5. Cardinality "Empty" tests don't test absence

**Where**: `core/codegen/src/testgen/codegen.rs` `cardinality_case_mock_value()`

**What happens**: For the `CardinalityCase::Empty` case, the function
generates concrete values with "empty content" rather than actual absence:

```rust
CardinalityCase::Empty => match type_id {
    "String" => "Value::Str(String::new())".to_string(),   // empty string
    "Bool"   => "Value::Bool(false)".to_string(),           // false
    "Int"    => "Value::Int(0)".to_string(),                // zero
    _        => "Value::List(vec![])".to_string(),          // empty vec
}
```

Under set/tape semantics, a `[0,1]` cardinality port with `Empty`
should have **zero elements** (absent), not **one element with empty
content**. `false` is still a `Bool` — it's one element, not zero.
The generated "empty" tests actually exercise the "one" case.

**Why it matters**: The cardinality boundary tests (Bucket B.3) are
meant to verify DAG behavior at cardinality boundaries. If Empty
doesn't test absence, the most valuable boundary (present vs absent)
is never exercised. This blocks property-based and boundary-based
testing from catching real cardinality bugs.

**Example from generated code**: For a `[0,1]` Bool port, the generated
"empty" test does `mocks.set_value("node", "port", Value::Bool(false))`.
This is indistinguishable from "one element that happens to be false."

**Root cause**: `BoundaryMocks` has no way to represent "absent." It
stores `HashMap<String, HashMap<String, Value>>` — every entry is
present with a concrete `Value`. There's no `unset_value()` or
`Option<Value>` semantics.

**Possible approaches**:

1. **`Option<Value>` in BoundaryMocks** — Change mock storage to
   `HashMap<String, HashMap<String, Option<Value>>>`. `None` means
   absent, `Some(v)` means present. Executor treats `None` as
   zero elements.

2. **`Value::Absent` variant** — Add an explicit variant that the
   executor interprets as "this port has no value." Simpler than
   `Option<Value>` because it doesn't change the map type, but
   adds a variant to a core enum.

3. **`unset_value()` API on BoundaryMocks** — Store absent ports
   separately (e.g., `HashSet<(String, String)>` of removed ports).
   `set_value()` adds, `unset_value()` removes.

**Affected tests**: All Bucket B.3 "empty" tests for scalar types
(`Bool`, `Int`, `String`). List-typed ports already use
`Value::List(vec![])` which is arguably correct (zero elements
represented as empty collection).

---

## Tasks

- [x] Hack 1: Replace `Satisfies` comment-only codegen with typed matcher variants
  - Added `IsBool`, `IsInt`, `IsString`, `IsRequest`, `IsResponse`, `IntGe(i64)`, `IntLe(i64)` to `OutputMatcher`
  - All typed matchers generate real assertions in codegen (via `Assert::True` with method chains)
  - Migrated existing `Satisfies` usages in `ci/graph_mock.rs` and `makegen/graph_mock.rs`
  - `Satisfies` remains as escape hatch for truly custom predicates
- [x] Hack 2: Add `Value::Response`/`Value::Request` support (done via ValueExpr pipeline)
- [ ] Hack 2: Update parse node examples to use real transport responses
- [x] Hack 3: Make `NonEmpty` codegen type-aware (or add `Value::is_empty()`)
- [x] Hack 4: Replace `value_to_rust_literal` catch-all with `panic!()` or `compile_error!()`
- [x] Hack 4: Fix `Value::List` filter_map silent dropping of non-string elements
- [x] Hack 5: Make cardinality Empty tests represent absence, not empty content
  - `cardinality_case_mock_value()` now emits `Value::Unit` for scalar Empty
    (String, Bool, Int) instead of concrete "empty content" (`false`, `0`, `""`)
  - Collection types (List, Set) still use empty collections (correct: zero elements)
  - Aligns with `contract::witnesses()` which already uses `Value::Unit` for absence
  - Cardinality boundary tests now exercise the actual absent-vs-present boundary

## Notes

- **Root cause fixed**: Hacks 2, 3, and 4 were all consequences of the
  same limitation: `value_to_rust_literal()` didn't cover all `Value`
  variants. This is now resolved — the `ValueExpr` intermediate
  representation handles every `Value` variant exhaustively (including
  `Request`/`Response` transport types), rendered via `RustRenderer`.
  The old `value_to_rust_literal` is a one-liner delegating to this
  pipeline; the old `value_to_code` and `to_check_code` in mock_spec.rs
  have been deleted as dead code.
- **Hack 1 resolved**: Typed matcher variants (`IsBool`, `IsInt`, etc.)
  now generate real codegen assertions. These are serializable to Rust
  source without closure references. `Satisfies` remains only for
  complex predicates that can't be expressed as simple type/range checks.
  Existing usages in CI and makegen graph mocks have been migrated.
- **Hack 5 resolved**: Scalar Empty now uses `Value::Unit` (absence),
  matching the behavior of `contract::witnesses()`. No infrastructure
  change to `BoundaryMocks` was needed — `Value::Unit` already serves
  as the "absent" signal within the existing `Value` type.
- Only hack 2 (parse node examples with real transport responses) remains.
  This requires serializing full transport response constructors in
  generated test code, which the `ValueExpr` pipeline already supports.
