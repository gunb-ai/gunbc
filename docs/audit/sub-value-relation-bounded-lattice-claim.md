# SubValueRelation BoundedLattice Claim

**Date:** 2026-05-02  
**Lane:** R3 Substrate + Verification  
**Scope:** audit / retirement spec plus retirement receipt.

## Summary

At audit time, `src/v3/std/induction.dag` said:

- `SubValueRelation inhabits BoundedLattice`
- `meet = meet_sub_value`
- `join = join_sub_value`
- `top = PreservedValue (meet identity)`
- `bottom = SubValueUnknown`

That claim does **not** match the implementation. `meet_sub_value(PreservedValue, b)`
returns `PreservedValue` for every structural `b`, so `PreservedValue` is not a
meet identity for the structural variants. The join side is fine: `join_sub_value`
does satisfy `join(SubValueUnknown, a) == a` for all five variants.

The least-risk retirement path is to **drop the BoundedLattice claim** and weaken
the text to a meet-oriented algebra claim. No `src/v3/` consumer currently relies
on `PreservedValue` as a real top element, so this is a documentation fix unless
Substrate wants to add a genuine top variant later.

## 1. Meet / join behavior matrix

Notation:

- `U` = `SubValueUnknown`
- `P` = `PreservedValue`
- `S*` = `StrictSubValue` with matching field/factor payload on both sides
- `I*` = `IteratedSubValue` with matching field payload on both sides
- `A*` = `ArithmeticDescent` with matching param/factor payload on both sides
- Mismatched structural payloads on the same variant collapse to `U`

### `meet_sub_value(a, b)`

| a \ b | U | P | S* | I* | A* |
|---|---|---|---|---|---|
| U | U | U | U | U | U |
| P | U | P | P | P | P |
| S* | U | P | S* | U | U |
| I* | U | P | U | I* | U |
| A* | U | P | U | U | A* |

### `join_sub_value(a, b)`

| a \ b | U | P | S* | I* | A* |
|---|---|---|---|---|---|
| U | U | P | S* | I* | A* |
| P | P | P | S* | I* | A* |
| S* | S* | S* | S* | U | U |
| I* | I* | I* | U | I* | U |
| A* | A* | A* | U | U | A* |

## 2. Law checks

### Meet identity check

The declared top law would require `meet(P, b) == b` for every `b`.

| b | `meet(P, b)` | Expected | Mismatch |
|---|---|---|---|
| U | U | U | No |
| P | P | P | No |
| S* | P | S* | Yes |
| I* | P | I* | Yes |
| A* | P | A* | Yes |

The failing cases are exactly the three structural variants:
`StrictSubValue`, `IteratedSubValue`, and `ArithmeticDescent`.

### Join bottom check

The declared bottom law would require `join(U, a) == a` for every `a`.

| a | `join(U, a)` | Expected | Mismatch |
|---|---|---|---|
| U | U | U | No |
| P | P | P | No |
| S* | S* | S* | No |
| I* | I* | I* | No |
| A* | A* | A* | No |

The symmetric check `join(a, U) == a` also holds from the implementation.

## 3. Fix-path spec

### Path A: add a real top variant

If Substrate wants `SubValueRelation` to remain a bounded lattice, it needs a
genuine top element, not `PreservedValue`. A plausible shape would be:

- add `UnconstrainedSubValue` or `SubValueAny`
- make `meet(UnconstrainedSubValue, x) == x`
- make `join(UnconstrainedSubValue, x) == UnconstrainedSubValue`
- leave `PreservedValue` as the "same argument / no descent" case only

That path changes the algebra surface and the meaning of the existing
`PreservedValue` carrier.

### Path B: weaken the claim

Drop the `BoundedLattice` claim and state only the meet/join helpers that are
actually implemented. The conservative interpretation is `MeetSemilattice` for
the failure-proof side of the analysis, with `join_sub_value` left as an
auxiliary helper rather than a bounded-lattice witness.

### Recommendation

Choose **Path B**.

Reasoning:

- The current code already behaves like a fail-closed merge helper plus an
  optimistic helper, not like a bounded lattice with a true top.
- No `src/v3/` consumer currently depends on `PreservedValue` being the top.
- Adding a top variant would force a semantic split between "same argument"
  and "top element" that the current carrier does not have.

## 4. Consumer audit

Search scope: `src/v3/`.

Findings:

- The only direct claim is the doc comment in `src/v3/std/induction.dag`.
- `src/v3/compiler/src/dag.rs` mirrors the `SubValueRelation` variants and
  produces `PreservedValue` for same-argument provenance, but it does not treat
  `PreservedValue` as a lattice top.
- `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` checks
  mirror shape alignment for `SubValueRelation`; it does not consume a
  bounded-lattice witness or top element.

Conclusion: no `src/v3/` consumer found that would break if the bounded-lattice
claim were removed or weakened. The fix is therefore a claim correction, not a
consumer rewrite.

## 5. Debt-route receipt

Existing ledger row:

| Row | Introduced | Owner / lane | Status | Retirement shape |
|---|---|---|---|---|
| `SubValueRelation bounded-lattice law violation` | 2026-05-01 | R3 Substrate + Verification | Open | Fix ordering/top semantics or stop claiming `BoundedLattice`. |

Ledger file: `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` line 91.

Routing:

- Substrate Mgr inbox: `#1130`
- Debt-Paydown Mgr inbox: `#1526`

Retirement receipt:

- `src/v3/std/induction.dag` now documents `meet_sub_value` /
  `join_sub_value` as merge helpers, not a bounded-lattice witness.
- `dsl/std/induction.dag` carries the same correction so the v2/v3 mirror text
  stays aligned.
- `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` marks the row `Retired`.

## 6. PR receipt text

Use this disposition in the PR body:

- **Debt retired**: `SubValueRelation bounded-lattice law violation`; the
  false `BoundedLattice` claim is removed from both v3 and dsl induction
  authorities, and `SubValueRelation` is documented as fail-closed merge
  helpers instead.
