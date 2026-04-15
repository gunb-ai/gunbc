# Inference as Data — I0 Result

This note records the outcome of **Experiment I0** from
`docs/inference-as-data-experiments.md` on the implementation
branch that landed Prereq 0 and the first lambda pass. The
canonical experiment doc lives on the docs branch; this note is the
implementation-side result so the finding is captured in-tree.

## Outcome

**Pass.** The representative inference rule checked for I0 is
decidability-compatible under the current `.dag` bounded-iteration
invariant, and a second bounded-walk spot-check also passes.

## Rule 1 — Literal Type Filling

**Rust rule checked:** [`src/v3/compiler/src/infer.rs`](../src/v3/compiler/src/infer.rs)
`decide()` `Behavior::Value` arm.

Current Rust shape:

- `LiteralBits::Int(_)` resolves to `Int`
- `LiteralBits::Bool(_)` resolves to `Bool`
- `LiteralBits::String(_)` resolves to `String`

Pseudo-`.dag` transliteration:

```text
fn infer_value(v: ValueNode, d: Dag) -> Dag =
  match v.data {
    IntLit(_)    => d.with_port_type(v.output, Int)
    BoolLit(_)   => d.with_port_type(v.output, Bool)
    StringLit(_) => d.with_port_type(v.output, String)
  }
```

Decidability check:

- `match v.data { ... }`: bounded exhaustive match over a finite
  payload variant set. No iteration.
- `d.with_port_type(...)`: a single Dag-to-Dag update. No mutation is
  required at the rule boundary; the rule is still expressible as
  pure input-to-output transformation.
- Return value: one new Dag value. No side channel or external state.
- Whole rule: no recursion, no search, no unbounded iteration.

**Result:** pass.

## Rule 2 — Bounded Sum Walk Spot-Check

As a stronger follow-up than the trivial literal rule, I also
checked the sum-resolution helper used by branch typing:
`walk_to_disj_decl()` in
[`src/v3/compiler/src/infer.rs`](../src/v3/compiler/src/infer.rs).

Why it still passes:

- The walk is explicitly bounded by `WALK_DEPTH_LIMIT`.
- Each step follows one structural edge
  (`Instantiation` or `ResolvedIdentifier`) or stops.
- The caller-side branch checks iterate over a finite arm list and a
  finite variant list.

This means the first non-trivial structural walk in the inference
pass also stays within the decidability invariant: bounded forward
descent over finite substrate structure.

## Recommendation

`I0` does **not** surface a decidability blocker for the
inference-as-data direction. The next experiments can proceed. If a
later rule in the sequence fails, the failure is more likely to be a
write-surface or representation question than a global
decidability-invariant violation.
