### Root Cause B: Closed Sets Dispatched as Strings

**Invariants violated:** No case enumeration for open sets, No parallel
implementations.

**The problem:** Several finite, known-at-compile-time sets are encoded as
strings and dispatched via `if x == "..."` ladders across multiple files.
Adding a value to any set requires editing every dispatch site — there is
no compiler-enforced exhaustiveness.

**Design decision required (methods only):** Are method/builtin intrinsics a
closed compiler-known set (→ enum) or structural DSL-defined facts the
compiler discovers? The language thesis says "smart facts + dumb compiler,"
so methods should eventually be data declarations in `.dag`. Pragmatically,
an `IntrinsicId` enum is the right intermediate step — it centralizes the
set and gives exhaustiveness checking. The enum definition becomes the single
authority; reconcile tags each call with an `IntrinsicId`; emit matches on
the enum instead of strings.

Transport kind, item kind, and type structure are mechanical enum conversions
with no design ambiguity.

| # | Closed set | Values | Dispatch sites | Files affected |
|---|-----------|--------|---------------|----------------|
| B-1 | Transport kind | rest, shell, file, local | 21 | 04_reconcile, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-2 | Item kind (`classify_typed_item`) | type_def, type_alias, function, data_def, service_def, resource_def, extern_func, unhandled | 8 dispatch chains | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-3 | Type structure (`classify_type_structure`) | leaf, conj, disj | 3 dispatch chains | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-4 | Method/builtin intrinsics | ~35 methods + ~20 builtins | ~60 string branches | 04_reconcile (inference), 05_emit (classification), 05_emit_rust (lowering) |
| B-5 | Operation modifiers | idempotent, readonly, hermetic | 1 filter expression | 05_emit_rust:2836 |
| B-6 | Config property names | base_url, auth_scheme, auth_header, auth_token | `config_names` list + constructors + accessors | 00_core.dag (triple representation) |

**Previously tracked as:** TD-2, TD-3, F7 (partially — the emit-side ladder is Root Cause A)

---

