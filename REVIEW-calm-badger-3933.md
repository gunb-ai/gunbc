# Review: calm-badger-3933

**Branch:** `origin/calm-badger-3933`
**Reviewer:** claude/review
**Date:** 2026-03-13

## Summary

This branch implements ~15 sustainability items (S11, S12, S14, S15, S16, S17,
S22, S26, S31, S32, S33, S40, S41, S44, S45, S46, S48, S49, S57, S66, S67,
S70, S74) plus IR cleanup (LoweredExpr::Return removal). Net: +3,998 / −2,451
lines across 65 files. The consolidation work is directionally excellent — many
of these fixes are long overdue. However, the branch has several bugs (including
5 failing tests) and introduces new invariant violations that should be
addressed before merge.

---

## Part 1: Bugs

### BUG-1 (High): 5 failing tests — `mock_corpus` strict-mode regression

**Files:** `src/v1/01_surfaces/codegen/src/testgen/mock_corpus.rs`
**Root cause:** S32 changed `DryRunStrictness` default from `Lenient` to `Strict`

`cargo test --workspace --lib` produces 5 failures:
- `testgen::mock_corpus::tests::single_workflow_produces_corpus`
- `testgen::mock_corpus::tests::multi_workflow_accumulates`
- `testgen::mock_corpus::tests::edge_examples_captured`
- `testgen::mock_corpus::tests::effectful_nodes_get_type_contract_only`
- `testgen::mock_corpus::tests::pure_nodes_get_exact_outputs`

All fail with: `strict mode: node 'mod_b::node_b' is missing 1 required input`.
The test DAG wires `node_a.out1 → node_b.in1` via an edge, and mocks only
`node_a.in1`. Under strict mode the pre-execution check rejects `node_b`
because its input arrives via edge resolution (not explicit mock). The tests
expect lenient behavior.

**Fix:** Either set `DryRunStrictness::Lenient` explicitly in `build_corpus`'s
`ExecuteConfig`, or update the test DAGs to include explicit mocks for all
inputs.

### BUG-2 (High): `DryRunStrictness::Strict` default affects non-test callers

**File:** `src/v1/09_execute/exec/src/execute/mod.rs`

The `#[default]` moved from `Lenient` to `Strict`, but multiple production
callers use `..Default::default()` without setting `strictness`:
- `daglang-cli/src/compile/context.rs:136` — CLI execution path
- `codegen/src/testgen/mock_corpus.rs:113` — mock corpus generation
- `test/src/boundary.rs:134` — `execute_via_engine_with_inputs`
- `daglang-emit/src/rust_exec_runtime.rs:1059` — generated Rust sources

These callers will silently start rejecting DAGs with unwired inputs. The strict
check also fires regardless of execution mode (Real vs DryRun), despite the name
`DryRunStrictness` implying dry-run-only semantics.

### BUG-3 (High): Transport mock tests emitted for Prepare/Parse phases

**File:** `src/v1/07_emit/daglang-emit/src/lib.rs` — `transport_mock_entries_from_classified`

The old code path only emitted mock tests for `TransportObligation::Execute`
nodes (documented invariant at `test_gen.rs:69-71`). The new classified path
iterates `input.transports` unconditionally, and `classify_for_emit` pushes
every `LoweredOp::Transport` variant regardless of phase. Prepare and Parse
nodes now get spurious mock entries across all backends.

**Fix:** Filter `input.transports.iter().filter(|t| t.obligation == TransportObligation::Execute)`.

### BUG-4 (Medium): `Option` vs `Optional` naming split in type system

**File:** `src/v1/00_foundation/ir/src/types.rs`

`TypeId::option()` now emits `"Optional<T>"`, but `BUILTIN_TYPES` registers the
type under `"Option"` (no `"Optional"` entry). `BuiltinType::lookup("Optional")`
returns `None`. Once `validate_port_type_ids()` is wired into the pipeline, any
port typed via `TypeId::option()` will be flagged as unregistered.

**Fix:** Add `"Optional"` as the registered name (or an alias) in `BUILTIN_TYPES`.

### BUG-5 (Medium): `Float` backing maps to `OutputMatcher::IsInt`

**File:** `src/v1/00_foundation/ir/src/types.rs` — `output_matcher_path()`

```rust
ValueBacking::Int | ValueBacking::Float => Some("gunbc_test::OutputMatcher::IsInt"),
```

Float-backed types will generate integer matchers in test output, accepting
`42` but rejecting `3.14`.

**Fix:** Separate the arms — `Float => Some("gunbc_test::OutputMatcher::IsFloat")`.

### BUG-6 (Medium): `sum` alias arity contradicts canonical `Fold` contract

**File:** `src/v1/00_foundation/ir/src/patterns/collection.rs`

`from_name_or_alias("sum")` resolves to `CollectionKind::Fold` (arity 3:
`[collection, init, f]`). But `alias_contracts()` declares `"sum"` with arity 1
(`[collection]`, output `"Int"`). Call-site arity validation will see different
answers depending on lookup path.

### BUG-7 (Low): `strip_optional_wrapper` misses `OptionalT` concatenated form

**File:** `src/v1/00_foundation/ir/src/value_bridge.rs`

`optional_inner_type_id()` in `types.rs` recognizes `T?`, `Optional<T>`, and
`OptionalT`. But `strip_optional_wrapper` in `value_bridge.rs` handles `T?`,
`Optional<T>`, and legacy `Option<T>` — but **not** `OptionalT`. Concatenated
forms won't be recognized as optional during bridge deserialization.

### BUG-8 (Low): `is_eval_intrinsic` missing builtins

**File:** `src/v1/00_foundation/ir/src/patterns/collection.rs`

`non_collection_builtin_contracts()` defines contracts for `max_by`,
`replace_section`, `to_bytes`, `to_json`, and `hash`, but `is_eval_intrinsic()`
does not include them. These operations will fall through to non-intrinsic
dispatch. Also, `"any"`, `"all"`, `"contains"` are dead code in
`is_eval_intrinsic` since they're already covered by `CollectionKind::from_name`.

---

## Part 2: Invariant Violations (src/v1/README.md)

### INV-1: No fallbacks that fabricate — `parse_response_provider` / `parse_shell_output_parsing`

**Files:** `src/v1/05_graph/daglang-lower/src/lib.rs:6512-6520` and `:6985-6994`
**Also flagged by:** PR comment feedback (codex-connector)

Both functions return `None` for unrecognized strings, and callers silently fall
back to inference:
```rust
.and_then(parse_response_provider)
.or_else(|| infer_response_provider(&service.name));
```

A typo like `response_provider: "OpenAii"` compiles but silently falls back to
name-based inference, potentially selecting the wrong provider. Explicit
user-authored annotations should fail fast on unrecognized values, not degrade
to inference.

**Fix:** Return `Result` from the parse functions; propagate `Err` as a
`LowerError` when the annotation was explicitly provided.

### INV-2: No parallel implementations — duplicate `ResponseProvider` parsers

**Files:** `daglang-lower/src/lib.rs:parse_response_provider` and
`ir/src/transport/middleware.rs:impl FromStr for ResponseProvider`

Two independent parsers for the same enum, **already divergent**:
- `FromStr` accepts `"git_hub"`, `"open_ai"` — lowerer does not
- Lowerer accepts `"OpenAI"` — `FromStr` does not

The lowerer should call `name.parse::<ResponseProvider>()` instead of
maintaining its own match. Same applies to `ShellOutputParsing` — should have
`impl FromStr` on the enum, not a parser in the lowerer.

### INV-3: No fallbacks that fabricate — `"TransportResponse"` default

**File:** `src/v1/07_emit/daglang-emit/src/lib.rs`

```rust
let response_type = node.outputs.first()
    .map(|p| p.type_id.0.clone())
    .unwrap_or_else(|| "TransportResponse".to_string());
```

A transport node with zero output ports fabricates a type string. This is the
FC-7 antipattern. Should either fail or be structurally impossible (the
`EmitTransport` type should require outputs).

### INV-4: No fallbacks that fabricate — `.ok()` swallowing in testgen

**File:** `src/v1/07_emit/daglang-emit/src/test_mock_emit.rs`

```rust
let backing = gunbc_ir::value_backing_for_type_id(type_id).ok()?;
```

Silently drops type-resolution errors. Already tracked as S24.

### INV-5: No duplicate representations — `is_eval_intrinsic` string list

**File:** `src/v1/00_foundation/ir/src/patterns/collection.rs`

Hand-maintained string list must be kept in sync with
`non_collection_builtin_contracts()`. Adding a new builtin requires editing
both. Should derive `is_eval_intrinsic` from the contracts registry.

### INV-6: Explicit boundary contracts — new thread-local `TYPE_WARNINGS`

**File:** `src/v1/05_graph/daglang-eval/src/eval_stack.rs`

S57 type-boundary diagnostics use `thread_local!` hidden state. This is a new
instance of the S71 antipattern (thread-local `TmpCounter` in emit). Callers
must remember to call `take_type_warnings()` after `evaluate_stack()`.
Warnings are silently lost if another evaluation runs without draining.

**Fix:** Return warnings as part of `evaluate_stack`'s return type.

### INV-7: No case enumeration for open sets — `parse_shell_output_parsing` in lowerer

**File:** `src/v1/05_graph/daglang-lower/src/lib.rs:6985`

`ShellOutputParsing` is a closed enum but the string→variant mapping lives in
the lowerer rather than as `impl FromStr` on the enum itself. A second consumer
would need to duplicate the match.

---

## Part 3: Positive observations

- **S16 transport consolidation** is well done — `TransportTripletSpec` +
  `build_transport_triplet` genuinely eliminates 4-way duplication.
- **S57 runtime type checks** are a valuable interim guard, despite the
  thread-local delivery mechanism.
- **LoweredExpr::Return removal** is a clean simplification.
- **S67 Unknown→Inferred rename** with `is_inferred()` guard is a good step.
- **S70 emit classification** (`classify_for_emit` + typed structs) is a
  meaningful improvement over raw variant matching.
- **S26 LLM provider registry** consolidation is solid.
- **CollectionKind registry** (S11) is the right approach.

---

## Recommended priority

1. **Fix the 5 failing tests** (BUG-1/BUG-2) — decide on strict vs lenient default
2. **Fix transport mock phase filtering** (BUG-3)
3. **Fix `Option`/`Optional` naming split** (BUG-4)
4. **Fix silent fallback on explicit annotations** (INV-1/INV-2) — use `FromStr` + fail fast
5. **Fix `Float` → `IsInt` matcher** (BUG-5)
6. Remaining items are lower priority / can be follow-up
