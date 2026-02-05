# Type System Evolution — Completed Items

**Status**: ✅ Done
**Date**: 2026-02-05

Completed milestones from the type-system roadmap. Remaining work is tracked
in `TODO/TODO_type_system.md`.

---

## Emptiness consolidated on cardinality (codegen path)

**Resolved**: Non-empty checks now use `Value::is_empty()` via `Assert::NonEmpty`,
so list/set/string emptiness is handled consistently. The codegen path no
longer relies on string-only `as_str()` checks.

Files:
- `core/ir/src/value.rs` (Value::is_empty)
- `core/codegen/src/testgen/codegen.rs` (Assert::NonEmpty emission)
- `core/codegen/src/testgen/test_ir.rs` (Assert::NonEmpty)

## Codegen now models language elements as structured data

**Resolved**: Test generation is IR-driven (Expr/Stmt/Assert + ValueExpr)
with renderer backends; no silent fallbacks in string templates.

Files:
- `core/codegen/src/testgen/test_ir.rs`
- `core/codegen/src/testgen/render.rs`, `render_rust.rs`
- `core/codegen/src/testgen/render_python.rs` (stub), `render_ts.rs` (stub)

## Set is a concrete type

**Resolved**:
- `Value::Set` with set algebra + helpers.
- `WrapperKind::Set` / `NonEmptySet` in type system.
- `SetOp` primitives.

Files:
- `core/ir/src/value.rs`
- `core/ir/src/type_op.rs`, `core/ir/src/type_lib.rs`
- `lib/primitives/src/collection.rs`

## Boundary witness generation from type contracts

**Resolved**: `contract::witnesses()` generates boundary witnesses from type DAGs.

Files:
- `core/ir/src/contract.rs`

## Cardinality-driven CLI generation

**Resolved**:
- `CliEntrypoint` now carries `Cardinality`.
- CLI generation derives repeatable/optional from cardinality rather than
  `type_id == "List"` heuristics.
- Loop pattern defaults now use element type + cardinality.

Files:
- `core/codegen/src/cli_gen.rs`
- `gunbc-dag/src/makegen/registry.rs`
- `core/ir/src/patterns/loop_pattern.rs`

---

## Additional completed items (from review session)

- Typed `OutputMatcher` variants (`IsBool`, `IsInt`, `IsString`, `IsRequest`, `IsResponse`, `IntGe`, `IntLe`) now generate real assertions.
- `ShellResponse::ok()` / `failed()` helpers added for transport modeling.
- `From<T> for TransportRequest/TransportResponse` impls added for all transport types.
- `ExecError::context()` + `ResultExt` for structured error context.
- `propagate_skipped()` helper added and migrated across call sites.
- Codegen Empty case now uses `Value::Unit` for scalar absence.
