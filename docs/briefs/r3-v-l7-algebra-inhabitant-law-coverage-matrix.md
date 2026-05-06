# R3 L7 Algebra Inhabitant Law Coverage Matrix

Status: PROPOSAL / research-only. This extends PR #1419's
`(algebra, law)` fixture matrix into the Director-ratified
`(algebra, inhabitant, law)` coverage surface. No substrate edits, no fixture
authoring, no new `AlgebraicLawKind` variants, and no new `TestPredicate`
variants are proposed here.

## Authority

- Path-grounded sources verified at HEAD with `git cat-file -e`:
  `dsl/std/algebra.dag`, `docs/r3-structure.md`, and
  `docs/briefs/r3-v-l7-algebra-coverage-matrix.md`.
- [`docs/r3-structure.md`](../r3-structure.md) §"Acceptance" — gate row
  `section_ref_substrate_landed` under **T-Lens-Application-Surface**, and the
  **Demonstration principle** worked-example row `crdt_cost_basis_demonstrated`,
  expand T-V-L4-L7-Direct to exhaustive per-(algebra, inhabitant, law) witness coverage.
- PR #1419 is the direct lineage: it authored 17 enum-backed
  `(algebra, law)` rows for the current `AlgebraicLawKind` surface.
- `dsl/std/algebra.dag:30-47` defines the law-emergence table;
  `:51-87` names denotational inhabitants; `:459-467` and `:515-522`
  name the current compiler-enriched kernel inhabitant profiles.
- `src/v3/std/verification.dag:104-107` currently exposes only
  `Associativity`, `Commutativity`, and `Identity` as `AlgebraicLawKind`
  variants. Distributivity, absorption, inverse, complement, annihilation,
  order laws, and approximate-field laws remain substrate-introduction
  candidates under INVARIANTS P1.

## Why the Inhabitant Axis Matters

The pair matrix can say "Semiring x Distributivity exists" once, but that
does not prove every semiring inhabitant has a witness. The motivating bug
class is PR #1430 section A / the local design receipt at
[`docs/design-cost-lens-sizevar-dimension-wiring.md`](../design-cost-lens-sizevar-dimension-wiring.md)
§"3.2 Where the consumer attaches in the lens": `SymbolicCost`
normalization treated additive zero and multiplicative zero with one helper,
so a product containing zero failed to collapse to zero. The invariant is not
just "Semiring has laws"; it is "`SymbolicCost` as a Semiring inhabitant has
the semiring annihilation witness."

The expanded matrix therefore treats each concrete inhabitant family as its
own witness obligation. This does not bypass P1: laws absent from
`AlgebraicLawKind` still cannot be fixture-encoded locally.

## Coverage Legend

- Wired: enum-backed law exists and the current runner has a bounded
  operational witness path for that law.
- NYI: enum-backed law exists but the current runner returns
  `NotYetImplemented`.
- P1: the law is theory in `dsl/std/algebra.dag` but lacks an
  `AlgebraicLawKind` substrate variant or carrier edge.
- Profile: live compiler-enriched inhabitant in `kernel_algebra_profile`.
- Declared: live adjacent declaration outside `kernel_algebra_profile`.
- Candidate: design-ratified or queued inhabitance, not yet a live substrate
  row for L7 fixture authoring.

## Current Runner Surface

`Associativity` and `Commutativity` are wired as bounded operational witnesses.
`Identity` is enum-backed but blocked on the lens identity-element edge.
`Distributivity` is intentionally absent from `AlgebraicLawKind`; the runner
routes any future non-enum law through P1 rather than accepting a fixture-local
encoding.

## Inhabitant Matrix

| Inhabitant family | Live source | Algebra surface | Enum-backed obligations | Missing-law obligations | Disposition |
|---|---|---|---|---|---|
| Bool | Profile: `kernel_algebra_profile["Bool"] = BooleanAlgebraProfile`; denotational Bool inhabits BooleanAlgebra | BooleanAlgebra / bounded lattice | meet/join associativity Wired; meet/join commutativity Wired; top/bottom or join identity NYI | complement, absorption, distributivity P1 | Needs per-Bool rows when L7 moves from pair skeleton to exhaustive fixtures. |
| Set<A> | Profile: `kernel_algebra_profile["Set"] = BooleanAlgebraCollectionProfile`; denotational Set<A> pointwise BooleanAlgebra | BooleanAlgebra<A> / bounded lattice | same enum-backed lattice/Boolean rows as Bool, but over Set<A> | complement, absorption, distributivity P1 | Separate from Bool; pointwise lifting is a distinct inhabitant witness. |
| String | Profile plus `dsl/std/string_type.dag:14-16` | FreeMonoid<Char> | concat associativity Wired; empty identity NYI | none on current FreeMonoid law surface beyond enum-backed rows | Needs String-specific rows; PR #1419's placeholder does not establish Char-sequence inhabitance. |
| List<T> | Profile: `kernel_algebra_profile["List"] = FreeMonoidCollectionProfile`; denotational List<T> inhabits FreeMonoid<T> | FreeMonoid<T> | concat associativity Wired; empty identity NYI | element-parametric witness coverage per concrete T remains a later monomorphization question | Needs at least one List<T> family row and future concrete T expansion if T becomes executable witness data. |
| Map<K,V> | Profile: `kernel_algebra_profile["Map"] = PartialFunctionProfile`; denotational Map<K,V> inhabits PartialFunction<K,V> | PartialFunction<K,V> | no current `AlgebraicLawKind` row in PR #1419 | merge associativity/identity/conflict behavior P1 | Track as an inhabitant surface, but do not author `AlgebraicLaw` rows until a law variant exists. |
| Nat | Declared: `dsl/std/nat.dag:55` is `Nat = Semiring<Magnitude>`; `algebra.dag:55-59` says denotationally CommutativeSemiring | Semiring now; CommutativeSemiring sharpening pending | additive commutativity Wired; multiplicative identity NYI if represented through current enum surface | distributivity, annihilation, multiplication associativity-by-operation, future multiplicative commutativity sharpening P1 | Declared live but not kernel-profiled; matrix must mark Semiring status separate from future CommutativeSemiring sharpening. |
| UInt8/16/32/64/128 | Declared: `dsl/std/integer.dag:57-61`; Rust target rows mirror unsigned Semiring carriers | Semiring<Word*> | additive commutativity Wired; multiplicative identity NYI where fixture chooses the mul lens | distributivity and annihilation P1 | Each width is a separate inhabitant obligation because overflow/range facts differ by carrier. |
| Int8/16/32/64/128 | Declared: `dsl/std/integer.dag:50-54`; Rust target rows mirror signed OrderedRing carriers | OrderedRing<Word*> | additive commutativity Wired; multiplicative identity NYI | additive inverse, order compatibility, distributivity P1 | Fixed-width signed rows stay distinct from abstract Int after the construction-chain pivot. |
| Int | Declared: `dsl/std/integer.dag:83` is `AbelianGroup<GroupCompletion<Nat>>`; kernel profile still maps "Int" to OrderedRingProfile | transitional abstract integer | Abelian-group identity NYI / commutativity Wired if using current enum surface | OrderedRing/Ring residual requires cascade decision; order and distributivity P1 | Audit-sensitive: do not collapse kernel profile and construction-chain alias into one witness without a lane decision. |
| Float | Profile: `kernel_algebra_profile["Float"] = ApproximateFieldProfile`; denotational Float is ApproximateField | approximate field, not exact Field | exact associativity should not be claimed for floating addition; current enum laws need approximate semantics before fixture use | approximate identity/rounding, reciprocal/division, order laws P1 | Exclude from exact L7 law closure until approximate-law substrate shape exists. |
| SymbolicCost | Candidate: [`design-cost-lens-sizevar-dimension-wiring.md`](../design-cost-lens-sizevar-dimension-wiring.md) §"4. SymbolicCost commutative-semiring discipline (product-zero bug class)" proposes `Semiring<SymbolicCost>`; §"8.3 `Dimension<SymbolicCost>` declaration scope for slice 2" resolves Semiring, not CommutativeSemiring | Semiring<SymbolicCost> candidate | no PR #1419 fixture row yet | annihilation/product-zero, distributivity, add/mul identity P1 | Canonical proof that per-inhabitant coverage is required; author only after the semiring declaration lands. |

## Implications for PR #1419 Fixture Lineage

PR #1419 remains a valid enum-surface receipt: it enumerates the law tags that
the current `AlgebraicLaw` predicate can consume. It is not exhaustive after
the #1480 fold-in because its placeholder Int-like operations do not name each
inhabitant family above. The next fixture-authoring slice should therefore add
rows by inhabitant cluster, not just by algebra name.

Recommended sequencing:

1. Keep the current 17-row pair fixture as the enum-backed law skeleton.
2. Add per-inhabitant rows only for laws already expressible by
   `AlgebraicLawKind`.
3. Route every non-enum law as a P1 substrate-introduction candidate.
4. Treat `SymbolicCost` as blocked until its `Semiring<SymbolicCost>`
   inhabitance lands; do not encode product-zero as a one-off runner check.

## Dispatch Readiness

Option B: exhaustive L7 coverage is not fixture-dispatchable as one complete
implementation slice today. The dispatchable next unit is a bounded
per-inhabitant fixture extension for enum-backed `Associativity`,
`Commutativity`, and `Identity` over live kernel/declaration inhabitants, with
`Identity` still expected to return `NotYetImplemented`. The complete
`l7_algebraic_laws_witnessed` gate remains gated on P1 law-surface expansion
for distributivity, absorption, complement, inverse, annihilation, order, and
approximate-field semantics.
