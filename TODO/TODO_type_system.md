# Type System Evolution

**Status**: In Progress
**Date**: 2026-02-04
**Updated**: Reflects design decisions and implementation from review session.

Directions for making the type system more structural.
These build on existing work (cardinality algebra, `TypeContract`,
`type_id == "List"` migration).

---

## 1. Types are DAGs — structural comparison, not string matching

**Design decision**: Types are already modeled as DAGs (`Dag<TypeOp>`) in
`core/ir/src/type_op.rs`. The four operations are `Identity`, `Validate`,
`Transform`, and `Wrap`. Type constructors in `type_lib.rs` compose these.

**Current gap**: Ports reference types by `type_id: TypeId` (a String wrapper)
and require a `TypeRegistry` lookup to get the actual DAG. Type compatibility
in `TypeRegistry::is_compatible()` compares base type + cardinality but
ignores predicates. So `Url` and `String` are incompatible by name even
though Url is a refinement of String.

**Goal**: Ports should carry (or directly reference) the type DAG, and
comparison should be structural subsumption — "is this type DAG a refinement
of that type DAG?" This is not a newtype wrapper; it's moving from name-based
to structure-based type identity.

**What it enables**:
- Edge compatibility becomes compositional (contract subsumption)
- Refinement types work naturally (Url satisfies String)
- CLI generation derives `repeatable` from cardinality, not type names

**Status**: Not started (requires deeper architectural change to Port).
The `TypeId` newtype already exists as a String wrapper.

---

## 2. Emptiness consolidates on cardinality

**Design decision**: Emptiness is a property of the cardinality model, not
a standalone concept. `Cardinality` already has `allows_empty()` (`min == 0`),
a full lattice algebra in `algebra.rs`, and `test_cases()` generating
`Empty/One/Many` case variants.

The remaining work is wiring cardinality through to places that currently
use ad-hoc emptiness checks (e.g., `OutputMatcher::NonEmpty` in codegen
uses `as_str()` — a string check, not a cardinality check). A `Value::is_empty()`
helper could exist as a convenience, but it should delegate to cardinality
semantics, not define its own.

**Note**: The codegen-specific wiring (items 2, 3 from hacks) is being
addressed separately as part of the codegen rework.

---

## 3. Codegen needs structured language modeling

**Design decision**: The `Satisfies` closure / `value_to_rust_literal`
catch-all problem points to a deeper issue — codegen doesn't model
target language elements as structured data the way we model other
interfaces (git, gist).

**Goal**: Model Rust (and eventually Python, TS) language elements
as structured data — the same way we model git ops or gist ops.
Something like a `RustExpr` enum with variants for literals, assertions,
function calls, etc. Codegen builds that data structure, then a separate
renderer emits text. This makes serialization exhaustive (no catch-all)
and new variants force compiler errors instead of silent degradation.

**Status**: Being addressed in a separate codegen rework effort.

---

## 4. Set is a concrete type ✅

**Design decision**: List is a list, Set is a set. `Value::Set` was added
as a concrete type with language-agnostic set algebra.

**Implemented**:
- `Value::Set(Vec<Value>)` — unordered, unique-element collection
  (`core/ir/src/value.rs`). Uniqueness enforced via `PartialEq` at
  construction time. Constructors: `Value::set()`, `Value::str_set()`.
- Set algebra methods on `Value`: `set_union()`, `set_intersection()`,
  `set_difference()`, `set_symmetric_difference()`, `is_subset()`,
  `set_contains()`.
- `WrapperKind::Set` and `WrapperKind::NonEmptySet` in the type system
  (`core/ir/src/type_op.rs`).
- Type constructors: `type_lib::set()`, `type_lib::non_empty_set()`.
- `SetOp` in collection primitives (`lib/primitives/src/collection.rs`)
  — `Union`, `Intersection`, `Difference`, `SymmetricDifference`.
  Accepts both Set and List inputs (List is auto-promoted by deduplication).
- All exhaustive match statements updated across the codebase.

---

## 5. Boundary witness generation from type contracts ✅

**Implemented**: `contract::witnesses()` generates boundary witness values
from a type DAG's contract (`core/ir/src/contract.rs`).

Given a type DAG, it:
1. Extracts base type, predicates, cardinality, and wrapper kind
2. Generates a scalar witness refined by predicates (e.g., URL type
   → `"https://example.com"` because of the `Matches` predicate)
3. Produces one `BoundaryWitness { case, value }` per cardinality case

Examples:
- `String` (cardinality ONE) → 1 witness: `Value::Str("example")`
- `Optional<String>` → 2 witnesses: `Value::Unit` (Empty), `Value::Str(...)` (One)
- `List<String>` → 3 witnesses: `[]` (Empty), `["example"]` (One), `["example_1", "example_2"]` (Many)
- `Set<String>` → 3 witnesses: `{}` (Empty), `{"example"}` (One), `{"example_1", "example_2"}` (Many)

**What's next**: Wire witnesses into testgen to replace hardcoded
`cardinality_case_mock_value()` tables. This is part of the codegen rework.

---

## 6. Cardinality-driven CLI generation ✅

**Implemented**: `CliEntrypoint` now carries a `cardinality: Cardinality`
field (`core/codegen/src/cli_gen.rs`). All CLI generation logic —
`rust_type()`, `value_constructor()`, `is_repeatable()`, arg parsing,
help text — derives collection behavior from `cardinality.allows_many()`
instead of string-matching `type_id == "List"`.

The makegen registry (`gunbc-dag/src/makegen/registry.rs`) now uses
`ep.cardinality.allows_many()` directly, eliminating the TODO that
previously marked this for replacement.

**Loop pattern fixed**: `core/ir/src/patterns/loop_pattern.rs` defaults
now use `"String"` as the type_id (element type) instead of `"List"`.
Cardinality `ZERO_OR_MORE` handles the collection semantics — this
eliminates the dual encoding where `"List"` was both a type name and
a cardinality indicator.

---

## Remaining Tasks

- [ ] Move toward structural type DAG comparison in Port (item 1)
- [ ] Wire cardinality through to codegen emptiness checks (item 2, codegen rework)
- [ ] Model target language elements as structured data (item 3, codegen rework — in progress)
- [ ] Wire `contract::witnesses()` into testgen (codegen rework)

## Completed

- [x] Add `Value::is_empty()` method to `core/ir` (item 2, convenience method)
- [x] Add `Value::Set` as concrete type with set algebra (item 4)
- [x] Add `WrapperKind::Set` / `NonEmptySet` to type system
- [x] Add `SetOp` to collection primitives
- [x] Implement `contract::witnesses()` boundary witness generation (item 5)
- [x] Replace `type_id == "List"` in non-codegen locations (item 6 partial)
- [x] Fix loop pattern dual encoding (type_id "List" → element type + cardinality)
- [x] Add `cardinality: Cardinality` field to `CliEntrypoint` (item 6)
- [x] Replace `type_id ==` string checks in CLI generation with `cardinality.allows_many()`
- [x] Add typed `OutputMatcher` variants (`IsBool`, `IsInt`, `IsString`, `IsRequest`, `IsResponse`, `IntGe`, `IntLe`) that generate real codegen assertions
- [x] Add `ShellResponse::ok()` / `failed()` constructors for transport response modeling
- [x] Add `From<T> for TransportRequest` / `TransportResponse` impls for all transport types
- [x] Add `ExecError::context()` and `ResultExt` trait for structured error context
- [x] Add `propagate_skipped()` helper for skip propagation pattern

## Design Principles (from review)

- **Types are DAGs**: String, Bool, Json are all composed on each other.
  Comparison should be structural DAG subsumption, not string equality.
- **Cardinality owns emptiness**: Don't add standalone `is_empty()` as a
  separate concept. Consolidate on the cardinality model — emptiness falls
  out naturally from `cardinality.allows_empty()`.
- **Codegen must model its target**: For any backend (Python, TS, Rust),
  model the language elements correctly as structured data — the same way
  we model git, gist, and other interfaces. String concatenation is the
  root cause of serialization hacks.
- **List is a list, Set is a set**: Don't overload List with set semantics.
  Set is a distinct type with its own algebra. Language-agnostic design
  ensures it works across backends.
