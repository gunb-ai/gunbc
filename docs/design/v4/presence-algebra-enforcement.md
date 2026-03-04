# Presence Algebra & Compile-Time Enforcement Sweep

**Status**: Draft
**Date**: 2026-03-04
**Companion**: `domain-hard-error-no-fallback-plan.md` (extern symbol linking)

## Core Invariant

> **Missing is not a value. Every absence must be an explicit, typed decision.**

Absence in this system occurs on multiple orthogonal axes. Today these axes collapse into a single runtime sentinel (`Value::Skipped`) or are untracked entirely. The result: code cannot distinguish "legitimately empty" from "missing due to a wiring bug" from "skipped by control flow."

The fix is a **presence algebra** — a type-level axis orthogonal to value type and cardinality, with explicit combinators for transitions.

## The Presence Algebra

### Axes of a port type

A fully-specified port type has three orthogonal components:

```
PortType = ValueType × Cardinality × PresenceMode
```

| Axis | What it tracks | Already modeled? |
|------|---------------|-----------------|
| **ValueType** | Domain type (`String`, `Int`, `Config`, sum types) | Yes — `Port.type_id` + `TypeRegistry` |
| **Cardinality** | How many values (`ONE`, `ZERO_OR_ONE`, `ZERO_OR_MORE`) | Yes — `Port.cardinality` |
| **PresenceMode** | Whether the value *exists at all* | **No** — conflated into `Value::Skipped` at runtime |

### PresenceMode values

| Mode | Meaning | Example |
|------|---------|---------|
| `Required` | Must have a value; absence is a compile error | `fn` parameter, service endpoint |
| `Optional` | Domain-nullable by design; consumer must handle `None` | `description?` field, optional config |
| `Guardable` | May be absent due to control-flow skip; consumer must handle `Skip` | Output of a guarded node |
| `Guardable?` | May be domain-null OR control-flow skipped | Optional output of a guarded node |

### Transition operators (the only sanctioned narrowing paths)

| Operator | Transition | Semantics |
|----------|-----------|-----------|
| `require` | `Guardable<T> → Required<T>` | Error if skipped |
| `default(v)` | `Guardable<T> → Required<T>` | Substitute explicit fallback |
| `coalesce` (`??`) | `Optional<T> → Required<T>` | Substitute explicit fallback |
| `guard` | `Required<T> → Guardable<T>` | Node may not run (widening, always safe) |

**The key rule**: these are the *only* presence-mode narrowing operators. The executor/evaluator must not implicitly narrow. If a `Guardable` output feeds a `Required` input without an explicit narrowing node, it is a **compile-time wiring error**.

---

## Feature-by-Feature Enforcement Audit

Legend:
- **C** = enforced at compile time (parse/typecheck/lower)
- **R** = enforced at runtime (executor/evaluator) — deferred
- **S** = silent (no enforcement, silently proceeds with default/coercion)
- **X** = not applicable

### 1. Type System

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| Type name resolution | C | typecheck | `UndefinedType` error |
| Generic arity (`List<A,B>`) | C | typecheck | `ArityMismatch` error |
| Refinement range sanity (`@range(min > max)`) | C | typecheck | Only for literal min/max |
| Refinement pattern non-empty | C | typecheck | |
| Refinement brand non-empty | C | typecheck | |
| Refinement content encoding | C | typecheck | Known set: Text/UTF8/ASCII/Latin1/Binary/Unknown |
| `T` vs `T?` compatibility | **S** | — | **GAP**: No enforcement. `String` and `String?` are interchangeable. The typechecker normalizes via `normalize_type_id` which doesn't distinguish optionality. |
| Record field type compatibility | C (partial) | typecheck | Checked for return types and interface signatures. Not checked for arbitrary assignment or data flow. |
| Sum type exhaustiveness | **R** | eval | Match arms checked at runtime only. Missing arm → `"no match arm matched"` runtime error. |
| Type compatibility across edges | C (partial) | builder | `DagBuilder::add_edge` checks `is_compatible`. But the typechecker does NOT validate that the lowerer wires type-compatible edges — the builder is the last line of defense. |
| Semantic carrier compatibility | C | builder | `SemanticCarrierKind` checked on every edge. `UnknownSemantic` is fail-closed. |

### 2. Port Wiring

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| Required input has incoming edge | C (partial) | validate | `validate_required_inputs` checks `cardinality.min > 0` ports. But skips `res:*`, `tool:*`, `__*` prefixed ports. |
| Required input has a value at execution time | **R** | executor | No pre-dispatch check. Op calls `require_str` etc., which fails with `"missing or invalid"`. |
| Scalar port fan-in rejection | C | builder | `FanInOnScalar` error. Also re-checked at execution. |
| Fan-in cardinality overflow | C | builder | `FanInCardinalityOverflow` error. |
| SubDag interface consistency | C | validate | Bidirectional port name/type check. |
| Resource port wiring | C | validate | `validate_resource_wiring_recursive` detects unwired `res:*` ports. |
| Resource access mode conflicts | C | validate | `detect_conflicts` via Floyd-Warshall reachability. |
| **Presence mode tracking** | **X** | — | **GAP**: No presence mode on ports. Guard-skippable outputs are indistinguishable from required outputs in the type system. |

### 3. Guards and Control Flow

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| `[when cond]` guard type | **S** | — | **GAP**: Guard condition is lowered to `IfElse`; no check that cond is Bool. Pipeline `when` IS checked (`PipelineStageWhenTypeMismatch`), but node-level `when` is not. |
| Guard skip propagation | R | executor | All outputs → `Value::Skipped`. Downstream ops must handle. |
| Guard absence = skip | **S** | executor | **GAP**: If a guard's input value is missing from the input map, the node is skipped. This conflates "unwired" with "guard failed". |
| `if` without `else` | **S** | eval | Returns `Value::Unit` — no warning that the value may be absent. |
| `if/else` branch type compatibility | **S** | — | **GAP**: No check that both branches return the same type. |
| Match exhaustiveness | **R** | eval | Runtime-only. `"no match arm matched"` error. |
| Match arm type compatibility | **S** | — | **GAP**: Different arms can return different types. No enforcement. |

### 4. Function Calls

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| Call arity (too few / too many args) | C | typecheck | `CallArityMismatch` |
| Named arg validity | C | typecheck | `UnknownCallArgument`, `DuplicateCallArgument` |
| Call target resolution | C (partial) | typecheck | Strict mode only. Relaxed mode silently allows unresolved targets. |
| Return type compatibility | C (partial) | typecheck | Checked when return type is known. `ValueType::Unknown` (inferred) skips the check entirely. `is_lossy` bodies skip entirely. |
| `fn` purity (no effectful nodes) | C | lower | `PureFnContainsEffectfulNode` |
| Callable output wiring | C (partial) | lower | `MissingCallablePassthrough` only for `fn_body: None` nodes. Nodes with `fn_body: Some(...)` are exempt. |
| Service call arg wiring | **S** | lower | **GAP**: Unresolved ident args silently `continue` — unwired prepare inputs. Discovered at runtime as `"missing input"`. |

### 5. Service / Transport

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| Transport block required | C | lower | `MissingTransport` (fail-closed per RT4) |
| REST output field path format | C | lower | `InvalidTransportSpec` |
| Auth input field exists + is Secret | C | lower | `InvalidAuthInput` |
| Provider config field validation | C | lower | `InvalidProviderConfigField` / `UnknownProviderPrefix` |
| Service endpoint non-empty | **S** | lower | **GAP**: `service.config.endpoint.clone().unwrap_or_default()` — empty endpoint is silently accepted. |
| Transport phase metadata on emit | C | emit | `require_service_phase` — hard error if missing. |
| Credential chain defaults | **S** | runtime | **GAP**: `unwrap_or_default()` on credential fields. |

### 6. Evaluator Operations

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| Pipe methods on `Skipped` receiver | R | eval | All 15 methods → `"receiver is Skipped (unwired input)"`. This is correct. |
| Pipe methods on wrong type | R | eval | `"join requires a list"` etc. |
| Pipe methods with missing lambda | **S** | eval | **GAP**: `map`/`filter`/`filter_map`/`flat_map`/`sort_by` return list unchanged if lambda is missing. No error. |
| Division by zero | **S** | eval | **GAP**: Returns `0` silently. |
| Modulo by zero | **S** | eval | **GAP**: Returns `0` silently. |
| Field access on `Skipped` | **S** | eval | **GAP**: Returns `Value::Unit` silently. |
| Field access on `Unit` | **S** | eval | **GAP**: Returns `Value::Unit` silently. |
| Field access on JSON null | **S** | eval | Returns `Value::Json(Null)`. Propagates null. |
| Field access on JSON object, missing field | **S** | eval | **GAP**: Returns `Value::Json(Null)`. Map field access errors, JSON doesn't. Inconsistent. |
| String interpolation of `Skipped` | **S** | eval | **GAP**: Becomes `""`. Invisible in output. |
| String interpolation of `Unit` | **S** | eval | Becomes `""`. May be intentional. |
| String interpolation of `Map` | **S** | eval | Becomes `"map(N)"` — almost certainly a bug if hit. |
| `sum` on non-Int list elements | **S** | eval | **GAP**: Non-Int elements silently filtered. `["a","b"] |> sum()` = `0`. |
| `join` separator type | **S** | eval | **GAP**: Non-Str separator defaults to `","`. |
| `contains` with no needle | **S** | eval | **GAP**: Returns `false` instead of erroring. |
| `sort_by` key eval error | **S** | eval | **GAP**: Error discarded, key becomes `""`. |
| `for` on non-list | **S** | eval | **GAP**: Scalar wrapped in `vec![scalar]`, iterates once. |
| `first`/`last` on empty list | **S** | eval | Returns `Value::Unit`. Arguably correct for `first?`. |
| Variant construct (uppercase ident) | **S** | eval | **GAP**: Unbound uppercase ident → `Value::Str(name)` via heuristic. Not validated as a real variant. |

### 7. Executor Operations

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| Missing required input at execution | **R** | executor | No pre-dispatch check. Op's `require_*` helper errors. |
| `Value::Skipped` in fan-in | **S** | executor | **GAP**: Silently dropped from list aggregation. List becomes shorter. |
| `Value::Skipped` in `list_values()` | **S** | executor | **GAP**: Becomes `vec![]`. Loop runs 0 iterations. |
| `Value::Skipped` in `value_truthy()` | **S** | eval | **GAP**: Becomes `false`. |
| `Value::Skipped` in `value_to_string()` | **S** | eval | **GAP**: Becomes `""`. |
| `Value::Skipped == Value::Unit` | **S** | eval | **GAP**: `values_equal` returns `true`. These should be distinguishable. |
| DryRun node interception | C (partial) | executor | `validate_node_kinds_for_interception` pre-flight. But `DryRunStrictness::Strict` has no effect (Phase 1 stub). |
| Loop body with Skipped element list | **S** | executor | Zero iterations, no error. |
| Auto-mock transport in loop bodies | **S** | executor | Default `ShellResponse::ok("")` injected. May mask wiring bugs. |

### 8. Emission

| Feature | Enforcement | Phase | Notes |
|---------|------------|-------|-------|
| Reachable-only emission | C | emit | `ReachableDag<T>` type enforces structurally. |
| Service transport phase present | C | emit | `require_service_phase` — hard error. |
| Obligation category classification | C | lower+emit | `ObligationCategory` assigned at lower, consumed at emit. |
| Float literal in IR | **S** | lower | **GAP**: Floats demoted to strings (no `Float` variant in `LoweredLiteral`). No warning. |

---

## Summary of Gaps by Severity

### Tier 1: Type-system holes (presence/optionality not enforced)

These are the "meta missing" problems — absence is not tracked in the type algebra.

| Gap | Impact | Fix |
|-----|--------|-----|
| No `PresenceMode` on ports | Guard-skippable outputs silently feed required inputs | Add `presence: PresenceMode` to `Port`; validate transitions in `DagBuilder::add_edge` |
| `T` vs `T?` interchangeable | Optional values silently used where required values expected | Track optionality structurally in typechecker (not just string suffix) |
| `if` without `else` has no type | Returns `Unit` with no indication value may be absent | Require `if/else` in expression position, or infer `Optional` return |
| Match arm type compatibility not checked | Different arms can return different types | Unify arm types in typechecker |
| Match exhaustiveness not checked | Missing arms discovered at runtime only | Static exhaustiveness check for known sum types |
| Guard absence = skip (not error) | Unwired guard input indistinguishable from failed guard | Unwired guard input → compile error, not runtime skip |

### Tier 2: Silent coercions (bad state treated as good state)

| Gap | Impact | Fix |
|-----|--------|-----|
| `Value::Skipped` → `""` in string interp | Missing data invisible in output | Error on Skipped in `value_to_string`; require explicit `default` |
| `Value::Skipped` → `vec![]` in `list_values` | Loops silently run 0 iterations | Error on Skipped; require explicit `default([])` |
| `Value::Skipped` → `false` in `value_truthy` | Missing data treated as falsy | Error on Skipped in boolean context |
| `Value::Skipped` → dropped in fan-in | Lists silently shorter | Error on Skipped in fan-in (or require explicit filter) |
| `Skipped == Unit` | Can't distinguish "null" from "skipped" | Remove this equivalence |
| Field access on Skipped → Unit | Missing data propagates as null | Error, like `GetFieldOp` already does |
| Div/mod by zero → `0` | Silent wrong answer | Error |
| `sum` on non-Int → filtered | `["a"] |> sum()` = `0` | Error on non-Int elements |

### Tier 3: Deferred validation (catchable at compile time but deferred to runtime)

| Gap | Impact | Fix |
|-----|--------|-----|
| Service call arg wiring gaps | Unresolved idents silently dropped | Lower-time error for unresolved args |
| Required inputs not pre-checked before execution | Missing inputs discovered by op, not executor | Pre-dispatch required input check |
| Callable output wiring exemption for `fn_body` nodes | Unwired outputs discovered at runtime | Validate all callable outputs, including `fn_body` nodes |
| Service endpoint `unwrap_or_default()` | Empty endpoint accepted | Lower-time validation: endpoint must be non-empty |
| `[when]` guard condition type | No Bool check for node guards | Typecheck node guard conditions like pipeline guards |
| Variant ident heuristic | Uppercase unbound → `Value::Str(name)` | Validate against known variant names at typecheck/lower time |
| Pipe methods missing lambda → passthrough | `list |> map()` returns list unchanged | Error on missing lambda |
| `sort_by` key error → `""` | Sort key errors silently discarded | Propagate key eval error |
| `contains` no needle → `false` | Missing arg returns false | Error on missing required arg |
| `join` non-string separator → `","` | Wrong type silently coerced | Error on non-Str separator |
| `for` on scalar → `[scalar]` | Non-list silently wrapped | Error or warn; require explicit `[x]` |
| Float → String in lowered IR | Precision loss, no warning | Add `LoweredLiteral::Float` or emit warning |

### Tier 4: Inconsistencies (same concept, different enforcement)

| Inconsistency | Details |
|---------------|---------|
| Map field access errors, JSON field access doesn't | `Map.missing` → error; `Json.missing` → `Null` |
| `GetFieldOp` (resolve) rejects Skipped, `field_access` (eval) doesn't | Two code paths, different behavior |
| Pipeline `when` type-checked, node `when` not | Same language feature, different enforcement |
| Strict mode vs relaxed mode | Relaxed is the default; many errors only surface in strict mode |
| `fn_body: None` output wiring checked, `fn_body: Some` not | Same callable shape, different validation |

---

## Remediation Roadmap

### Phase 1: Presence mode (Tier 1 + Tier 2 core)

1. Add `presence: PresenceMode` to `Port` — `Required | Optional | Guardable | GuardableOptional`.
2. `DagBuilder::add_edge` validates presence transitions: `Guardable → Required` requires an explicit narrowing node.
3. Guard skip now produces `Value::Skipped` only on `Guardable` output ports. Required outputs on skipped nodes → hard error (graph was wired wrong).
4. Eliminate silent Skipped coercions: `value_to_string`, `list_values`, `value_truthy`, `collect_fan_in`, `values_equal(Skipped, Unit)` all become hard errors.
5. Add `default(value, fallback)` and `require(value)` as explicit DAG-level narrowing operators.

### Phase 2: Typechecker hardening (Tier 1 remaining + Tier 3)

1. Structural optionality tracking (not string suffix) — `TypeExpr::Optional` affects compatibility checking.
2. `if/else` branch type unification (both arms must return compatible types).
3. Match exhaustiveness checking for known sum types.
4. Node-level `[when]` guard condition type-checked as Bool.
5. Variant ident validation against known variant names.
6. Lower-time error for unresolved service call arguments (no silent `continue`).

### Phase 3: Evaluator strictness (Tier 2 remaining + Tier 3 remaining)

1. Division/modulo by zero → error.
2. `sum` on non-Int elements → error.
3. Pipe methods require lambda when documented as required.
4. `sort_by` key error propagation.
5. `contains` requires needle argument.
6. `join` requires Str separator.
7. `for` requires List receiver (or explicit wrap).
8. Float literal support in `LoweredLiteral`.

### Phase 4: Consistency (Tier 4)

1. Unify Map and JSON field access behavior (both strict, or both null-propagating with explicit opt-in).
2. Unify `GetFieldOp` (resolve) and `field_access` (eval) Skipped behavior.
3. Make strict mode the default; relaxed mode requires explicit opt-in.
4. Validate `fn_body: Some` callable output wiring.
5. Pre-dispatch required input validation in executor.

---

## Relationship to Existing Design Docs

- **`domain-hard-error-no-fallback-plan.md`**: Covers extern symbol linking — "missing symbol = hard error." This doc covers the complementary axis: "missing *value* = hard error unless explicitly narrowed."
- **`dsl-design.md`**: Language spec. Phase 2 changes here (exhaustiveness, branch typing) would require spec updates.
- **`SPEC.md`**: Formal IR spec. Phase 1 changes (PresenceMode on Port) would require spec updates.
- **Refactor-Pressure Checklist (CLAUDE.md)**: "Translation layers are total or error" and "No stubs that look like features" are already stated invariants. This doc makes them mechanically enforceable.

---

## Appendix: Skipped Coercion Sites (Exhaustive)

Every location where `Value::Skipped` is silently treated as a valid value:

| File | Function | Line | Coercion |
|------|----------|------|----------|
| `daglang-lower/src/eval.rs` | `value_to_string` | 1148 | `Skipped → ""` |
| `daglang-lower/src/eval.rs` | `value_truthy` | 1184 | `Skipped → false` |
| `daglang-lower/src/eval.rs` | `values_equal` | 462-463 | `Skipped == Skipped`, `Skipped == Unit` |
| `daglang-lower/src/eval.rs` | `field_access` | 404 | `Skipped.field → Unit` |
| `daglang-lower/src/eval.rs` | `sort_key` | 1136 | `Skipped → "skipped"` |
| `core/exec/src/execute/mod.rs` | `collect_fan_in` | 1278 | `Skipped → None` (dropped) |
| `core/exec/src/pattern_op.rs` | `list_values` | 162-168 | `Skipped → vec![]` |
| `core/exec/src/helpers.rs` | `optional_int` | 174 | `Skipped → Ok(None)` |
| `core/resolve/src/resolve.rs` | `DeclaredOutputCallableOp` | 223 | Optional port `Skipped → Skipped` (passthrough, not coercion) |
| `core/resolve/src/resolve.rs` | `StringInterpolateOp` | 540-549 | All-Skipped inputs → `Skipped` output (propagation, not coercion) |
