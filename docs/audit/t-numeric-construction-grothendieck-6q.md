# T-Numeric-Construction - Grothendieck `Int` 6Q design audit

**Lane:** R3 #6 (T-Numeric-Construction). **Authority:** [`docs/design-numeric-construction.md`](../design-numeric-construction.md). **Subject:** the Grothendieck layer for `Int` as the completion carrier over `Nat` in the `Magnitude -> Nat -> Int -> Rational -> Real` construction chain.

This audit runs `feedback_substrate_principle_audit` on the three candidate encodings named in the design doc before any `Int` substrate declaration or consumer migration. This PR is docs-only: it records the encoding decision and does not edit parser, literal, tokenizer, refinement syntax, or existing `Int` consumers.

## Encoding design call

Three candidate encodings from the design doc:

| Encoding | Shape | Q3 risk | Q6 risk | Verdict |
|---|---|---|---|---|
| Quotient of pairs | `Int = (Nat, Nat) / ~`, where `(a, b) ~ (c, d)` iff `a + d = c + b` | Introduces a pair representation and quotient relation beside the abstract completion authority | Requires equivalence-class quotient machinery before there is a concrete consumer that needs it; equality could be represented both by quotient normalization and group laws | **Reject** |
| Sign-magnitude | `Int = (Sign, Nat)`, with `Pos 0 == Neg 0` collapsed | Duplicates sign and magnitude facts that the additive inverse and identity laws already own | Exposes a representation target groundings may choose internally; the zero-collapse rule either creates a second equality authority or requires quotient/normalization machinery | **Reject** |
| **Abstract via algebra** | `Int` is the Grothendieck completion carrier over `Nat`, with `AbelianGroup<Int>` as its algebra witness | None for this decision: no sign, magnitude, or pair fields are added | None for this decision: one substrate authority, with concrete representation deferred to target grounding | **Accept with declaration deferred** |

**Decision: abstract via algebra, not direct aliasing to the witness.** The accepted owner is the abstract Grothendieck completion of `Nat`: the integer carrier is completed from `Nat`, and the additive group laws are witnessed by `AbelianGroup<Int>`. The existing `AbelianGroup<T>` in `dsl/std/algebra.dag` is a witness over an already-existing carrier `T`; it is not itself a carrier constructor. Therefore the spelling `type Int = AbelianGroup<Nat>` is not a valid current substrate declaration: it would require `Nat` itself to have additive inverses.

Until the substrate can express either a distinct completion carrier (for example, a future `GrothendieckCompletion<Nat>`-shaped authority) or carrier/witness syntax (`Int` as carrier, `AbelianGroup<Int>` as witness), this audit is a design decision only. Representation details such as sign bit, limb layout, pair normalization, or target-native integer width belong to grounding and refinement slices, not to the comparable substrate shape.

No STOP+PING is required for the encoding choice: the design doc names option 3 as the PM recommendation, and Q3/Q6 both point the same way. The declaration form is deferred because the current substrate only has the algebra witness record, not a completion-carrier constructor or carrier/witness spelling.

## The 6 questions

### Q1 - Cardinality invariants

Does the type admit `[]` when invariant says >=1, or singletons when >=2?

**Answer: PASS for abstract-via-algebra.** The accepted encoding introduces no list or tuple fields. Quotient-of-pairs would need a two-coordinate product carrier and a quotient relation; sign-magnitude would need at least one sign variant plus one magnitude carrier. Those are representational commitments, not required cardinality facts for `Int`.

### Q2 - Index/handle types

Does a raw `Int` / `NodeId` encode something with a domain restriction?

**Answer: PASS.** The abstract encoding adds no index, handle, limb offset, sign tag, or normalization handle. Pair and sign-magnitude encodings would each need additional domain constraints (`Nat` coordinates in a quotient relation, or a zero-collapse invariant) before the value is well formed.

### Q3 - Duplicated fact

Does Field A duplicate what's derivable from Field B?

**Answer: ACCEPT option 3; reject options 1 and 2.** The completed integer carrier plus its `AbelianGroup<Int>` witness makes additive identity and inverse the authority for integer structure. A sign-magnitude substrate shape would duplicate that authority by separately storing sign and magnitude, then patching the duplicate-zero case with `Pos 0 == Neg 0`. A quotient-of-pairs shape would store positive and negative components while also relying on algebraic addition to decide equality.

For substrate, the important fact is not "how a target stores a negative integer." The important fact is that the Grothendieck completion of `Nat` has additive inverses. The witness is over the completed carrier, not over raw `Nat`, so no parallel sign or pair representation is added.

### Q4 - Coproduct compression

Does one variant compress N distinct causes that downstream needs to distinguish?

**Answer: PASS for abstract-via-algebra.** No coproduct is introduced. Sign-magnitude would introduce `Sign = Pos | Neg`; that coproduct is not currently needed by downstream substrate consumers, and the only special case it would expose is the duplicate representation of zero.

### Q5 - Construction authority

Are multiple call sites independently constructing the same fact?

**Answer: PASS with explicit sequencing.** The construction authority should be:

1. `Magnitude` as the terminal counting carrier.
2. `Nat = Semiring<Magnitude>` as the natural-number algebra.
3. `Int` as the Grothendieck completion carrier over `Nat`, with `AbelianGroup<Int>` as its additive-group witness.

This audit does not author step 2 or step 3. It only records that when step 3 lands, the completion carrier is the construction authority and `AbelianGroup<Int>` is the algebra witness. The existing `dsl/std/integer.dag` aliases remain legacy consumers until the design doc's migration slice replaces them; this audit does not create a second `Int` surface.

### Q6 - Representation duality

Can the same fact be expressed in two structurally different shapes that comparison treats differently?

**Answer: ACCEPT option 3; reject options 1 and 2.** This is the decisive question for the Grothendieck layer. Quotient-of-pairs and sign-magnitude are both valid implementation strategies, but as substrate encodings they create a second structural way to express the same integer fact:

- quotient-of-pairs: equality lives both in `(a, b) ~ (c, d)` and in algebraic group laws;
- sign-magnitude: negativity and zero live both in `Sign + Nat` fields and in additive inverse/identity laws.

An abstract completion carrier has one structural shape. Target groundings may choose pairs, sign-magnitude, two's-complement words, arbitrary-precision limbs, or native integers internally, but those choices do not become comparable substrate facts.

## Verdict

**PASS - choose abstract-via-algebra for the Grothendieck layer, with declaration deferred.**

- Recommended encoding: `Int` is the abstract Grothendieck completion carrier over `Nat`; `AbelianGroup<Int>` is the algebra witness.
- Rejected current spelling: `type Int = AbelianGroup<Nat>`, because `AbelianGroup<T>` is a witness over an existing carrier, not a carrier constructor, and would require impossible inverses on raw `Nat`.
- Rejected: quotient-of-pairs, because quotient/equivalence-class machinery is heavy substrate with no current consumer and would duplicate equality authority.
- Rejected: sign-magnitude, because sign and magnitude are target representation facts and would duplicate the algebraic inverse/identity authority, especially at zero.
- Sized: docs-only design decision now; later substrate declaration is S-M depending on whether carrier/witness syntax already exists or a distinct completion carrier must be introduced.
- Hard boundaries: no parser, tokenizer, literal, refinement-syntax, or `Int` consumer migration in this slice.

## Out of scope

- Authoring `Nat = Semiring<Magnitude>`.
- Rewriting `dsl/std/integer.dag` default aliases or fixed-width integer consumers.
- Introducing quotient types, sign enums, normalization rules, or representation-specific equality.
- Target realization for Rust, Python, Go, or any other backend.
- Refinement syntax such as `Int<N>` or range-refinement composition.

## Cross-refs

- `docs/design-numeric-construction.md` section 3 (Grothendieck construction for `Int` over `Nat`).
- `docs/audit/t-numeric-construction-magnitude-6q.md` (preceding `Magnitude` encoding audit).
- `dsl/std/algebra.dag:132` (`AbelianGroup<T>` fields: `op`, `identity`, `inverse`).
- `dsl/std/integer.dag` (legacy default integer aliases, intentionally untouched here).
