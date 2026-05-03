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
| Class-5 data-body lowering | Partially live for scalar, top-level record/list/map structural data. | `Lens<C>` needs nested structural values and function-valued fields. The existing `ValueBody` surface is not enough to express a full lens instance as executable data. |
| Function-valued field references | Blocked for `Lens<C>` fields. | `read`, `branch`, `iterate`, `validate`, and `Monoid<C>.op` must be references to executable functions of arrow type. Current structural data references are declaration-reference shaped, not first-class function values consumable by E6-G1. |
| `Monoid<C>` structural witness value authoring | Blocked for the nested `sequential` field. | E6-G1 must read `sequential.identity` and `sequential.op` from the declared value. A host default such as integer `0` or symbolic-cost `ConstantCost(0)` would fabricate the monoid witness. |
| `Witness<C>` / `OptionalDiagnostic` constructor expression support | Blocked for representative `read` and `validate` functions. | A useful lens read must return `Inhabits(c)` or `Violates { reason, at }`; validation must return `NoDiagnostic` or `SomeDiagnostic { value }`. Current lens files still document this as deferred instead of authoring constructor-returning function bodies. |
| Explicit typed lens-instance handle instead of full `data Lens<C>` | Not chosen in this slice. | A handle may become the right narrow API if class-5 value bodies remain delayed, but it must be a substrate carrier that points at a real declared lens authority. This dispatch did not have enough evaluator consumer shape to specify that handle without inventing a registry. |

## Next Dispatch

Recommended next substrate dispatch:

`lens_value_structural_data_fields_lands`

Scope:

- make structural data bodies able to carry function-valued declaration
  references for fields whose declared type is an arrow;
- make nested structural data sufficient for `Monoid<C>` witness values;
- prove one minimal `Lens<Int>` fixture lowers to a `ValueBody::Structural`
  with `name`, `read`, `sequential`, `branch`, `iterate`, and `validate`
  present as typed fields;
- keep `Witness<C>`, `OptionalDiagnostic`, and `DimensionReport<C>` unchanged.

Only after that substrate fact lands should Evaluator E6-G0/G1 consume the
value through field projection and callable execution. E6-G1 still owns report
lifting; this STOP receipt does not implement it.

## Non-Changes

This receipt introduces no Rust lens registry, no per-lens callbacks, no new
report or witness carriers, no `test_runner.rs` predicate, and no value that
fabricates `DimensionReport<C>`.

