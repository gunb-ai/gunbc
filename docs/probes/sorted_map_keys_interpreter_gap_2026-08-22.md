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

## The repair's real deliverable is the ORDER, not the arm

Raised by `smart-ram-730` and recorded here because it is the half that can go silently wrong, and
because a reader who finds this doc will otherwise conclude the fix is two lines.

`v1.runtime_rust` defines `sorted_map_keys` as `map_keys` followed by `keys.sort()` over `K: Ord` — so
**emitted Rust sorts by Rust's `Ord` for the key type.** An interpreter arm would sort `Value`s, and
`cmp_values` already exists and is what `method_call.sort_by` uses.

If those two orderings disagree for any key type in real use — strings, ints, interned ids — then
interpretation and emission compute **different key orders from the same source, silently**, and every
downstream fold over `sorted_map_keys` diverges between the two realizations. `v1.compiler.infer`'s
`entry_keys` and `bare_keys` folds are exactly such consumers.

That would be a **fresh cross-realization divergence introduced by the fix** — the same class this
whole lane exists to measure, created while closing a gap in it.

So completion is not *"the arm exists and the census runs."* It is **"the arm exists AND the order
provably matches emitted Rust for the key types in use, shown by execution"** — not by reading the two
sort implementations and judging them equivalent.

## The contrast worth keeping: one boundary fabricates, the other refuses

This repository currently has both failure modes live, and the pair is more informative than either
alone:

| boundary | behaviour on a capability it lacks |
|---|---|
| emission (the `Filesystem` finding, neighbouring probe) | emits plausible-looking output; fails at `rustc`, or compiles and does the wrong thing |
| interpretation (this finding) | names the missing function and **stops** |

The difference is not difficulty. It is that one of them was built to stop. That is why this blocker
cost an hour instead of a week spent trusting a wrong census answer — and it is the concrete argument
for the fail-closed discipline, made by the two boundaries disagreeing about the same repository.
