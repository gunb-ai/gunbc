---
status: Mgr canvas (substrate-shape question for Director ratification; surfaced per feedback_substrate_shape_belongs_in_mgr_canvas after PM msg_4fd650b7 formal canvas-trigger relay of Director ratification msg_ad5e934d 2026-05-13)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #105 `symbolic_cost_textbook_coverage_landed` (added via PM PR #2824)
ratification anchor: PM msg_4fd650b7 relaying Director msg_ad5e934d — Path A Tier 1 RATIFIED + 5 sub-canvas questions Q1-Q5 routed
operator framing: "current list looks very slim ... we need to land this all in R3 please" (2026-05-13)
authority docs:
  - `src/v3/std/algebra.dag:190-197` — current 7-variant `SymbolicCost`
  - `src/v3/std/algebra.dag:69-72` — `STOP SIGNAL: wanting an eighth variant` (will reset)
  - `docs/design-symbolic-cost-algebra.md` — current algebra
  - `dsl/std/algebra.dag:268-286` — existing `OrderedRing<T>` witness pattern
  - `dsl/std/algebra.dag:294` — `Field<T>` carries `compare: fn(T, T) -> Ordering` (foundational order primitive; derived predicates lt/le/gt/ge live lens-local under Q1-α)
  - `dsl/std/rational.dag:26` — `type Rational = Field<FieldOfFractions<Int>>` (inherits `Field.compare` via Field-shape)
  - `feedback_groundedness_gates_lenses` (Tier 2 structural-extension caveat)
---

# Gate #105 — SymbolicCost Tier 1 carrier-extension canvas

## §0. Status

Director ratified Path A Tier 1 on 2026-05-13 (PM msg_4fd650b7 relaying msg_ad5e934d). Net 7 → 9 SymbolicCost variants; one promoted (PolynomialCost.degree to Rational). This canvas surfaces 5 sub-questions for Director ratification before worker brief authoring.

PR #2824 (PM) carries §1.8 row #105 authority anchor; landing pending. Worker brief authoring **gates on this canvas being Director-ratified AND PR #2824 landing**.

## §1. Ratified Tier 1 variant list (verbatim per Director msg_ad5e934d)

1. **PROMOTE**: `PolynomialCost { var: SizeVariable, degree: NonZeroRational }` — **signed Rational with carrier-level `where nonzero` refinement** (Director RATIFIED scope-extension msg_2c1bfb0e — sign-admission intent — AS REFINED BY msg_b80bcaa8: Practice-2 carrier-level exclusion of degree=0 to prevent parallel authority with ConstantCost; sign-admission preserved — refinement excludes ONLY 0, admits ±). Subsumes positive degrees: √n = n^(1/2), ∛n = n^(1/3), n^(2/3), non-integer 2.373/2.807, existing integer poly. Subsumes negative degrees: 1/n = n^(-1), 1/√n = n^(-1/2), 1/n² = n^(-2) (asymptotic-decay coverage per operator directive 2026-05-13). Arbitrary roots + inverse roots covered uniformly; degree=0 structurally unrepresentable per Practice 2.
2. **ADD**: `PolyLogCost { var: SizeVariable, exponent: PolyLogExponent }` for log² n, log^k n (PolyLogExponent = Rational > 1 refinement; supports log^7.5 n / AKS Tier-1 case per operator BLOCKING PR #2824:333)
3. **ADD**: `ExponentialCost { base: ExponentialBase, var: SizeVariable }` for 2^n, c^n with c ≥ 2 (ExponentialBase = Int ≥ 2 refinement)
4. **ADD**: `FactorialCost { var: SizeVariable }` for n!

Net `src/v3/std/algebra.dag:190-197` final variant count: **9** (was 7).

Tier 2 R4-DEFERRED per Director with named-consumer-trigger requirement:
- LogLogCost (vEB trees)
- InverseAckermannCost (α(n) union-find)
- IteratedLogCost (log* n)
- HyperExponentialCost (n^n super-exp)

**Structural-extension caveat** (Director-named): if this canvas surfaces a compositional mechanism for Tier 2 satisfying `feedback_groundedness_gates_lenses` + composing with Sum/Product algebra + carries consumer-evidence justification, accept that mechanism IN-R3 instead of named variants. Canvas §6 addresses.

## §2. State at HEAD (grep-verified 2026-05-13)

- `src/v3/std/algebra.dag:190-197`: current 7 variants (ConstantCost, LinearCost, PolynomialCost with `degree: DegreeAtLeastTwo`, ProductCost, SumCost, LogCost, UnknownCost)
- `src/v3/std/algebra.dag:69-72`: STOP SIGNAL "wanting an eighth variant"
- `dsl/std/algebra.dag:268-286`: `OrderedRing<T>` witness with `compare/lt/le/gt/ge` already defined
- `dsl/std/algebra.dag:287-295`: `Field<T>` — **CORRECTION per operator BLOCKING 2026-05-13 (canvas:48)**: Field ALREADY has `compare: fn(T, T) -> Ordering` (line 294). The earlier "defined WITHOUT order operations" claim was wrong. Field is missing the derived order predicates (lt/le/gt/ge/eq/ne) that `OrderedRing<T>` carries (lines 268-286), but the foundational `compare` operator IS already on Field.
- `dsl/std/rational.dag:26`: `type Rational = Field<FieldOfFractions<Int>>`. Per above correction: Rational already carries `compare` via Field — order primitive is present. What's missing is the convenience-predicate set (lt/le/gt/ge/eq/ne). This invalidates the original Q1 premise; see §3 revised candidate set below.
- `dsl/std/computation.dag:8`: "algebra.dag says 'Int inhabits OrderedRing'" — Int is ordered, but Rational is not

## §3. Q1 — Rational dominance lattice (Director RATIFIED Q1-α per msg_676ad4e7 2026-05-13; supersedes msg_d86a5987 Q1-c)

**Premise correction**: Earlier canvas authoring claimed Rational "carries no order witness." This was wrong — `Field<T>` at `dsl/std/algebra.dag:294` already has `compare: fn(T, T) -> Ordering`. Rational therefore already carries the foundational order primitive. What's missing is the **derived order predicates** (lt/le/gt/ge/eq/ne) that `OrderedRing<T>` carries on top of `compare`.

The original Q1-a/Q1-b/Q1-c framing — and the prior Director-ratified Q1-c disposition — was based on the stale premise. Introducing `OrderedField<T>` now would create **parallel order authority** with the existing `Field.compare` field. Anti-pattern.

### REVISED Candidate Q1-α — Use `Field.compare` directly + derive helpers as free functions

Scope:
- Cost-lens fold uses `Rational.compare` directly via existing `Field<FieldOfFractions<Int>>.compare`
- Where lt/le/gt/ge convenience predicates are needed in cost-lens body, define them as free functions on `Rational` in cost-lens module (e.g., `rational_lt(a, b) = compare(a, b) == Less`)
- **No `OrderedField<T>` introduction**; no `Rational` re-declaration; no modifications to `dsl/std/algebra.dag` or `dsl/std/rational.dag`

Pros:
- Zero new substrate; uses existing `Field.compare`
- Cost-lens-local convenience helpers; no foundational algebra touched
- No parallel order authority

Cons:
- lt/le/gt/ge live as free functions, not on the carrier (mild Cost-of-Change-2 for future predicate sites if free functions need refactor)

### REVISED Candidate Q1-β — Extend `Field<T>` with derived order predicates (in-place)

Scope:
- Add `lt: fn(T, T) -> Bool`, `le`, `gt`, `ge`, `eq`, `ne` to `Field<T>` at `dsl/std/algebra.dag:287` — mirroring the OrderedRing record's predicate set
- No new type; `Field<T>` becomes the single authority for ordered-field operations
- Refit Rational stays at `Field<FieldOfFractions<Int>>`

Pros:
- Strict in-place extension of foundational carrier; no parallel-rep
- Single authority — `Field.compare` is foundational; derived predicates compose on it
- Cost-lens fold uses `Rational.lt`, `Rational.le`, etc. directly (Cost-of-Change-1 for future predicate sites)

Cons:
- Touches foundational `Field<T>` shape; affects all `Field<T>` consumers (witness record gets 6 new fields)
- Migration: existing `Field<T>` witness realizations must populate the new predicate fields

### REVISED Candidate Q1-γ — `OrderedField<T>` strict-superset (former Q1-c REVISED with explicit reconciliation)

Scope:
- Add `type OrderedField<T>` extending Field with the missing 6 predicates, EXPLICITLY reconciled: `Field.compare` is the foundational primitive; OrderedField inherits compare from its Field-shaped sub-record and adds derived predicates
- This requires DSL support for sub-record inheritance / type-level inclusion — check at HEAD whether the grammar supports this

Pros:
- Cleanest Practice-4 layering; OrderedField is structurally Field-plus-predicates
- Field consumers unchanged; OrderedField consumers gain full predicate set

Cons:
- DSL inheritance grammar may not exist at HEAD (worker brief must grep-verify before authoring)
- Two carriers (Field + OrderedField); the prior parallel-authority concern returns unless inheritance is structural, not parallel

**Revised Mgr recommendation**: **Q1-α** (use `Field.compare` + cost-lens-local free functions). Reasoning:
- Zero new substrate; zero foundational-algebra-touch
- `feedback_strict_mirror_vs_novel_substrate_fact` does NOT apply here — there's no need for a new witness shape since Field already carries the foundational primitive
- `feedback_no_short_term_solutions` is also irrelevant — this is the canonical use-existing-substrate pattern, not a workaround
- Q1-β is acceptable if Director prefers carrier-uniform predicate set, but the migration scope is broader
- Q1-γ requires DSL-grammar prerequisite check; defer unless Q1-α is rejected

**Director ratified Q1-α** (msg_676ad4e7 2026-05-13, explicit retraction of msg_d86a5987 Q1-c). Director acknowledged discipline-miss (failure to grep `dsl/std/algebra.dag` for existing Field carrier shape before ratifying); will fold incident as Case 3 in `feedback_grep_substrate_before_naming_ratification`.

Q1-β REJECTED: doubles Field carrier surface (7→13) for predicates derivable from Ordering pattern-match.

Q1-γ REJECTED: DSL grammar inheritance/superset typing R4-scope at earliest.

## §4. Q2 — Linear-vs-Polynomial split reconciliation

Current shape: `LinearCost(SizeVariable)` is a distinct variant; `PolynomialCost { degree: DegreeAtLeastTwo }` excludes degree=1. After PolynomialCost.degree → Rational, the structural separation question:

### Candidate Q2-X — Keep Linear separate

```dag
| LinearCost(SizeVariable)
| PolynomialCost { var: SizeVariable, degree: Rational where degree ≠ 1 }
```

Pros:
- Preserves all existing LinearCost-consumer paths (no migration)
- Linear is a meaningful named case; reviewers can grep for it

Cons:
- `degree ≠ 1` refinement is awkward; structural separation no longer matches algebraic reality (n = n^1 is exactly poly degree=1)
- Algebra rules need a branch for Linear vs PolynomialCost in sum/product (e.g., `LinearCost · LinearCost = PolynomialCost(degree=2)` introduces a cross-variant rule)

### Candidate Q2-Y — Collapse Linear into Polynomial(degree=1)

```dag
| PolynomialCost { var: SizeVariable, degree: NonZeroRational }
```
(LinearCost removed; `LinearCost(v)` ≡ `PolynomialCost { var: v, degree: 1 }`. Per Director scope-extension msg_2c1bfb0e + msg_b80bcaa8: **no positivity / `gt_zero` refinement** — signed Rational admits negative degrees for asymptotic-decay coverage; **carrier-level `where nonzero` refinement IS present** to exclude degree=0 collision with ConstantCost per Practice-2 (Director Option B ratification msg_b80bcaa8).)

Pros:
- Single uniform variant for all positive-degree polynomial bounds
- Sum/product algebra closes uniformly: `PolyCost(d1) · PolyCost(d2) = PolyCost(d1+d2)` no Linear special case
- 7 → 9 net (per §1 ratified scope: PROMOTE PolyCost.degree + ADD 3 new variants + REMOVE LinearCost = +3 net new variants over the existing 7; LinearCost-absorption aligns with §P5 Progress Is Dissolution). PolynomialCost.degree promotion is not a new variant — see §4 closing line for the variant-count reconciliation.

Cons:
- **Migration**: all LinearCost-consumer paths must rewrite to PolynomialCost(degree=1)
- Reviewers lose the named "Linear" landmark in cost analysis output (cosmetic)

**Mgr recommendation**: Q2-Y. The `degree ≠ 1` refinement (Q2-X) is exactly the kind of structural fudge §P5 progress-is-dissolution discipline rejects. LinearCost as a separate variant in a Rational-degree world is a pre-Rational vestige. Migration scope is bounded (cost-lens consumers) and per `feedback_load_bearing_ratchet_preservation` the dissolution-receipt is the standard pattern.

Net Tier 1 final variant count under Q2-Y: **9**, not 10 (PolynomialCost.degree promotion is not a new variant). Reviewer-ratchet adjusts accordingly.

## §5. Q3 — Sum/Product algebra interaction rules

Existing rules (from `docs/design-symbolic-cost-algebra.md` + algebra.dag fold operators):
- `PolyCost(d1) + PolyCost(d2) = PolyCost(max(d1, d2))` (sum takes dominant)
- `PolyCost(d1) · PolyCost(d2) = PolyCost(d1 + d2)` (product adds degrees)
- `LogCost(v) + ConstantCost(c) = LogCost(v)` (log dominates constant)

**New rules required** (per Director Q3):

| Operation | Result | Justification |
|---|---|---|
| `PolyCost(d) · LogCost(v)` | `PolyLogCost { var: v, exponent: 1 }`? OR composite ProductCost? | Canvas question (see §5.1) |
| `PolyLogCost(v, k1) · PolyLogCost(v, k2)` | `PolyLogCost(v, k1+k2)` | log^a · log^b = log^(a+b) |
| `PolyCost(d) + ExpCost(c, v)` | `ExpCost(c, v)` | exp dominates poly |
| `PolyCost(d) · ExpCost(c, v)` | `ProductCost([PolyCost(d), ExpCost(c, v)])` | composite, NOT absorbed — n^d · c^n is NOT O(c^n) (operator BLOCKING worker:140); additive absorption above is sound, multiplicative is NOT |
| `ExpCost(c1, v) · ExpCost(c2, v)` | `ExpCost(c1·c2, v)` | c1^v · c2^v = (c1·c2)^v |
| `ExpCost(c1, v) + ExpCost(c2, v)` (c1 < c2) | `ExpCost(c2, v)` | dominant base |
| `FactorialCost(v) + ConstantCost / PolyCost(_,v) / LogCost(v) / PolyLogCost(v,_) / ExpCost(_,v)` | `FactorialCost(v)` | factorial dominates same-variable Tier-1 below |
| `FactorialCost(v) + FactorialCost(w)` (v ≠ w) | `SumCost([FactorialCost(v), FactorialCost(w)])` | cross-variable preserved as composite (no inter-variable dominance) |
| `FactorialCost(v) + UnknownCost(reason)` | `SumCost([FactorialCost(v), UnknownCost(reason)])` | UnknownCost is conservative-top; never absorbed |
| `FactorialCost(v) + SumCost([…]) / ProductCost([…])` | distribute then re-fold per §6 | composite-fold delegates to algebra rules |
| `FactorialCost(v) · PolyCost(d)` | `ProductCost([FactorialCost(v), PolyCost(d)])` | composite, NOT absorbed (n! · n^d not O(n!)) |
| `FactorialCost(v) · ExpCost(c, v)` | `ProductCost([FactorialCost(v), ExpCost(c, v)])` | composite, NOT absorbed (n! · c^n not O(n!)) |
| `FactorialCost(v) · FactorialCost(v)` | `FactorialCost(v)`? OR `UnknownCost("v! · v! exceeds Tier 1")` | Canvas question (see §5.2) |

### §5.1 — PolyCost · LogCost normalization

Two candidate shapes:
- (a) `PolyCost(d) · LogCost(v) = PolyLogCost { var: v, exponent: 1 }` if d=0, but d=0 means ConstantCost not PolyCost; if d=1 then it's `v · log(v)` which is canonically n log n
- (b) Keep as composite `ProductCost([PolyCost, LogCost])`; PolyLogCost only constructed from explicit log² n etc.

Mgr recommendation: (b). PolyLogCost is for log^k n only (single variable, rational-exponent log per Q1-α + PolyLogExponent refinement). The n log n shape is `ProductCost([PolynomialCost { var: n, degree: 1 }, LogCost(n)])` — representable via PolynomialCost(degree=1) post-Q2-Y collapse (LinearCost dissolved); the algebra fold via ordered-dominance correctly identifies it. Avoiding (a) prevents semantic-collision between "poly times log" and "polylog".

### §5.2 — FactorialCost · FactorialCost

Two candidate shapes:
- (a) `FactorialCost(v) · FactorialCost(v) = FactorialCost(v)` (factorial absorbs)
- (b) `UnknownCost("(v!)² exceeds Tier 1 — pending R4 named-variant canvas")` per Tier-2-deferral receipt

Mgr recommendation: (b). `(n!)²` is genuinely outside Tier 1; it's super-factorial / hyperfactorial territory. Per Director anti-pattern #5: "UnknownCost used for textbook-Tier-1-coverable bounds post-promotion (STOP-SIGNAL violation)" — (n!)² is NOT Tier-1-coverable, so UnknownCost("...pending R4...") is the correct disposition.

### §5.3 — Normalization (rational-degree polynomial)

- `PolyCost(d1) · PolyCost(d2) = PolyCost(d1 + d2)` — uses `Field.add` on Rational (Q1-α; Rational inherits Field's Ring-shape add)
- `PolyCost(1/2) · PolyCost(1/2) = PolyCost(1)` — and per Q2-Y, this is PolyCost(degree=1), the absorbed Linear
- `PolyCost(d1) + PolyCost(d2) = PolyCost(max(d1, d2))` — uses `Field.compare` on Rational via cost-lens-local `rational_max` helper (Q1-α; NO OrderedField)

## §6. Q4 — STOP-SIGNAL update

Current `src/v3/std/algebra.dag:69-72`:
> STOP SIGNAL: wanting an eighth variant. Pause and escalate rather than extending; the thesis claim is that seven covers the asymptotic surface, and any new variant should carry its own dissolution receipt.

**Post-extension proposed text** (Mgr recommendation):
> STOP SIGNAL: wanting a 10th variant (or 11th if Tier-2 IteratedLog/LogLog/InverseAckermann/HyperExp surface). Pause and escalate. Tier-1 textbook coverage (gate #105 carrier-extension 2026-05-13) lands 9 variants covering ConstantCost / PolynomialCost { degree: NonZeroRational } (signed per Q6; nonzero per msg_b80bcaa8) / PolyLogCost { exponent: PolyLogExponent } / LogCost / ProductCost / SumCost / ExponentialCost { base: ExponentialBase } / FactorialCost / UnknownCost — sufficient for the asymptotic surface that R3-load-bearing lenses reason about. Tier-2 (LogLog / InverseAckermann / IteratedLog / HyperExp) is R4-DEFERRED per Director ratification msg_d86a5987 (§8 disposition); new variants in R4 require consumer-evidence-justified canvas. UnknownCost("reason: ...") remains algebra-top, but reviewer-tier STOP-SIGNAL fires if a Tier-1-coverable bound is collapsed to Unknown — that is anti-pattern #5 per gate #105.

**Type-level refinement carriers (CORRECTED per PM msg_a52ed981)** — refinement-mechanism `type X = Y where predicate` is ALREADY RATIFIED at HEAD per gunbc#828 issuecomment-4390333451 Path 3 + Director Option 2 (gunbc#828 issuecomment-4390199218). Precedent: `dsl/std/integer.dag:181` (`PositiveInt = Nat where gt_zero`). KNOWN_PREDICATES registry at `src/v3/compiler/src/lower.rs:798-862`: `range / non_empty / brand / gt_zero / unicode_scalar`. NO fresh-records / inductive-sum carriers — refinement over canonical carrier is the canonical path:

- ~~`PositiveRational = Rational where gt_zero`~~ — **DROPPED per Director msg_2c1bfb0e scope-extension**: PolynomialCost.degree is plain `Rational` (signed; admits negative-degree decay). No gt_zero allowed_carriers extension needed for PolynomialCost.
- `ExponentialBase = Int where range(min: 2)` — IMMEDIATELY available via `range` predicate (allowed_carriers includes Int)
- `PolyLogExponent = Rational where gt_one` — REQUIRES NEW `gt_one` predicate (allowed_carriers: Rational + Int; mirrors `gt_zero` shape; atomic with carrier landing per Phase A)
- `NonZeroRational = Rational where nonzero` — REQUIRES NEW `nonzero` predicate (allowed_carriers: Rational; arg_shape: Bare). **Named alias** per HEAD parser constraint (codex BLOCKING worker:167): `where` refinements only attach to type aliases / parameters at HEAD (precedent `dsl/std/integer.dag:181 type PositiveInt = Nat where gt_zero`), NOT inline in struct field types. Used as `PolynomialCost.degree: NonZeroRational` per Director Option B msg_b80bcaa8.
- `PositiveInt = Nat where gt_zero` — ALREADY EXISTS at `dsl/std/integer.dag:181`; worker reuses

These refinements make `exponent ≤ 1` (PolyLogCost), `base ≤ 1` (ExponentialCost), and `degree = 0` (PolynomialCost via NonZeroRational) **structurally unrepresentable** at the carrier level via the ratified refinement mechanism — Practice 2 + Practice 6 satisfied; INVARIANTS P1 (single authority) preserved. PolynomialCost.degree has **no positivity refinement** (signed Rational admits asymptotic decay / negative degrees per Director msg_2c1bfb0e sign-admission), but DOES carry the named `NonZeroRational` alias (msg_b80bcaa8 Practice-2 zero-exclusion to prevent ConstantCost collision). Sign-admission preserved; zero-exclusion enforced.

## §6.1 — Q6 Asymptotic-dominance ordering with signed degrees (Director RATIFIED msg_2c1bfb0e)

Signed-Rational degrees require explicit dominance rules across the sign boundary. Director-verbatim conjecture (RATIFIED):

> - For positive degree a, b > 0: `n^a > n^b` iff `a > b` (existing rule).
> - Between positive + negative: any positive-degree term dominates any negative-degree term (`n^a > n^(-b)` for a, b > 0; positive grows → ∞, negative decays → 0).
> - Between negative + constant: `1 > n^(-a)` for any a > 0 (constant dominates decay-to-zero in asymptotic-magnitude lattice).
> - Between two negatives: `n^(-a) > n^(-b)` iff `a < b` (least-negative dominates; 1/n > 1/n²).
>
> **Conjecture**: the dominance rule is "compare degrees with reverse-sign-convention" — asymptotic dominance ≡ algebraic ordering of degrees, but the carrier-to-asymptotic-direction mapping handles sign.

**Authority**: Q1-α already provides `Field.compare: fn(Rational, Rational) -> Ordering` on the signed-rational carrier. Worker encodes the dominance rule as a derived ordering on `(SizeVariable, Rational)` pairs using `Field.compare` for the magnitude comparison — no new ordering authority introduced.

**Zero-degree exclusion via carrier-level refinement** (Director RATIFIED Option B per msg_b80bcaa8, supersedes prior canonicalize-fold approach): plain signed `Rational` would admit `degree = 0` which structurally collides with `ConstantCost` (n^0 ≡ 1) — Director rationale (verbatim): "degree=0 is P1 violation, not just Practice-4 normalization … type enforcement > API enforcement; Practice-2 carrier refinement = type-tier, Practice-4 canonicalize-fold = API-tier. Type wins." Resolution: **carrier is `Rational where nonzero`** — exclusion of degree=0 at carrier level via new `nonzero` predicate added to KNOWN_PREDICATES (Phase A; analogous to `gt_one` addition; orthogonal to sign — admits ± rationals). Practice 2 illegal-states-unrepresentable satisfied at type tier; no canonicalize-fold needed for this collision.

**Practice-2 vs Practice-4 disambiguation rule** (Director-distilled msg_b80bcaa8, NEW load-bearing discipline):
> Same-variant redundancy (e.g., LinearCost vs PolyCost(d=1)) → Practice-4 collapse (Q2-Y precedent). Cross-variant redundancy (e.g., PolyCost(d=0) vs ConstantCost) → Practice-2 carrier refinement. Type-level state-space tightening beats API-level normalization when redundant state crosses variant boundaries.

## §6.2 — Q7 SymbolicCost preserves full expression; Big-O is a derived operation (Director RATIFIED msg_2c1bfb0e)

Director-verbatim disposition (RATIFIED):

> SymbolicCost preserves the full expression ("symbolic" name commits to symbolic-representation, NOT pre-applied asymptotic-simplification). Sum-normalization rule: keep all terms in canonical sorted form (by dominance), DON'T drop sub-dominant terms during canonical-form construction.
>
> Big-O projection is a **derived operation** (separate function `dominant_term(SymbolicCost) -> SymbolicCost` or `asymptotic_class(SymbolicCost) -> ComplexityClass`); SymbolicCost itself is exact.

**Rationale**: per `feedback_compositional_not_templating` — preserve info structurally; consumer projects as needed. Asymptotic-simplification at canonical-form construction would destroy info.

**Implication for §5 algebra fold rules**: same-variable sums (e.g., `n + log(n) + 1/n`) canonicalize to `SumCost([PolyCost(n, 1), LogCost(n), PolyCost(n, -1)])` (dominance-sorted), NOT to the dominant term alone. The `+ ExpCost(c, v)` → `ExpCost(c, v)` style dominance rules in §5 are **derived-operation rules**, not canonical-form rules — they apply when computing `dominant_term`, not when constructing SymbolicCost. Worker brief Phase D encodes both: canonical-form preservation + dominant_term derivation.

Variant count: **9** post-Q2-Y (Director-ratified; PolynomialCost.degree promotion is not a new variant), or **10** if Q2-X is ratified.

## §7. Q5 — Carrier-shape canvas before worker dispatch — THIS DOC

This canvas IS the Q5 carrier-shape canvas. On Director ratification of Q1-Q4 + this canvas, worker dispatch proceeds with brief authored per ratified shape.

## §8. Tier-2 structural-extension caveat (Director-named)

Director: "if your canvas surfaces a compositional mechanism for Tier 2 that satisfies `feedback_groundedness_gates_lenses` + composes with Sum/Product algebra + carries consumer-evidence justification, accept that mechanism IN-R3 instead of named variants for Tier 2."

### Candidate compositional mechanism — `IteratedAlgebra<F, T>`

Hypothesis: most Tier-2 bounds are **iterates** of Tier-1 functions:
- LogLogCost = `IteratedAlgebra<Log, n>` (log applied twice)
- IteratedLogCost = `IteratedAlgebra<Log, n>` with iteration-count = log*(n)
- InverseAckermannCost = inverse of `Ackermann` iterate — doesn't fit cleanly
- HyperExpCost = `IteratedAlgebra<Exp, n>`

Sum/Product composition under iterate: complex — LogLog · LogLog = LogLog², which is itself a polylog of a logarithm. Composition discipline rapidly degrades.

**Mgr finding**: a uniform compositional mechanism for ALL Tier-2 cases is **not surfaced by this canvas**. InverseAckermann in particular doesn't fit the iterate pattern. Recommendation: **defer Tier 2 to R4 per Director default**; do NOT accept IteratedAlgebra in this canvas as a structural-extension shortcut. The consumer-evidence triggers (vEB trees, union-find, log*-bounded data structures) can drive named-variant canvases in R4.

## §9. Practice 4 (coproduct dissolution) classification

For each Tier-1 addition under §1:

| Addition | Practice 4 classification |
|---|---|
| PROMOTE PolynomialCost.degree to Rational | 🟢 GREEN — refines structural payload; no new sum-type arm; dissolution-trigger if Rational lands order |
| ADD PolyLogCost | 🟢 GREEN — new variant for distinct asymptotic class (log^k n); consumer evidence: polylog-time algorithms (Strassen, FFT preliminaries) |
| ADD ExponentialCost | 🟢 GREEN — distinct asymptotic class; consumer evidence: brute-force search, exponential-time hypotheses |
| ADD FactorialCost | 🟢 GREEN — distinct asymptotic class; consumer evidence: permutation enumeration, brute-force matching |
| REMOVE LinearCost (under Q2-Y) | 🟢 P5 dissolution — absorbed into PolynomialCost(degree=1); structural fact unchanged |

No 🔴 RED introductions. Anti-pattern #2 (Director-enumerated): "Path B revival (RootCost as separate variant — Practice-4 RED)" — explicitly NOT done; roots are PolynomialCost(degree=1/2 etc.).

## §10. Anti-patterns (7 Director-enumerated + 5 Mgr-derived; 12 total)

### Director-enumerated

1. Any Tier 2 variant named without consumer-evidence (premature variants) — §8 disposition: not introducing
2. Any Path B revival (RootCost as separate variant — Practice-4 RED) — §9: explicitly not done
3. Linear-Polynomial split decision authored without canvas (substrate-shape question goes through Mgr) — this canvas IS the §4 Q2 disposition
4. Dominance lattice fudging via string-tagged Rational (use real ordered-witness) — §3 Q1-α addresses via existing Field.compare
5. UnknownCost used for textbook-Tier-1-coverable bounds post-promotion (STOP-SIGNAL violation) — §6 STOP-SIGNAL text encodes
6. **Director-ratified msg_676ad4e7**: Introducing parallel ordered-algebraic-structure carriers (`Ordered<X>`) when the underlying carrier already provides `compare: fn(T, T) -> Ordering`. Lens-local predicate derivation from Ordering pattern-match is the canonical path. — §3 Q1-α addresses
7. **NEW (Director ratified per operator BLOCKING PR #2824:333)**: Tier-1 variant constructed with raw Int/Rational exponent/base that admits illegal collapse values (exponent=0/1 for PolyLogCost; base=0/1 for ExponentialCost) bypassing the refinement type. PolyLogExponent + ExponentialBase are required at the type level (Practice 2/6; illegal-states-unrepresentable). **PolynomialCost.degree is excluded** from this anti-pattern — Director msg_2c1bfb0e scope-extension intentionally admits signed Rational degrees for asymptotic-decay coverage (Q6).

### Mgr-derived (encoded for worker review)

8. Multiplicative absorption rules where one variant absorbs another (`X · Y = X`) when X is asymptotically larger than Y additively — asymptotic absorption is sound for SUM but NOT PRODUCT (n^d · c^n is NOT O(c^n)); cross-class products must be `ProductCost` composite. §5 algebra rules table addresses (operator BLOCKING worker:140).
9. **PM-grep-corrected per msg_a52ed981 + codex 014544f4 finding #1**: Parallel rational-number carriers (fresh records like `{ num: PositiveInt; denom: PositiveInt }`, inductive sums, or any carrier shape OTHER than refinement) when the canonical refinement-mechanism (`type X = Y where predicate`) is RATIFIED at HEAD per gunbc#828 issuecomment-4390333451 Path 3 + Director Option 2. Refinement over canonical `Rational = Field<FieldOfFractions<Int>>` is the canonical path; precedent `PositiveInt = Nat where gt_zero` at `dsl/std/integer.dag:181`. Anti-pattern fires on ANY fresh-carrier shape when refinement is available.
10. `LinearCost`-consumer paths preserved alongside `PolynomialCost(degree=1)` (Q2-Y atomic-migration; bridge variants violate §P5)
11. **Director-added msg_2c1bfb0e**: Introducing parallel `InverseCost(SymbolicCost)` / `ReciprocalCost` / `DecayCost` variants when carrier-extension via signed `degree: Rational` is structurally clean. Same Q1-α / Q1-c lesson class — don't bridge-wrap when carrier-extension dissolves the question (`feedback_dissolve_bridges` + `feedback_no_metadata_markers`).
12. **Director-added msg_b80bcaa8**: Introducing canonicalize-fold rules for **cross-variant redundancy** when carrier-level refinement (`where <predicate>`) is structurally available. Practice-2 type-tier exclusion beats Practice-4 API-tier normalization when redundant state crosses variant boundaries (PolyCost(d=0) ≡ ConstantCost(1) → use `where nonzero`, not canonicalize-fold). Same-variant collapse (Q2-Y LinearCost ≡ PolyCost(d=1)) is the appropriate Practice-4 lane; cross-variant redundancy must be carrier-refined.

## §11. Cost-of-change accounting

Per `INVARIANTS.md` "Cost of Change":

| State | Files to edit to add one new asymptotic-bound consumer |
|---|---|
| Pre-canvas (today) | ≥3 (variant if Tier-2-coverable; algebra rules; consumer site) |
| Post-canvas (9-variant Q2-Y) | 1 (consumer constructs the appropriate Tier-1 variant directly) |

For exotic (Tier-2) bounds: still requires UnknownCost("reason") at consumer site — that's the R4-deferral receipt.

## §12. Ratified dispositions (audit trail; all Q1-Q7 Director-ratified)

- **Q1**: **RATIFIED Q1-α** (msg_676ad4e7, supersedes msg_d86a5987 Q1-c) — use existing `Field.compare`; NO `OrderedField` introduction; cost-lens-local rational_lt/le/gt/ge/eq/ne free functions
- **Q2**: **RATIFIED Q2-Y** (collapse LinearCost into PolynomialCost(degree=1))
- **Q3**: **RATIFIED** §5 10-rule algebra interaction table; §5.1 b composite; §5.2 b UnknownCost (for (n!)²)
- **Q4**: **RATIFIED** §6 STOP-SIGNAL text re-reset at 10th variant (9 ratified + 1 trigger)
- **Q5**: **RATIFIED** this canvas (carrier-shape canvas before worker dispatch)
- **§8 Tier-2 mechanism**: **RATIFIED defer to R4**; IteratedAlgebra rejected per §8 analysis
- **Q6** (Director msg_2c1bfb0e): **RATIFIED** signed-Rational `PolynomialCost.degree` (drop `where gt_zero` refinement); asymptotic-dominance rule "compare degrees with reverse-sign-convention" using existing `Field.compare` authority. Practice 4 🟢 GREEN — no new sum-types; carrier extension.
- **Q7** (Director msg_2c1bfb0e): **RATIFIED** SymbolicCost preserves full expression; canonical-form is dominance-sorted SumCost preserving all terms; Big-O is derived operation via `dominant_term` / `asymptotic_class` projection functions.

## §13. Reference

- §1.8 row #105 (PM PR #2824 pending) — authority anchor
- `src/v3/std/algebra.dag:190-197` — current 7-variant SymbolicCost
- `src/v3/std/algebra.dag:69-72` — current STOP SIGNAL
- `dsl/std/algebra.dag:268-286` — OrderedRing<T> precedent
- `dsl/std/algebra.dag:294` — Field<T> carries `compare: fn(T, T) -> Ordering`
- `dsl/std/rational.dag:26` — Rational = Field<FieldOfFractions<Int>>
- `docs/design-symbolic-cost-algebra.md` — current algebra
- Director ratification: PM msg_4fd650b7 / Director msg_ad5e934d
- `feedback_groundedness_gates_lenses` — Tier-2 structural-extension caveat
- `feedback_strict_mirror_vs_novel_substrate_fact` — Q1-α applies (strict-mirror via existing Field.compare)
- `feedback_load_bearing_ratchet_preservation` — Q2-Y migration discipline

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
