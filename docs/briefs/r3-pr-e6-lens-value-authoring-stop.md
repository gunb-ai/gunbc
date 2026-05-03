# R3 PR-E6 Lens Value Authoring STOP Receipt

**Status:** STOP receipt for E6-G1 substrate/API unblocker. This slice does
not implement `fold_lens`, add evaluator callable/field execution, add a Rust
lens registry, or author a placeholder `Lens<C>` value.

**Inputs:** `docs/briefs/r3-pr-e6-lens-fold-readiness-audit.md`,
`docs/briefs/r3-pr-e6-post-blocker-gate-packet.md`, `src/v3/std/lens.dag`,
`src/v3/std/dimensions.dag`, `src/v3/lenses/cost.dag`, and
`src/v3/lenses/complexity.dag`.

## Decision

No honest `Lens<C>` value can be authored or referenced on current `main`
without introducing a parallel authority.

The smallest candidate values were `Lens<Int>` and
`Lens<SymbolicCost>`. Both require the same unsupported shape:

```text
data <name>: Lens<C> = {
  name: "...",
  read: <fn(Dag, Behavior) -> Witness<C>>,
  sequential: { op: <fn(C, C) -> C>, identity: <C> },
  branch: <fn(C, C) -> C>,
  iterate: <fn(C, LoopBound) -> C>,
  validate: <fn(Dag, C) -> OptionalDiagnostic>
}
```

Authoring only a typed name, empty record, string key, or Rust-side registry
would let E6-G1 compile against something weaker than the declared
`Lens<C>` boundary. That would recreate the exact parallel lens authority the
readiness audit rejects.

## Blocker Classification

| Case | Current status | Why it blocks an honest `Lens<C>` value |
| --- | --- | --- |
| Class-5 data-body lowering | Live for the non-generic pieces this shape needs. | Current tests already cover structural record data, nested structural records, and non-generic function-valued field references. This is not the remaining blocker by itself. |
| Function-valued field references | Live for non-generic Conj fields, blocked through instantiated generic Conj fields. | `MiniLens<Int>` / `Monoid<Int>`-shaped data still fails when field checking must substitute `C -> Int` through `Lens<C>` and `Monoid<C>`. E6-G1 needs that substitution-aware field lowering before `read`, `branch`, `iterate`, `validate`, and `sequential.op` can be trusted as declared function fields. |
| `Monoid<C>` structural witness value authoring | Blocked by the same instantiated generic Conj substitution gap. | Non-generic nested witness records lower. The missing fact is that `Lens<Int>.sequential: Monoid<Int>` must resolve `Monoid<C>` fields under substitution and accept `op: fn(Int, Int) -> Int` plus `identity: Int` without a host default. |
| `Witness<C>` / `OptionalDiagnostic` constructor expression support | Live for the representative constructors. | Current tests cover `Witness<Int>::Inhabits` and `OptionalDiagnostic::NoDiagnostic` constructor-returning functions. This is not the remaining authoring blocker, though E6-G1 still must execute and lift those values later. |
| Explicit typed lens-instance handle instead of full `data Lens<C>` | Not chosen in this slice. | A handle may become the right narrow API only if generic Conj substitution remains delayed, but it must be a substrate carrier that points at a real declared lens authority. This dispatch did not have enough evaluator consumer shape to specify that handle without inventing a registry. |

## Next Dispatch

Recommended next substrate dispatch:

`lens_value_generic_conj_field_substitution_lands`

Scope:

- make structural data-body checking substitute instantiated generic Conj
  fields, specifically `Lens<C> -> Lens<Int>` and
  `Monoid<C> -> Monoid<Int>`, before validating field values;
- prove one minimal `Lens<Int>` fixture lowers to a `ValueBody::Structural`
  with `name`, `read`, `sequential`, `branch`, `iterate`, and `validate`
  present as typed fields, with `sequential.op` and `sequential.identity`
  read from the nested `Monoid<Int>` value;
- keep `Witness<C>`, `OptionalDiagnostic`, and `DimensionReport<C>` unchanged.

Only after that substrate fact lands should Evaluator E6-G0/G1 consume the
value through field projection and callable execution. E6-G1 still owns report
lifting; this STOP receipt does not implement it.

## Non-Changes

This receipt introduces no Rust lens registry, no per-lens callbacks, no new
report or witness carriers, no `test_runner.rs` predicate, and no value that
fabricates `DimensionReport<C>`.
