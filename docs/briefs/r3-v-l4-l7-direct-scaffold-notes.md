# R3 T-V-L4-L7-Direct Scaffold Notes

**Status:** PROPOSAL — scaffold-only design notes for standby preparation. No implementation dispatch; no substrate changes. Parent authority is [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md), which remains gated on R2-Evaluator PR-A.3 carriers and PR-B body evaluator landing.

## Purpose

These notes narrow the first implementable shapes for Lane 1:

- **L4:** `l4_emit_eval_match` — emitted target output equals `.dag` evaluator output on the certification corpus.
- **L7:** `l7_algebraic_laws_witnessed` — every algebra declared in `dsl/std/algebra.dag` eventually has runtime-constructed witnesses for each applicable law.

They do not close either gate. They record what can be prepared before the evaluator can run bodies, and what must wait for post-PR-B runner support.

## Slice 1 — L4 Minimal Corpus Shape

The first L4 receipt should be minimal but not degenerate: it should evaluate a real expression, not only `Compiles`, while avoiding target runtime features outside the first evaluator path.

Proposed seed program shape:

```dag
fn add_then_branch(x: Int, y: Int) -> Int =
  match true {
    True => x + y
    False => x
  }

let l4_out: Int = add_then_branch(1, 2)
```

Why this shape:

- It exercises function call, `Int` arithmetic, a constant branch, and a named output bind.
- It avoids list/fold, effects, IO, external calls, and target-library behavior.
- It can later expand to list/fold once PR-B has proven body evaluation over collection programs.

The fixture name from the parent brief remains the preferred path: `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`, suite `r3_verification_l4_l7_direct_suite`.

## DifferentialEquals Runner Path

`TestPredicate::DifferentialEquals` already has the right substrate shape: `subject_ref`, `oracle_ref`, and `input_ref`. The landed PR-D slice proves the runner can dispatch this predicate, but the current implementation is cost-lineage-specific:

- `test_runner.rs` accepts only the `(v3_program_cost, v2_oracle_cost)` pairing.
- It resolves `input_ref` through `ProgramOutputBind`.
- It compiles `TestClaim.source`, finds the named bind, then compares host forward-fold cost against `lens_cost::cost_of`.

L4 target-vs-eval parity is therefore a **different runner path**, not a direct reuse of the current cost evaluator. It can reuse the `DifferentialEquals` constructor only if the runner grows new lineage producers such as:

- `rust_emit_output` — compile/emit the claim source to Rust, run the emitted artifact hermetically, capture the observable output for the named bind.
- `dag_eval_output` — execute the same claim source through the R2 body evaluator and capture the same observable value.

`ProgramOutputBind` still fits: `input_ref` names the output bind (`l4_out`) that both producers observe. The new work is producer dispatch and value normalization, not a new `TestPredicate` variant.

## Standby-Time Expressibility

Slice 1 is **not executable at standby time**. Before PR-B lands, there is no evaluator-side computational result for `.dag` bodies, so `dag_eval_output` would be fabricated if implemented now.

The design can be frozen now as:

- corpus source text and output-bind convention;
- lineage names / producer responsibilities;
- expected failure taxonomy: emit failure, target execution failure, evaluator failure, and value mismatch;
- no substrate extension.

The actual `DifferentialEquals` row should wait until PR-B exposes body-evaluator output and PR-A.3 provides deterministic strategy/memoization carriers.

## Slice 3 — L7 First Witness Surface

Current substrate and runner state:

- `AlgebraicLawKind` has `Associativity`, `Commutativity`, and `Identity`.
- `TestPredicate::AlgebraicLaw` accepts `law` plus `lens_ref`.
- The runner wires `Associativity`, `Commutativity`, and `Identity` through bounded operational witness tables / identity-candidate search.
- `dsl/std/algebra.dag` also names distributivity for semiring/ring/lattice-like structures, but there is no `Distributivity` variant yet.

The L7 matrix fixture uses the currently executable law surface and honest additive/multiplicative `Int` witnesses. It remains intentionally separate from non-enum laws such as distributivity, which require substrate §P1 expansion rather than fixture-local encodings.

The normal `r3_verification_l7_algebraic_law_matrix_has_current_runner_receipts` ratchet is the gate #10 receipt for the current `AlgebraicLawKind` surface.

## Algebra Coverage Audit

`dsl/std/algebra.dag` declares the following law-bearing structures:

| Structure | Applicable laws named by current model |
|---|---|
| `Semigroup<T>` | associativity |
| `Monoid<T>` | associativity, identity |
| `CommutativeMonoid<T>` | associativity, identity, commutativity |
| `Group<T>` | associativity, identity, inverse law |
| `AbelianGroup<T>` | associativity, identity, inverse law, commutativity |
| `Semiring<T>` | additive commutative monoid, multiplicative monoid, distribution, zero annihilation |
| `Ring<T>` / `OrderedRing<T>` | additive abelian group, multiplicative monoid, distribution; order laws for `OrderedRing` are outside current `AlgebraicLawKind` |
| `Field<T>` | ring-like laws plus multiplicative inverse where non-zero; exact-field law is not valid for `Float`'s approximate profile |
| `Lattice<T>` / `BoundedLattice<T>` | associativity, commutativity, absorption, identity via top/bottom for bounded forms |
| `BooleanAlgebra<T>` | bounded lattice laws, complement, distributivity |
| `FreeMonoid<T>` | concat associativity and empty identity |

The current `AlgebraicLawKind` enum is narrower than the model: it lacks distributivity, absorption, inverse, complement, annihilation, and order-law tags. Do **not** add variants from this lane; route that through `INVARIANTS.md` §P1 / Substrate Manager.

## Lens-Framework Cross-Check

`docs/design-lens-framework.md` I4/I9 defines worked-example and aggregate-validation TestClaims. They are useful seed references for witness construction, but do not rename them as L7 closure: L7 requires algebra-law witnesses over every applicable algebra in `dsl/std/algebra.dag`.

## Coverage Progression

1. **Slice 1:** one Rust L4 `DifferentialEquals` row over the minimal `add_then_branch` program, frozen until PR-B makes `dag_eval_output` real.
2. **Slice 2:** add Python and Go L4 rows as Shape A grounding closes; each row compares target output to `.dag` eval, not target-to-target.
3. **Slice 3:** add one `AlgebraicLaw(Associativity, ...)` seed using the current runner-wired path.
4. **Slice 4:** extend L7 to `Commutativity` and `Identity` only after runner support exists.
5. **Slice 5+:** enumerate the algebra coverage matrix from `dsl/std/algebra.dag`; for laws not represented by `AlgebraicLawKind`, escalate substrate shape rather than inventing fixture-local encodings.

Partial slice coverage remains lane evidence only. Lane 1 closes only when both `l4_emit_eval_match` and `l7_algebraic_laws_witnessed` satisfy the parent brief and `r3-structure.md` authorities.

## Non-Claims

- No new `TestPredicate` variants are proposed here.
- No L5 cross-target behavior is claimed or implied.
- No L5-absorbs-L4 dissolution path exists.
- No single-law L7 seed closes the lane.
