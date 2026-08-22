# `sorted_map_keys` exists everywhere except the interpreter (2026-08-22)

**Blocks the typed carrier-realization census.** Not this lane's to repair; recorded with a sibling
control so the gap is not mistaken for a design choice.

## OBSERVED ON / CLAIM ABOUT

- **OBSERVED ON:** `gunbc run --entry src/v1/tests/claim/carrier_realization_census.dag --function
  census_run_typed`, on a three-file subject, remote runner, 2026-08-22.
- **CLAIM ABOUT:** the v1 **interpreter's** builtin dispatch, relative to the same builtin's
  declaration, inference typing, and Rust emission.

## The failure

```
✓ compile.normalize done in 904ms
error: evaluating census_run_typed in src/v1/tests/claim/carrier_realization_census.dag
  cause: NoSuchFunction { name: "sorted_map_keys" }
```

It refuses **after normalize and before reconcile** — which is exactly where it should, because
`v1.compiler.infer` calls `sorted_map_keys` twice while building its index (`entry_keys`,
`bare_keys`). Any interpreted path that reaches reconcile hits this.

## The sibling control: three siblings, one missing arm

`sorted_map_keys` is present at every layer *except* execution-by-interpretation:

| | `map_keys` | `map_values` | `sorted_map_keys` |
|---|---|---|---|
| declared in `std.primitives` builtins | yes | yes | **yes** |
| typed in `v1.compiler.infer_method` | yes | yes | **yes** |
| Rust emit bridge (`extdeps.languages.rust.emit`) | yes | yes | **yes** |
| Rust runtime (`v1.runtime_rust`) | yes | yes | **yes** |
| **interpreter arms** (`v1_interpreter.rs`) | **2** | **2** | **0** |

Measured: `grep -c "sorted_map_keys" src/v1/stage0/src/v1_interpreter.rs` → `0`, against two
`arm "method_call.map_keys"` / `"free_call.map_keys"` arms and two for `map_values`.

The two siblings are the control that makes this a **gap rather than a decision**: the shape exists,
is implementable, and is implemented twice over for the neighbouring operations. `sorted_map_keys` is
`map_keys` with a `sort()` — the Rust runtime defines it that way literally.

## Why this is a refusal and not a silent wrong answer

The interpreter answered `NoSuchFunction` with the name — typed, located, loud. That is the floor
behaving correctly, and it is worth stating because this lane has spent the night on the opposite
class. Nothing was fabricated; a capability that does not exist said so.

## What it blocks

The typed carrier-realization census walks `compile_to_resolved` → `ResolvedGraph.modules` →
`TypedModule.items` + `type_env`. Reaching the typed graph **requires reconcile**, and reconcile
calls `sorted_map_keys`. So the typed re-grain **cannot be measured through `gunbc run` at all**
until one of:

1. the interpreter gains a `sorted_map_keys` arm (small — the two siblings are the template, and the
   Rust runtime's one-line definition is the semantics), or
2. the census runs from emitted Rust rather than interpretation — which the seed-closure and
   host-effect constraints recorded in the neighbouring probe currently prevent.

**Not routed around.** No substitute call, no re-spelling, no local re-implementation of the builtin
inside the census — any of those would hide a language-layer deficit inside a measurement instrument,
which is the failure mode this lane exists to measure.
