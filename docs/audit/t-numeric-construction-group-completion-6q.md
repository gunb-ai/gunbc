# T-Numeric-Construction — `GroupCompletion<M>` 6Q substrate-introduction audit

**Lane:** R3 #6 (T-Numeric-Construction). **Authority:** [`docs/design-numeric-construction.md`](../design-numeric-construction.md). **Subject:** `GroupCompletion<M>` — a new algebra-surface declaring "the abelian group derived from a commutative monoid `M` via the Grothendieck construction." Prerequisite for the Slice 3 alias pivot `type Int = AbelianGroup<GroupCompletion<Nat>>`.

**Why this audit exists.** Slice 3's first attempt at `type Int = AbelianGroup<Nat>` ([#1422](https://github.com/gunb-ai/gunbc/pull/1422)) was reverted after two reviewers and Director ratified a sharp M9 finding: under the standard parametric reading of `AbelianGroup<T>`, `T` is the carrier of group operations including `inverse: fn(T) -> T`. With `T = Nat`, this asserts `inverse: fn(Nat) -> Nat` — denotationally false, since ℕ is a commutative monoid (not a group; no additive inverses). The Grothendieck construction creates ℤ *from* ℕ, but ℤ's carrier is *derived* (the quotient `(Nat, Nat) / ~` or the sign-magnitude representation `(Sign, Nat)`), not ℕ itself.

Director's decision (inbox #1288 #4360232423): keep `AbelianGroup<T>` standard (group carrier is `T`); introduce a separate `GroupCompletion<M>` algebra-surface that is honest about taking a commutative monoid and producing the derived abelian group. Then Slice 3 honestly becomes `type Int = AbelianGroup<GroupCompletion<Nat>>`.

**Hard boundary (per dispatch):** "No quotient-of-pairs or sign-magnitude representation in this prerequisite unless the design explicitly chooses it." This audit captures the algebra surface, not the carrier representation. Carrier representation is per-target grounding (Rust `i128`, Python `int`, etc. — emission selects).

## Design call — what is `GroupCompletion<M>`?

Three candidate shapes:

| Shape | Declaration | Tradeoffs |
|---|---|---|
| **A — algebra-only** | `type GroupCompletion<M> { /* abelian group ops over a derived carrier; carrier left unspecified */ }` | Clean alignment with design doc Option 3 ("abstract via algebra"). Carrier is a substrate-internal opaque `<M>`-derived type, surfaced only at emission. Risk: the substrate has no language for "carrier derived from M" without committing to representation. |
| **B — algebra-with-explicit-derived-carrier** | `type GroupCompletion<M> { carrier: ?, op, identity, inverse }` | More structurally explicit but requires a way to name the derived carrier — likely either a phantom-parameter (`Brand`-style) or a refinement that says "carrier is some opaque `Pair<M, M>`/`Sum<M, M>` derived from M." Pulls in carrier-representation facts the design doc rejected. |
| **C — abstract atom + algebra-witness pattern** | `type GroupCompletion<M>` (opaque atom; no fields) + a separate inhabitance witness pattern that says "Int inhabits AbelianGroup via GroupCompletion<Nat>" | Closest to Magnitude's shape (Slice 1 abstract counting). The atom names the construction; the AbelianGroup witness over the abstract `GroupCompletion<M>` carrier is structurally honest because the carrier is opaque, not Nat. |

**Recommendation: Shape C as a carrier construction** — opaque-atom `type GroupCompletion<M>` parameterized by a commutative-monoid type, denoting the **carrier** of "the abelian group derived from `M`." `GroupCompletion<M>` is **not** an algebra-with-derived-carrier; it is the carrier alone. The algebra witness is named separately at the use site as standard `AbelianGroup<T>` with `T = GroupCompletion<M>`.

This matches:
- Slice 1's `Magnitude` precedent (abstract opaque atom; carrier shape at the algebraic-axiom layer; algebra inhabitance attached at use sites via `Semiring<Magnitude>`).
- Design doc §3 Option 3's "without committing to a specific encoding" framing — `GroupCompletion<M>` opaqueness defers representation.
- Director's "keep `AbelianGroup<T>` standard: group carrier is `T`" boundary (inbox #1288 [#4360232423](https://github.com/gunb-ai/gunbc/issues/1288#issuecomment-4360232423)).
- Director's "no quotient-of-pairs or sign-magnitude representation facts in this prerequisite" boundary.

**Canonical Slice 3 form:** `type Int = AbelianGroup<GroupCompletion<Nat>>`.

Type-correctness under the standard parametric reading:
- `T = GroupCompletion<Nat>` is an opaque atom denoting "the abelian-group carrier derived from Nat."
- `AbelianGroup<T>`'s structural shape `{ op: fn(T,T)->T, identity: T, inverse: fn(T)->T }` instantiates to `{ op, identity, inverse }` over `GroupCompletion<Nat>`. `inverse: fn(GroupCompletion<Nat>) -> GroupCompletion<Nat>` is honest — every element of the group-completion carrier has an additive inverse by construction.
- The derivation rule (how `M`'s commutative-monoid structure produces the abelian group) lives in a future inhabitance lens, not in the substrate shape.

**Q6 single-authority resolution.** The earlier draft of this audit floated an alternative compact form `type Int = GroupCompletion<Nat>` (collapsing carrier + algebra into a single named type, treating `GroupCompletion<M>` as both the derived carrier and the implied AbelianGroup algebra witness). That admits two structurally distinct shapes for the same Slice 3 fact and would violate Q6 representation-duality. **Rejected.** Canonical form is the explicit two-step `AbelianGroup<GroupCompletion<Nat>>`: `GroupCompletion<M>` is **only** the carrier, and the algebra witness is **only** the standard `AbelianGroup<T>`. This keeps single authority for both surfaces (`GroupCompletion<M>` declares the derived carrier; `AbelianGroup<T>` declares the algebra structure) and prevents drift between "GroupCompletion as carrier" and "GroupCompletion as algebra-witness" readings.

## The 6 questions

### Q1 — Cardinality invariants
Does the type admit `[]` when invariant says ≥1, or singletons when ≥2?

**Answer: N/A (under Shape C — opaque atom).** No fields, no list members. PASS by construction.

### Q2 — Index/handle types
Does a raw `Int` / `NodeId` encode something with a domain restriction?

**Answer: N/A (under Shape C).** No fields. The single type-parameter `<M>` is a substrate type-reference, not a raw index. PASS.

### Q3 — Duplicated fact
Does Field A duplicate what's derivable from Field B?

**Answer: PASS.** Under Shape C, `GroupCompletion<M>` shares no structural surface with existing carriers. It is parametrically distinct from:
- `AbelianGroup<T>` (still standard parametric reading: T is the group's carrier).
- `Magnitude` (concrete abstract counting carrier; not a parametric construction).
- `Word*` (storage carriers at `dsl/std/bit.dag`).
- The existing `CommutativeMonoid<T>` and `Monoid<T>` algebras (these are the *input* algebra layers; `GroupCompletion<M>` is a derived-construction layer above them).

The construction is named exactly once in std; future Slice 3 (`type Int = AbelianGroup<GroupCompletion<Nat>>`) is the unique consumer for this slice.

### Q4 — Coproduct compression
Does one variant compress N distinct causes that downstream needs to distinguish?

**Answer: N/A under Shape C (opaque atom; not a sum type).** PASS.

(Adjacent: Shape B would carry a `carrier:` field that could compress representation choices — Shape C's opaqueness avoids that.)

### Q5 — Construction authority
Are multiple call sites independently constructing the same fact?

**Answer: PASS.** Single declaration in `dsl/std/algebra.dag` (or a new `dsl/std/group_completion.dag` for separation). No consumers in this prerequisite slice. Slice 3 (post-prerequisite) consumes via `type Int = AbelianGroup<GroupCompletion<Nat>>` as the unique authority for ℤ-as-derived-from-ℕ.

### Q6 — Representation duality
Can the same fact be expressed in two structurally different shapes?

**Answer: PASS for Shape C.** The opaque-atom form has exactly one structural shape. Shapes A and B are alternative encodings; the audit's recommendation (Shape C) is single-form by construction. The carrier-representation choice (quotient vs sign-magnitude) is deferred to per-target emission, not exposed in std/ — preventing parallel-representation drift.

## Where does it live?

Two options:
1. **In `dsl/std/algebra.dag`** alongside other algebraic-construction surfaces (`Monoid<T>`, `Group<T>`, `AbelianGroup<T>`, `Semiring<T>`, `Ring<T>`, `Field<T>`).
2. **In a new `dsl/std/group_completion.dag`** (or `dsl/std/algebra_constructions.dag`) — separates the derived-construction layer from the base algebra surfaces.

Recommend (1) for proximity to existing algebra surfaces and to keep the algebra-surface dependency story compact. The construction-chain `Magnitude → Nat → Int` lives in separate `dsl/std/{magnitude,nat,integer}.dag` files; the algebra surfaces stay in `algebra.dag`.

## Constrained-inhabitance gap (tracked-scaffold note)

`GroupCompletion<M>` is **denotationally** parameterized over "a commutative monoid `M`" — Grothendieck's construction is well-defined only when `M` carries the commutative-monoid laws. The current substrate has no parametric where-clause syntax to enforce this structurally (no `<M> where M : CommutativeMonoid<_>` form), so the recommended Shape C declaration accepts any type-reference for `<M>` at the parser/lower level. This is a known substrate-feature gap, not specific to this audit.

**Two ways the gap is bounded in practice for this lane:**

1. **Slice 3's only consumer is `GroupCompletion<Nat>`.** `Nat = Semiring<Magnitude>` (Slice 2) carries `(Nat, +, 0)` as a commutative monoid by the Semiring algebra's structural definition (`add: fn(T, T) -> T` + `zero: T` + commutativity-on-add law). The single intended consumer denotationally satisfies the precondition. No other consumer in the construction-chain plan targets `GroupCompletion<M>` for arbitrary `M`.

2. **Future constrained-inhabitance dissolves the gap.** When the substrate gains a parametric where-clause / inhabitance-constraint surface (this is a separate substrate-feature lane, not in T-Numeric-Construction's scope), `GroupCompletion<M>` tightens to `GroupCompletion<M> where M inhabits CommutativeMonoid` (or whatever the chosen syntax is). Existing call sites (`type Int = AbelianGroup<GroupCompletion<Nat>>`) continue to type-check because `Nat` denotationally inhabits `CommutativeMonoid` already.

**Dissolution trigger:** when constrained-inhabitance / parametric where-clause syntax lands in the substrate, sharpen `GroupCompletion<M>` to require `M : CommutativeMonoid` (or `M : Semiring`-with-commutative-add, depending on the chosen algebra-strength). No consumer migration needed.

**Mitigation in this slice:** the substrate-introduction PR for `GroupCompletion<M>` should land alongside a structural ratchet that pins `GroupCompletion<Nat>` (the only intended consumer) at bootstrap time — making the denotational precondition observable as a use-site fact rather than an unconstrained parametric admission. The same pattern as Slice 2's tracked scaffold for `Semiring → CommutativeSemiring`.

## What this audit does NOT cover

- **Carrier representation.** Per Director's hard boundary, no quotient-of-pairs or sign-magnitude facts. Per-target representation is emission's job.
- **Algebra inhabitance proof.** Whether `GroupCompletion<M>` mechanically derives an `AbelianGroup` witness over its carrier is a follow-up modeling question; this audit pins the substrate shape, not the inhabitance lens.
- **Slice 3 alias-pivot edit.** Authoring `type Int = AbelianGroup<GroupCompletion<Nat>>` is the post-prerequisite slice; this audit gates that.
- **Refinement syntax.** `Int<N>` refinements are gated on T-V2-Retirement per design doc §"path (a) coordination."
- **Parametric where-clause / constrained inhabitance.** Required to structurally enforce `<M> : CommutativeMonoid`. Tracked above as a substrate-feature gap; Slice 3 lands `GroupCompletion<Nat>` as the only consumer in the meantime.

## Verdict

**PASS — proceed with `type GroupCompletion<M>` opaque-atom (Shape C) as a substrate-introduction PR.**

- Sized: **S** — single new type declaration in `dsl/std/algebra.dag` (or new file). No fields, no algebra-inhabitance shape declared at this layer (left to future inhabitance lens). Bootstrap regen.
- Hard boundaries: no carrier-representation facts; no migration of `Int = Int64` (still gated on this slice landing); no tokenizer/literal-grammar work.
- Once landed, Slice 3 becomes a one-line edit (`type Int = AbelianGroup<GroupCompletion<Nat>>`) plus structural ratchet — no further substrate-introduction work needed.

## Cross-refs

- `docs/design-numeric-construction.md` §3 (construction-chain layer 3; Option 3 abstract-via-algebra).
- `docs/audit/t-numeric-construction-magnitude-6q.md` (Slice 1 audit; opaque-atom precedent).
- `feedback_substrate_principle_audit` (the 6Q rule).
- `feedback_compositional_not_templating`, `feedback_naming_is_aliasing` (refinement-as-child rationale).
- Director ratifications:
  - inbox #1288 #4359832983 (Slice 3 alias-pivot Option 1, original direction).
  - inbox #1288 #4360232423 (Option 2 substrate-split decision after M9 finding; this audit's authority).
- M9 reviewer findings on PR #1422:
  - https://github.com/gunb-ai/gunbc/pull/1422#issuecomment-4360211927 (carrier-completion shape).
  - https://github.com/gunb-ai/gunbc/pull/1422#issuecomment-4360217139 (Director-escalated parametric-reading concern).
