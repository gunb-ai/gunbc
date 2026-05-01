# T-Numeric-Construction Fast Verification Audit

**Status:** AUDIT RECEIPT (no implementation). Authored 2026-05-01 for the
R3 Substrate dispatch "T-Numeric-Construction fast verification audits —
AbelianGroup, Field, String".

**Authority read:** `docs/design-numeric-construction.md`,
`dsl/std/algebra.dag`, `dsl/std/types.dag`, and
`dsl/std/string_type.dag`.

## Result Matrix

| Surface | Verdict | Disposition |
|---|---|---|
| `AbelianGroup<G>` | **VERIFY / no new substrate** | Existing `dsl/std/algebra.dag` carrier is sufficient for `Int = AbelianGroup<Nat>` at the current algebra-carrier abstraction. |
| `Field<F>` | **GAP** | Existing carrier names the field algebra, but its inverse surface is not structurally fail-closed for zero. Rational construction needs a non-zero inverse boundary before treating `Field<Int>` as complete executable substrate. |
| `String` | **VERIFY / no new substrate** | `String = FreeMonoid<Char>` is already the std authority. No width-baked or encoding-baked substrate fact found in the carrier. |

## 1. `AbelianGroup<G>`

`dsl/std/algebra.dag:131-137` declares:

```dag
type AbelianGroup<T> {
  op: fn(T, T) -> T
  identity: T
  inverse: fn(T) -> T
  // commutativity is a law
}
```

The immediately preceding `Group<T>` surface (`dsl/std/algebra.dag:124-129`)
documents the inverse law, while the algebra hierarchy header treats
associativity and commutativity as algebra laws, not runtime fields. At this
abstraction, the carrier structurally exposes the executable pieces needed by
the construction chain:

- closed binary operation: `op`
- additive identity: `identity`
- additive inverse: `inverse`
- abelian/group law identity: declared by the `AbelianGroup<T>` carrier itself

**Disposition:** VERIFY / no new substrate for the fast construction brief.
`Int = AbelianGroup<Nat>` can consume this existing algebra carrier without a
new `AbelianGroup` shape. If a later proof-carrying lane wants laws as first
class values, that is a general algebra-law-witness introduction, not a
numeric-construction blocker specific to `AbelianGroup`.

## 2. `Field<F>`

`dsl/std/algebra.dag:196-206` declares:

```dag
type Field<T> {
  add: fn(T, T) -> T
  zero: T
  negate: fn(T) -> T
  mul: fn(T, T) -> T
  one: T
  reciprocal: fn(T) -> T
  compare: fn(T, T) -> Ordering
}
```

The carrier has the expected ring-plus-inverse operation fields: additive
operation, zero, negation, multiplication, one, and reciprocal. The comment at
`dsl/std/algebra.dag:196` correctly states the mathematical domain:
`T \ {zero}` under multiplication.

The structural gap is that the domain restriction is not represented in the
field type. `reciprocal: fn(T) -> T` admits `zero` as an input at the carrier
boundary, and there is no `NonZero<T>` refinement, dependent input guard, or
`Result<T, DivError>` shape for the inverse. This is sharper than the
`AbelianGroup` law issue because division by zero is an executable partiality
boundary; `OrderedRing<T>` already uses `div: fn(T, T) -> Result<T, DivError>`
at `dsl/std/algebra.dag:179-186`.

**Exact missing carrier/refinement:** one of the following must land before the
Rational construction treats `Field<Int>` as complete executable substrate:

- preferred structural shape: `NonZero<T>` refinement plus
  `reciprocal: fn(NonZero<T>) -> T`; or
- checked-operation shape matching the existing `OrderedRing` precedent:
  `reciprocal: fn(T) -> Result<T, DivError>` and/or
  `div: fn(T, T) -> Result<T, DivError>`.

**Disposition:** GAP for `Rational = Field<Int>` if the construction brief
needs fail-closed executable division. No broad implementation in this audit
slice.

## 3. `String`

`dsl/std/string_type.dag` is already explicit:

```dag
type String = FreeMonoid<Char>
```

The local comments identify `String` as the free monoid over `Char`, and
`dsl/std/algebra.dag:70-77` gives the same denotation: every string is a unique
finite sequence of characters with no additional relations. The `FreeMonoid<T>`
carrier at `dsl/std/algebra.dag:280-324` supplies concatenation, empty,
append, slicing, length/count, and sequence transforms at the algebra level.

`dsl/std/types.dag:187-196` currently declares `Char = Int` as a
v2-compatible alias and documents the intended Unicode scalar-value refinement.
That is a known refinement-syntax gap, but it does not bake String width or
encoding into the substrate carrier. UTF-8 appears in target/grounding
projection code (for example `grounding_lifetime` names
`Utf8FreeMonoidChar`) rather than in `String`'s std definition.

**Disposition:** VERIFY / no new substrate for `String` in
T-Numeric-Construction. Route only the existing `Char` refinement-syntax gap
through the construction/v2-retirement coordination if a later worker needs
the Unicode scalar bound executable in `.dag`; do not introduce a new String
carrier.
