---
status: Mgr canvas (substrate-shape question for Director ratification; surfaced per feedback_substrate_shape_belongs_in_mgr_canvas after PM msg_4fd650b7 formal canvas-trigger relay of Director ratification msg_ad5e934d 2026-05-13)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #105 `symbolic_cost_textbook_coverage_landed` (added via PM PR #2824)
ratification anchor: PM msg_4fd650b7 relaying Director msg_ad5e934d — Path A Tier 1 RATIFIED + 5 sub-canvas questions Q1-Q5 routed
operator framing: "current list looks very slim ... we need to land this all in R3 please" (2026-05-13)
authority docs:
  - `src/v3/std/algebra.dag:190-197` — current 7-variant `SymbolicCost`
  - `src/v3/std/algebra.dag:60-72` — `STOP SIGNAL: wanting an eighth variant` (will reset)
  - `docs/design-symbolic-cost-algebra.md` — current algebra
  - `dsl/std/algebra.dag:268-286` — existing `OrderedRing<T>` witness pattern
  - `dsl/std/algebra.dag:287` — existing `Field<T>` (no order)
  - `dsl/std/rational.dag:26` — `type Rational = Field<FieldOfFractions<Int>>` (no order witness)
  - `feedback_groundedness_gates_lenses` (Tier 2 structural-extension caveat)
---

# Gate #105 — SymbolicCost Tier 1 carrier-extension canvas

## §0. Status

Director ratified Path A Tier 1 on 2026-05-13 (PM msg_4fd650b7 relaying msg_ad5e934d). Net 7 → 9 SymbolicCost variants; one promoted (PolynomialCost.degree to Rational). This canvas surfaces 5 sub-questions for Director ratification before worker brief authoring.

PR #2824 (PM) carries §1.8 row #105 authority anchor; landing pending. Worker brief authoring **gates on this canvas being Director-ratified AND PR #2824 landing**.

## §1. Ratified Tier 1 variant list (verbatim per Director msg_ad5e934d)

1. **PROMOTE**: `PolynomialCost { var: SizeVariable, degree: Rational }` (subsumes √n = n^(1/2), ∛n = n^(1/3), n^(2/3), and non-integer poly n^2.373 + n^2.807, plus existing integer poly)
2. **ADD**: `PolyLogCost { var: SizeVariable, exponent: Int }` for log² n, log^k n
3. **ADD**: `ExponentialCost { base: Int, var: SizeVariable }` for 2^n, c^n with c ≥ 2
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
- `src/v3/std/algebra.dag:60-72`: STOP SIGNAL "wanting an eighth variant"
- `dsl/std/algebra.dag:268-286`: `OrderedRing<T>` witness with `compare/lt/le/gt/ge` already defined
- `dsl/std/algebra.dag:287-295`: `Field<T>` — **CORRECTION per operator BLOCKING 2026-05-13 (canvas:48)**: Field ALREADY has `compare: fn(T, T) -> Ordering` (line 294). The earlier "defined WITHOUT order operations" claim was wrong. Field is missing the derived order predicates (lt/le/gt/ge/eq/ne) that `OrderedRing<T>` carries (lines 268-286), but the foundational `compare` operator IS already on Field.
- `dsl/std/rational.dag:26`: `type Rational = Field<FieldOfFractions<Int>>`. Per above correction: Rational already carries `compare` via Field — order primitive is present. What's missing is the convenience-predicate set (lt/le/gt/ge/eq/ne). This invalidates the original Q1 premise; see §3 revised candidate set below.
- `dsl/std/computation.dag:8`: "algebra.dag says 'Int inhabits OrderedRing'" — Int is ordered, but Rational is not

## §3. Q1 — Rational dominance lattice (substrate-shape question; REVISED per operator BLOCKING canvas:48)

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

**Re-ratification required**: Director's prior Q1-c ratification (PM msg_a055c38b relaying msg_d86a5987) was based on the stale premise. With premise corrected, Q1 needs re-disposition. Most likely lands on Q1-α (smallest scope, premise-corrected) but Director may prefer Q1-β for carrier uniformity.

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
| PolynomialCost { var: SizeVariable, degree: Rational where degree > 0 }
```
(LinearCost removed; `LinearCost(v)` ≡ `PolynomialCost { var: v, degree: 1 }`)

Pros:
- Single uniform variant for all positive-degree polynomial bounds
- Sum/product algebra closes uniformly: `PolyCost(d1) · PolyCost(d2) = PolyCost(d1+d2)` no Linear special case
- 11 → 10 net (Linear absorbed); aligns with §P5 Progress Is Dissolution

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
| `FactorialCost(v) + anything` | `FactorialCost(v)` | factorial dominates |
| `FactorialCost(v) · PolyCost(d)` | `ProductCost([FactorialCost(v), PolyCost(d)])` | composite, NOT absorbed (n! · n^d not O(n!)) |
| `FactorialCost(v) · ExpCost(c, v)` | `ProductCost([FactorialCost(v), ExpCost(c, v)])` | composite, NOT absorbed (n! · c^n not O(n!)) |
| `FactorialCost(v) · FactorialCost(v)` | `FactorialCost(v)`? OR `UnknownCost("v! · v! exceeds Tier 1")` | Canvas question (see §5.2) |

### §5.1 — PolyCost · LogCost normalization

Two candidate shapes:
- (a) `PolyCost(d) · LogCost(v) = PolyLogCost { var: v, exponent: 1 }` if d=0, but d=0 means ConstantCost not PolyCost; if d=1 then it's `v · log(v)` which is canonically n log n
- (b) Keep as composite `ProductCost([PolyCost, LogCost])`; PolyLogCost only constructed from explicit log² n etc.

Mgr recommendation: (b). PolyLogCost is for log^k n only (single variable, integer-exponent log). The n log n shape is `ProductCost([LinearCost(n), LogCost(n)])` — already representable, and the algebra fold via ordered-dominance correctly identifies it. Avoiding (a) prevents semantic-collision between "poly times log" and "polylog".

### §5.2 — FactorialCost · FactorialCost

Two candidate shapes:
- (a) `FactorialCost(v) · FactorialCost(v) = FactorialCost(v)` (factorial absorbs)
- (b) `UnknownCost("(v!)² exceeds Tier 1 — pending R4 named-variant canvas")` per Tier-2-deferral receipt

Mgr recommendation: (b). `(n!)²` is genuinely outside Tier 1; it's super-factorial / hyperfactorial territory. Per Director anti-pattern #5: "UnknownCost used for textbook-Tier-1-coverable bounds post-promotion (STOP-SIGNAL violation)" — (n!)² is NOT Tier-1-coverable, so UnknownCost("...pending R4...") is the correct disposition.

### §5.3 — Normalization (rational-degree polynomial)

- `PolyCost(d1) · PolyCost(d2) = PolyCost(d1 + d2)` — uses Q1-c OrderedField.add
- `PolyCost(1/2) · PolyCost(1/2) = PolyCost(1)` — and per Q2-Y, this is PolyCost(degree=1), the absorbed Linear
- `PolyCost(d1) + PolyCost(d2) = PolyCost(max(d1, d2))` — uses Q1-c OrderedField.compare

## §6. Q4 — STOP-SIGNAL update

Current `src/v3/std/algebra.dag:69-72`:
> STOP SIGNAL: wanting an eighth variant. Pause and escalate rather than extending; the thesis claim is that seven covers the asymptotic surface, and any new variant should carry its own dissolution receipt.

**Post-extension proposed text** (Mgr recommendation):
> STOP SIGNAL: wanting a 10th variant (or 11th if Tier-2 IteratedLog/LogLog/InverseAckermann/HyperExp surface). Pause and escalate. Tier-1 textbook coverage (gate #105 carrier-extension 2026-05-13) lands 9 variants covering ConstantCost / PolynomialCost { degree: PositiveRational } / PolyLogCost { exponent: PositiveInt } / LogCost / ProductCost / SumCost / ExponentialCost { base: IntAtLeastTwo } / FactorialCost / UnknownCost — sufficient for the asymptotic surface that R3-load-bearing lenses reason about. Tier-2 (LogLog / InverseAckermann / IteratedLog / HyperExp) is R4-DEFERRED per Director ratification msg_ad5e934d; new variants in R4 require consumer-evidence-justified canvas. UnknownCost("reason: ...") remains algebra-top, but reviewer-tier STOP-SIGNAL fires if a Tier-1-coverable bound is collapsed to Unknown — that is anti-pattern #5 per gate #105.

**Type-level refinement carriers** (per codex BLOCKING on PR #2828 — `docs/modeling-discipline.md` Practice 2 + Practice 6: invariants encoded in type system, NOT fold normalizer):

- `PositiveRational` — strict-positive Rational; constructor accepts only `> 0` values. Follows `DegreeAtLeastTwo` precedent at `src/v3/std/algebra.dag:171-173` (inductive carrier where illegal cases are structurally unrepresentable).
- `PositiveInt` — strict-positive Int; same Peano-style precedent.
- `IntAtLeastTwo` — Int ≥ 2; strict-mirror of `DegreeAtLeastTwo` shape.

These types make `degree ≤ 0`, `exponent ≤ 0`, `base ≤ 1` **structurally unrepresentable** at the carrier level — no normalizer-time enforcement required. Aligns with `feedback_state_space_vs_behavioral_invariants`.

Variant count: **10** post-Q2-Y (recommended), or **11** if Q2-X is ratified.

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

## §10. Anti-patterns (5 Director-enumerated + 2 derived)

### Director-enumerated

1. Any Tier 2 variant named without consumer-evidence (premature variants) — §8 disposition: not introducing
2. Any Path B revival (RootCost as separate variant — Practice-4 RED) — §9: explicitly not done
3. Linear-Polynomial split decision authored without canvas (substrate-shape question goes through Mgr) — this canvas IS the §4 Q2 disposition
4. Dominance lattice fudging via string-tagged Rational (use real ordered-witness) — §3 Q1-c addresses
5. UnknownCost used for textbook-Tier-1-coverable bounds post-promotion (STOP-SIGNAL violation) — §6 STOP-SIGNAL text encodes

### Mgr-derived (encoded for worker review)

6. `OrderedField<T>` witness landed without strict-mirror of `OrderedRing<T>` (Q1-c structural-mirror discipline)
7. `LinearCost`-consumer paths preserved alongside `PolynomialCost(degree=1)` (Q2-Y atomic-migration; bridge variants violate §P5)

## §11. Cost-of-change accounting

Per `INVARIANTS.md` "Cost of Change":

| State | Files to edit to add one new asymptotic-bound consumer |
|---|---|
| Pre-canvas (today) | ≥3 (variant if Tier-2-coverable; algebra rules; consumer site) |
| Post-canvas (9-variant Q2-Y) | 1 (consumer constructs the appropriate Tier-1 variant directly) |

For exotic (Tier-2) bounds: still requires UnknownCost("reason") at consumer site — that's the R4-deferral receipt.

## §12. Open questions for ratification

Director ratification on:

- **Q1**: a / b / c (Mgr-rec: c — layered OrderedField introduction + lazy consumer migration)
- **Q2**: X (keep Linear separate) / Y (collapse to PolynomialCost(degree=1)) (Mgr-rec: Y per §P5)
- **Q3**: §5 algebra interaction rule table; §5.1 PolyLog-vs-Product disposition (Mgr-rec: b composite); §5.2 FactorialCost squared (Mgr-rec: b UnknownCost)
- **Q4**: §6 STOP-SIGNAL text (9 or 10 variants depending on Q2)
- **Q5**: This canvas (ratification-of-canvas is itself the Q5 disposition)
- **§8 Tier-2 mechanism**: defer to R4 (Mgr-rec) or accept IteratedAlgebra mechanism (rejected by §8 analysis)

## §13. Reference

- §1.8 row #105 (PM PR #2824 pending) — authority anchor
- `src/v3/std/algebra.dag:190-197` — current 7-variant SymbolicCost
- `src/v3/std/algebra.dag:60-72` — current STOP SIGNAL
- `dsl/std/algebra.dag:268-286` — OrderedRing<T> precedent
- `dsl/std/algebra.dag:287` — Field<T> (no order)
- `dsl/std/rational.dag:26` — Rational = Field<FieldOfFractions<Int>>
- `docs/design-symbolic-cost-algebra.md` — current algebra
- Director ratification: PM msg_4fd650b7 / Director msg_ad5e934d
- `feedback_groundedness_gates_lenses` — Tier-2 structural-extension caveat
- `feedback_strict_mirror_vs_novel_substrate_fact` — Q1-c applies
- `feedback_load_bearing_ratchet_preservation` — Q2-Y migration discipline

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
