# R3 verification — L7 algebra coverage matrix (scaffold)

Authoritative theory rows live in `dsl/std/algebra.dag` (emergence table + hierarchy).
This matrix ties that vocabulary to today’s **`AlgebraicLaw(law, lens_ref)`** runner surface in
`src/v3/compiler/src/test_runner.rs` and `src/v3/std/verification.dag` (`AlgebraicLawKind`).

**INVARIANTS §P1:** Adding substrate predicates or extending `AlgebraicLawKind` (for example to
encode distributivity over two linked operators) is a Director ratification gate — fixtures must
not invent new enum variants.

| Algebraic structure (theory) | Typical laws (from `algebra.dag`) | `AlgebraicLawKind` today | Runner (`eval_algebraic_law`) |
|-----------------------------|-----------------------------------|---------------------------|--------------------------------|
| Magma | closure only | — | not applicable (`AlgebraicLaw` is law-tagged; no “closure-only” law) |
| Semigroup | associativity | `Associativity` | wired (bounded operational witness via lens apply tables) |
| Commutative monoid | + commutativity | `Commutativity` | wired (same witness discipline) |
| Monoid | + identity | `Identity` | **`NotYetImplemented`** (blocked until lens identity-element edge exists; PR-B.3 W2) |
| Semiring / ring | distributivity of `*` over `+`, etc. | **not in enum** | **flag §P1** — do not encode as a pretend `AlgebraicLaw` variant in fixtures |
| Lattice / Boolean algebra | absorption, distributivity of ∧/∨ | **not in enum** | **flag §P1** if modeled as `AlgebraicLawKind` extensions |

**Skeleton fixtures (Lane 2):** `src/v3/compiler/tests/fixtures/r3_verification_l7_algebraic_laws.dag` carries a
single `AlgebraicLaw(Identity, …)` placeholder claim — not a per-law explosion — aligned with
`docs/briefs/r3-v-l4-l7-direct-scaffold-notes.md` §algebra coverage audit.

**Lane 1 (`DifferentialEquals`):** `(rust_emit_output, dag_eval_output)` pairing is authored in
`r3_verification_l4_emit_eval_match.dag` but remains **`NotYetImplemented`** until the runner grows a
non–Lane-E-cost lineage pairing.

**Lane 5 (`ForAllTargets`):** `r3_verification_l5_corpus.dag` + `fixtures/r3_l5_corpus/add_then_branch_seed.v3`
— structural fixtures use `[]` for the argv list (not `empty()` calls in `.dag` data bodies). The runner still hits the default **`TestPredicate::ForAllTargets is not wired`** arm. The skeleton integration test asserts `TestClaim.source` equals the sidecar `.v3` bytes so program text stays single-authority under CI.
