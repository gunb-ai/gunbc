# Design: General Value-Set Lattice (non-integer containment for derivable coercion)

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). No code lands from
> this doc without the consumer named in §6 (E-10). Part of the coercion thesis arc: extends the
> `find_witness` completion (R1 widening / R2 narrowing-refuse / R3 project-to-core, gunbc#4585)
> from integer value-sets to a general — but deliberately *carved* — class of refined types.
>
> The in-tree dissolution marker for this exact design already exists:
> `src/v2/std/integer_value_set.dag:30` — "dissolve-on-arrival: **general value-set lattice
> authority lands** and integer-only decode can fold into the shared substrate predicate"
> (bound task `node://adhoc-248e4e6c-3da`, shared with the `model_core` axis-role classifiers).
> This doc is the design that marker is waiting on.

## 1. Problem

`find_witness` derives safe coercion when it can decide *value-domain containment* between two
refined types. Today that decision exists for exactly one shape: `IntegerValueSet =
Unbounded | Interval` with containment via `integer_interval_spec_node_contains`
(`src/v2/std/integer_value_set.dag`). R1 (`refinement_widening_preservation_holds`) and R2
(`refinement_widening_is_strict_coarsening`) consume it through
`src/v2/std/refinement_widening_predicate.dag:63-86`.

The general problem — "is every value of refined type A also a value of refined type B?" — is
**predicate implication, which is undecidable** for arbitrary predicates. `v2.std.refinement`'s
`Validation<B>` carriers are exactly such predicates (`admits: B -> Bool`): they can *discharge
membership of one value* at a construction boundary, but two `Validation`s cannot be compared.
So the design problem is to carve a **closed description language for value domains** whose
containment is structurally decidable, general over a *class* of types rather than per-pair,
and honest about its edge: anything outside the language fails closed, never guesses.

This is the thesis boundary applied to itself: *derive what's determined; refuse loss,
ambiguity, and the inexpressible.* "Decidable, not heuristic" forbids an SMT escape hatch.

## 2. What already exists (M9 DFS — the design attaches, it does not invent)

| Concept | Where | Role here |
|---|---|---|
| `IntegerValueSet` + `integer_value_set_contains` / `_strictly_contains` | `src/v2/std/integer_value_set.dag` | the **template and the first leaf**: Node fact-bundle encoding with a closed `kind` field, fail-closed decode, containment + strictness. Folds into the general authority per its own marker. |
| R1/R2 predicates (the consumers, landed #4585) | `src/v2/std/refinement_widening_predicate.dag` | `refinement_widening_value_sets_hold` (containment ∧ strict) and `refinement_widening_is_strict_coarsening` (reverse strict ⇒ `WouldLoseInformation`) — the swap point. `^refinement_value_set_unknown` (line 26) is the existing fail-closed floor for "no description." |
| `find_witness` closed-candidate fold + rule dispatch | `src/v2/std/find_witness.dag` | unchanged. The general lattice plugs in *under* `preservation_rule_refinement_widening`; no new rule symbol, no fifth fold variant. |
| `Validation<B>` / `Refined<B>` predicate refinements | `src/v2/std/refinement.dag` | the **complement**, not a competitor: predicates discharge membership at constructor boundaries (runtime, per-value); value-set descriptions make refinement *pairs* comparable (compile time). §4.5 fixes their relationship. |
| `ModelCorePrimitiveFactAxis` (5 core axes + surface spelling) | `src/v2/std/model_core.dag:38` | the **anchor**: a value-set description denotes the `Range` axis of a carrier's semantic core; descriptions are core-axis-anchored so containment composes with R3 instead of bypassing it. |
| `BoundedLattice` inhabitance machinery | `dsl/std/algebra.dag`, `src/v2/std/bounded_lattice_completeness.dag` | the order-theoretic home when meet/join gain a consumer (§8 Q-V4); wave 1 needs only the containment partial order. |

**Substrate target named (P1):** one new `std/` module, `src/v2/std/value_set.dag`, as the
single containment authority; `integer_value_set.dag` dissolves into it (its marker is the
receipt); `refinement_widening_predicate.dag` swaps its import in the same change. No
connective/behavior extension; descriptions are ordinary `Conj` fact-bundles.

## 3. Substrate-fact introduction procedure (MODELING.md, cited)

- **Step 1 (DAG-ancestor):** ran. The ancestor concept is "a description of a set of values,
  ordered by inclusion" — abstract-interpretation value domains (Cousot & Cousot 1977) and
  the existing `IntegerValueSet` instance. The general type is declared as the parent;
  `IntegerValueSet` retrofits as one kind (the BoundDeclaration/`Interval<D>` precedent from
  INVARIANTS Step-1 worked examples).
- **Step 2 (coproduct-vs-coordinate):** ran. The description kinds (§4.1) are genuine
  **alternatives** — one description is one kind — so a closed coproduct is correct. Inside
  kinds, `Product` fields are coordinates (record), `TaggedUnion` arms are alternatives —
  each applied per the test.
- **Step 3 (primitive-vs-lens-extensible):** ran. The kind vocabulary is **substrate-declared
  and closed** — decidability of containment is proven per kind-pair, so an open
  (lens-extensible) kind set would forfeit the decidability claim. Extending the family is a
  substrate change with a new decidability argument, by design.

## 4. Design

### 4.1 The closed description family

`ValueSet` descriptions, encoded exactly like the integer template (a `Conj` fact-bundle with
a `kind` field from a closed symbol vocabulary; fail-closed decode to a typed coproduct):

```
ValueSet
  = ValueSetEmpty                                  // ⊥ — uninhabited (Never-backed refinements)
  | ValueSetUniversal                              // ⊤ for the anchored carrier (folds in IntegerValueSetUnbounded)
  | ValueSetIntegerInterval { interval: Node }     // folds in IntegerValueSetInterval (existing decode + containment)
  | ValueSetEnumeration { members: List<Node> }    // finite, canonical-form member nodes (enum subsets, symbol sets)
  | ValueSetProduct { fields: List<NamedValueSet> }    // record refinement: per-field descriptions (Conj)
  | ValueSetTaggedUnion { arms: List<TaggedValueSet> } // sum refinement: per-variant descriptions (Disj)
```

Every description carries one more fact: its **anchor** — the carrier class + core axes it
denotes (`Range` over which `model_core` fact-bundle). The anchor is what makes §4.4's
composition with project-to-core sound.

Deliberately **not** in wave 1 (each is a later kind with its own decidability note, added
only with a consumer): float intervals (total-order semantics for NaN/-0.0 must come from the
float fact-bundle first — `v2.std.float`), text/regex languages, length-refined collections
(`{ element: ValueSet, length: interval }` — the shape is known, the consumer is not).
Anything not in the family is **inexpressible** and takes the `refinement_value_set_unknown`
floor: containment is false, the coercion refuses. No partial credit.

### 4.2 The containment relation (one structural recursion, closed rule table)

`value_set_contains(container, contained) -> Bool`, by recursion on the description pair:

- `(_, Empty)` → true. `(Empty, x)` → only if x = Empty. `(Universal, x)` → true **iff anchors
  match**; `(x, Universal)` → false unless x = Universal.
- `(IntegerInterval, IntegerInterval)` → existing endpoint logic (ported, not rewritten).
- `(Enumeration, Enumeration)` → finite subset over canonical member nodes.
- `(IntegerInterval, Enumeration-of-integers)` → every member in interval — the one blessed
  cross-kind rule inside a carrier class, stated explicitly in the table.
- `(Product, Product)` → field-name sets must be **equal**; then pointwise containment per
  field. Missing/extra fields are not defaulted to ⊤ — that is the fabrication pattern (C-8);
  unequal field sets → false.
- `(TaggedUnion, TaggedUnion)` → every contained arm's tag exists in the container with
  containment on the payload; cross-tag subsumption is refused (tags are nominal identity —
  the A3 brand discipline applied to value sets).
- **Every other pair → false** (fail-closed default; the table is closed).

Strictness is **semantic**: `value_set_strictly_contains(a, b) = contains(a,b) && !contains(b,a)`
(already the integer module's pattern at `integer_value_set.dag:182`).

**This fixes a latent bug-class in R1 as it stands:** `refinement_widening_value_sets_hold`
currently requires `source_value_set != candidate_value_set` — *syntactic* inequality. Two
syntactically different descriptions of the *same* set (e.g. interval `[1,3]` vs enumeration
`{1,2,3}`) would pass the current "strict widening" check while widening nothing. The general
design replaces the `!=` with semantic strictness; the discriminating claim for this exact
case is in the minimal slice (§6).

### 4.3 Decidability argument (the carve, stated honestly)

- Descriptions are finite acyclic `Node` trees; the recursion descends structurally on the
  pair (TreeSize on the container, lexicographically the contained). This module is
  structurally terminating on its own — it is **not gated on** the termination lane; when
  that checker lands (`docs/design-termination-checker.md` §4.2) it *validates* this
  recursion as an early worked example, but nothing here waits for it.
- Each leaf rule is a finite comparison (integer endpoint compare; finite-list subset;
  field-name set equality). No quantifier instantiation, no predicate evaluation, no search.
- The rule table is closed × closed: adding a kind forces writing its row against every kind
  (the compiler's exhaustive match makes omission unrepresentable, P3 "no case enumeration
  for open sets" satisfied by the typed coproduct, not a string default).
- What is *given up*, on purpose: arbitrary `where`-predicate refinements never get derived
  coercion from this lane. They keep `Validation<B>` runtime discharge and the fail-closed
  refusal. The thesis line — "the compiler detects the mismatch but cannot invent the
  resolution" — applies: inexpressible domains are surfaced, not approximated.

### 4.4 Composition with project-to-core (R3) — the anti-rig discipline

The anti-rig requirement is that one relation accepts widening, refuses coarsening, **and
refuses wrong-core** — a containment check must not be riggable by comparing raw structure
across different semantic cores. Mechanism:

- A description's **anchor** is produced by core projection (the same `model_core` core-axis
  facts R3 compares), not authored free-form. The value set is the denotation of the
  **Range** axis of an already-core-projected fact-bundle.
- `value_set_contains` short-circuits false on anchor mismatch, *before* any structural rule
  runs. Wrong-core pairs therefore cannot reach a structural-containment accept — the R3
  verdict and the R1 verdict cannot disagree by construction.
- This keeps the §4.2 table honest: `(Universal, Universal)` across different carriers is
  false; an enumeration of symbols never contains an enumeration of integers even if the
  member nodes happen to be structurally equal.

### 4.5 Relationship to predicate refinements (`v2.std.refinement`)

One sentence each, to prevent dual-authority drift (P2):

- `Validation<B>` answers "*is this one value a member?*" — discharged at constructor
  boundaries, runtime, fail-closed. Unchanged.
- `ValueSet` answers "*is every member of A a member of B?*" — compile time, for coercion.
- A refinement **opts into derived coercion** by carrying a `ValueSet` description alongside
  its validation (the existing int refinements — `PositiveInt`, `NonNegativeInt`,
  `refinement.dag:194+` — get `IntegerInterval` descriptions mechanically; that is the
  fold-in's first breadth payoff).
- Where both exist, the description is the **coercion** authority and the validation is the
  **construction** authority; they describe the same set, and the slice includes one claim
  asserting agreement on a sampled boundary value (cheap coherence ratchet, not a proof).

## 5. What does *not* change

- `find_witness` / `find_witness_derives`: untouched. Same fold, same closed candidate set,
  same four preservation-rule symbols, same rejection-priority logic. (The "one fold
  parameterized by predicate" collapse is design Q2 of the dep graph — orthogonal lane.)
- `CoercionMismatchKind`: untouched in wave 1. Inexpressible-description refusals surface as
  R1 simply not holding (no candidate / `WouldLoseInformation` via R2 where coarsening is
  proven). Whether "inexpressible" deserves its own located mismatch variant is Q-V2 —
  escalate, the taxonomy is closed and load-bearing.

## 6. Consumers and minimal slice (E-10 / seesaw)

- **Consumer (exists, executing):** R1/R2 in `refinement_widening_predicate.dag`, exercised
  by the #4585 claim corpus — the swap from `integer_value_set_contains` to
  `value_set_contains` happens in the same PR as the new module, with the existing integer
  claims green as the regression floor (the integer fold-in must be behavior-preserving).
- **Minimal slice** (exercises the committed risk — kind dispatch + recursion + anchors —
  not a toy):
  1. `v2.std.value_set` with kinds `Empty | Universal | IntegerInterval | Enumeration |
     Product` (TaggedUnion may ride wave 2 — it adds breadth, not new risk shape);
  2. integer fold-in + R1/R2 swap + delete `integer_value_set.dag` (the marker's dissolution,
     P5 receipt in the same change);
  3. `TestClaim`s under `src/v2/test/claim/value_set/`:
     - **green**: enumeration widening (`{a,b} ⊆ {a,b,c}`) derives with a witness — the first
       non-integer derived coercion, by execution;
     - **green**: product pointwise widening over two fields;
     - **red (coarsening)**: `{a,b,c} → {a,b}` refuses (`WouldLoseInformation` path);
     - **red (wrong-core)**: anchor mismatch refuses despite structurally-identical members;
     - **red (the strictness fix)**: interval `[1,3]` vs enumeration `{1,2,3}` — semantically
       equal, so *not* a strict widening; the claim is red under the old syntactic `!=` rule's
       behavior and green only with semantic strictness. This is the discriminating
       red-when-wrong case (reviewer's three questions).
- Follow-on, consumer-triggered: int-refinement descriptions for `refinement.dag` carriers,
  TaggedUnion, then the deferred kinds (§4.1) each with its own claim.

## 7. Dissolution receipts (P5)

- `integer_value_set.dag` deleted; its gated coproduct marker (bound task
  `node://adhoc-248e4e6c-3da`) closes — the lattice *is* the named arrival.
- The same bound task's `model_core` axis-role-classifier interim
  (`model_core.dag:46-58`) gets its dissolution input: axis-role predicates derive from the
  closed fact-axis coproduct that anchors now consume.
- Forbidden, going forward: any new per-pair containment special case outside the closed
  table (that is the per-pair hard-code this design exists to prevent), and any
  predicate-evaluation path inside `value_set_contains`.

## 8. Open questions — escalate, don't improvise

- **Q-V1 — float carrier class.** Needs the float fact-bundle to fix total-order semantics
  (NaN, -0.0) first; until then float-anchored descriptions are inexpressible (refuse). Do
  not improvise an IEEE order inside the lattice.
- **Q-V2 — "inexpressible" as a located mismatch. RESOLVED (operator 2026-06-09: prefer
  fewer variants for now).** No new `CoercionMismatchKind` variant; inexpressible
  descriptions surface through the existing refusal reasons. Revisit only when a consumer
  demonstrably needs the located "why" at the boundary — with that need as the receipt, not
  preemptively.
- **Q-V3 — canonical forms.** Should descriptions normalize (e.g. enumeration sorted, single
  member intervals collapse to enumerations) so syntactic equality approximates semantic
  equality? Recommended **no** for wave 1: semantic operations only, normalization is an
  optimization with its own proof burden.
- **Q-V4 — meet/join.** The lattice's meet/join (intersection/union approximations) have no
  consumer yet — branch-merging of refinements would be the first. Declare the partial order
  now; land `BoundedLattice<ValueSet>` inhabitance only when that consumer exists (E-10).

## 9. Non-goals

- No predicate implication, no SMT, no proof search (P4: decision procedure with a verdict).
- No per-target or per-pair containment hard-codes (the N² that the thesis collapses).
- No change to the find_witness fold shape, rule vocabulary, or candidate-set closedness
  invariant (`coercion_property_closed_candidate_set` stands untouched).
