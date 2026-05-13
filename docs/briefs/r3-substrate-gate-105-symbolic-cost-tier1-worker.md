---
status: dispatchable-on-cascade (worker brief; ratified shape per canvas PR #2828 Director-ratified 2026-05-13 via PM msg_a055c38b relaying msg_d86a5987; dispatch gates on PR #2824 row #105 anchor + PR #2828 canvas landing — both AND)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #105 `symbolic_cost_textbook_coverage_landed`
parent canvas: PR #2828 / `docs/briefs/r3-substrate-gate-105-symbolic-cost-tier1-canvas.md` — Q1-Q5 + §8 Tier-2 RATIFIED
ratification anchor: PM msg_a055c38b relaying Director msg_d86a5987
row anchor: PM PR #2824
---

# Gate #105 — SymbolicCost Tier 1 carrier-extension worker brief

## §0. Status — DISPATCH-READY (cascade-gated)

Director ratified all Q1-Q5 + §8 Tier-2 dispositions per PM msg_a055c38b. Worker dispatch gates on:
1. **PR #2824 merge** — §1.8 row #105 authority anchor lands
2. **PR #2828 merge** — canvas authored shape lands (both PRs may merge in parallel)

This brief encodes the ratified Tier-1 4-addition + 1-collapse + 1-promotion structure as a single coordinated PR. **No sub-phase merges independently** (mirrors F-β.2 atomic-PR receipt; gate-row STOP discipline).

## §1. Ratified scope summary

Net 7 → **10** SymbolicCost variants:
- **PROMOTE**: `PolynomialCost.degree: DegreeAtLeastTwo` → `PolynomialCost.degree: Rational where degree > 0` (subsumes √n, ∛n, n^(2/3), non-Int 2.373, AND the absorbed Linear case at degree=1)
- **REMOVE**: `LinearCost(SizeVariable)` — atomic migration to `PolynomialCost { var: v, degree: 1 }` (Q2-Y)
- **ADD**: `PolyLogCost { var: SizeVariable, exponent: Int }` for log^k n
- **ADD**: `ExponentialCost { base: Int, var: SizeVariable }` for c^n with c ≥ 2
- **ADD**: `FactorialCost { var: SizeVariable }` for n!

Sibling substrate (Q1-c — separate but co-landing in same PR):
- **ADD**: `OrderedField<T>` witness in `dsl/std/algebra.dag` (strict-mirror of `OrderedRing<T>` at :268-286)
- **PROMOTE**: `Rational = OrderedField<FieldOfFractions<Int>>` at `dsl/std/rational.dag:26`

## §2. Authority chain (verbatim)

- `src/v3/std/algebra.dag:60-72` — current STOP SIGNAL (will be rewritten per §4)
- `src/v3/std/algebra.dag:190-197` — current 7-variant `SymbolicCost`
- `dsl/std/algebra.dag:268-286` — `OrderedRing<T>` precedent (`compare/lt/le/gt/ge` witness pattern)
- `dsl/std/algebra.dag:287-...` — current `Field<T>` (no order)
- `dsl/std/rational.dag:26` — `type Rational = Field<FieldOfFractions<Int>>`
- `docs/design-symbolic-cost-algebra.md` — current algebra
- Canvas: PR #2828 / `docs/briefs/r3-substrate-gate-105-symbolic-cost-tier1-canvas.md` §§3-9
- Ratification: PM msg_a055c38b relaying Director msg_d86a5987

## §3. Phase A — OrderedField<T> witness landing (Q1-c)

**Sub-phase A.1 — Declare `OrderedField<T>`** in `dsl/std/algebra.dag` adjacent to existing `Field<T>`. Strict-mirror of `OrderedRing<T>` (:268-286) — same 7 order operators added on top of Field:

```dag
type OrderedField<T> {
  add: fn(T, T) -> T
  sub: fn(T, T) -> T
  zero: T
  negate: fn(T) -> T
  mul: fn(T, T) -> T
  div: fn(T, T) -> Result<T, DivError>
  one: T
  reciprocal: fn(T) -> T
  divide: fn(T, T) -> T
  compare: fn(T, T) -> Ordering
  eq: fn(T, T) -> Bool
  ne: fn(T, T) -> Bool
  lt: fn(T, T) -> Bool
  le: fn(T, T) -> Bool
  gt: fn(T, T) -> Bool
  ge: fn(T, T) -> Bool
}
```

**Sub-phase A.2 — Re-declare `Rational`** at `dsl/std/rational.dag:26`:

```dag
type Rational = OrderedField<FieldOfFractions<Int>>
```

**Sub-phase A.3 — Lazy consumer migration** (anti-pattern #6): do NOT migrate existing `Field<T>` consumers preemptively. Field + OrderedField coexist as the Ring/OrderedRing parallel. Only the cost-lens (this PR) needs OrderedField operations on Rational.

**Sub-phase A.4 — Witness realization**: the `OrderedField<FieldOfFractions<Int>>` realization can use Int cross-multiplication for compare (`a/b ≶ c/d ⟺ ad ≶ bc` when b·d > 0). Hand-author the data witness alongside the type.

## §4. Phase B — STOP SIGNAL rewrite (Q4)

Replace `src/v3/std/algebra.dag:60-72` block with (Director-ratified verbatim per canvas §6):

```
// STOP SIGNAL: wanting an 11th variant (or 12th if Tier-2
// IteratedLog/LogLog/InverseAckermann/HyperExp surface). Pause and
// escalate. Tier-1 textbook coverage (gate #105 carrier-extension
// 2026-05-13, PR #<this PR>) lands 10 variants covering ConstantCost /
// PolynomialCost(Rational > 0) / PolyLogCost / LogCost / ProductCost /
// SumCost / ExponentialCost / FactorialCost / UnknownCost — sufficient
// for the asymptotic surface that R3-load-bearing lenses reason about.
// Tier-2 (LogLog / InverseAckermann / IteratedLog / HyperExp) is
// R4-DEFERRED per Director ratification msg_d86a5987; new variants in
// R4 require consumer-evidence-justified canvas. UnknownCost("reason: ...")
// remains algebra-top, but reviewer-tier STOP-SIGNAL fires if a
// Tier-1-coverable bound is collapsed to Unknown — that is anti-pattern
// #5 per gate #105.
```

Cite gate #105 + PR #2828 + msg_d86a5987 in the comment block.

## §5. Phase C — SymbolicCost carrier reshape (Q2-Y + variant additions)

### §5.0 — Type-level refinement carriers (per codex BLOCKING on PR #2828)

Before reshaping SymbolicCost, introduce three refinement carriers that make illegal field values **structurally unrepresentable** (Practice 2 + Practice 6; `docs/modeling-discipline.md`). Strict-mirror of `DegreeAtLeastTwo` precedent at `src/v3/std/algebra.dag:171-173`:

```dag
// Mirror of DegreeAtLeastTwo (lines 171-173): Peano-style inductive
// carrier where illegal cases are structurally unrepresentable, not
// normalizer-cleaned-up.
type IntAtLeastTwo
  = IntTwo
  | IntSuccessor { previous: IntAtLeastTwo }

type PositiveInt
  = One
  | PositiveSuccessor { previous: PositiveInt }

// PositiveRational: strict-positive Rational. Constructor accepts only
// numerator > 0 + denominator > 0 (excluding 0 and negative). Realization
// follows OrderedField<Rational> from Phase A.
type PositiveRational {
  numerator: PositiveInt
  denominator: PositiveInt
}
```

These types make `degree ≤ 0`, `exponent ≤ 0`, `base ≤ 1` structurally impossible to construct — no fold-time enforcement required.

### §5.1 — Replace SymbolicCost variant set

Replace `src/v3/std/algebra.dag:190-197` with:

```dag
type SymbolicCost inhabits Semiring<SymbolicCost>
  = ConstantCost(Int)
  | PolynomialCost { var: SizeVariable, degree: PositiveRational }   // Q2-Y: absorbs LinearCost via degree=1; degree > 0 by carrier
  | PolyLogCost { var: SizeVariable, exponent: PositiveInt }         // NEW: log^k n; exponent ≥ 1 by carrier
  | ProductCost(NonSingletonList<SymbolicCost>)
  | SumCost(NonSingletonList<SymbolicCost>)
  | LogCost(SizeVariable)
  | ExponentialCost { base: IntAtLeastTwo, var: SizeVariable }       // NEW: c^n with c ≥ 2 by carrier
  | FactorialCost { var: SizeVariable }                              // NEW: n!
  | UnknownCost(String)
```

10 variants. **LinearCost is REMOVED** (anti-pattern #7: no bridge variants; atomic migration).

**Invariants encoded at carrier level** (Practice 2/6 — NOT fold normalizer):
- `PolynomialCost.degree: PositiveRational` — `degree ≤ 0` is structurally impossible
- `PolyLogCost.exponent: PositiveInt` — `exponent ≤ 0` is structurally impossible
- `ExponentialCost.base: IntAtLeastTwo` — `base ≤ 1` is structurally impossible

Reviewers MUST flag any attempt to use raw `Rational` / `Int` for these fields.

## §6. Phase D — Algebra interaction rules (Q3)

Update sum + product fold logic to implement the Director-ratified rule table (canvas §5):

| Operation | Result |
|---|---|
| `PolyCost(d1) + PolyCost(d2)` | `PolyCost(max(d1, d2))` (uses `OrderedField<Rational>.compare`) |
| `PolyCost(d1) · PolyCost(d2)` | `PolyCost(d1 + d2)` (uses `OrderedField<Rational>.add`) |
| `PolyCost(d) · LogCost(v)` | `ProductCost([PolyCost(d), LogCost(v)])` (§5.1: composite, NOT PolyLogCost) |
| `PolyLogCost(v, k1) · PolyLogCost(v, k2)` | `PolyLogCost(v, k1+k2)` |
| `PolyLogCost(v, k1) + PolyLogCost(v, k2)` | `PolyLogCost(v, max(k1, k2))` |
| `PolyCost(d) + ExpCost(c, v)` | `ExpCost(c, v)` (exp dominates poly) |
| `PolyCost(d) · ExpCost(c, v)` | `ExpCost(c, v)` (exp absorbs poly) |
| `ExpCost(c1, v) + ExpCost(c2, v)`, c1 ≤ c2 | `ExpCost(c2, v)` (dominant base) |
| `ExpCost(c1, v) · ExpCost(c2, v)` | `ExpCost(c1·c2, v)` (multiplicative composition) |
| `FactorialCost(v) + anything` | `FactorialCost(v)` (factorial dominates) |
| `FactorialCost(v) · PolyCost(d)` | `FactorialCost(v)` (factorial absorbs poly) |
| `FactorialCost(v) · ExpCost(c, v)` | `FactorialCost(v)` (factorial dominates exp) |
| `FactorialCost(v) · FactorialCost(v)` | `UnknownCost("(v!)² exceeds Tier 1 — pending R4 named-variant canvas")` (§5.2 verbatim) |

Normalization invariants in fold (canvas §5.3):
- `PolyCost(1/2) · PolyCost(1/2)` normalizes to `PolyCost(1)` via `OrderedField.add`
- Sum/product dominance applies `OrderedField.compare` for all max-style reductions
- The collapsed Linear (`PolyCost(d=1)`) participates uniformly in sum/product

## §7. Phase E — Bootstrap ratchet test

Mirror `src/v3/compiler/tests/integration/cementing/` shape (cf. `complexity_lens_behavioral_completion.rs`):

`src/v3/compiler/tests/integration/cementing/symbolic_cost_tier1_carrier_test.rs` asserting:
- 10 variant count (assert against `cost.dag` or `algebra.dag` source); REMOVED LinearCost
- All 10 variant names + field shapes structurally present
- `OrderedField<T>` declared in `dsl/std/algebra.dag` with 16 record fields (Field 9 + Order 7)
- `Rational = OrderedField<FieldOfFractions<Int>>` at `dsl/std/rational.dag:26`
- Algebra rule sample tests (≥6 of §6 rules): assert fold output for representative inputs (e.g., `PolyCost(1/2) · PolyCost(1/2)` produces `PolyCost(1)`; `ExpCost(2,n) · PolyCost(d)` produces `ExpCost(2,n)`; `FactorialCost(n)²` produces `UnknownCost` with the exact §5.2 reason-string)
- STOP-SIGNAL text at `:60-72` contains new "11th variant" wording

## §8. Phase F — Consumer migration (atomic)

Migrate cost-lens consumers from `LinearCost(v)` → `PolynomialCost { var: v, degree: 1 }` in the **same PR**. No bridge variant; no `LinearCost`-fallback path (anti-pattern #7).

Inventory required (worker greps at HEAD before authoring):
- `git grep -nE "\\bLinearCost\\b" src/v3/ dsl/` — all consumer sites
- For each site, replace with `PolynomialCost { var: <existing-var>, degree: rational_from_int(1) }` (or canvas-ratified helper name)
- The fold algebra under §6 ensures correctness: `PolyCost(1) + PolyCost(1) = PolyCost(1)`; `PolyCost(1) · PolyCost(1) = PolyCost(2)` etc.

## §9. Phase G — §1.8 row #105 ledger update

After Phase A-F land + tests green, update `docs/r3-program-plan.md` §1.8 row #105 from DECLARED (or CANVAS_RATIFIED if PM ledger-maintenance landed first) → **CONSUMER_LANDED** with cite to this PR + canvas PR #2828 + Director msg_d86a5987.

## §10. STOP conditions

1. **`OrderedRing<T>` shape drift** at HEAD — if `dsl/std/algebra.dag:268-286` no longer carries the exact 14-field signature this brief mirrors, **STOP** and surface — strict-mirror authority broken.
2. **Existing `LinearCost`-consumer surface differs from canvas assumption** — if grep reveals consumer paths that can't migrate to `PolynomialCost(degree=1)` losslessly (e.g., type-level dispatches on LinearCost variant-tag), **STOP** — anti-pattern #7 atomic-migration discipline requires lossless migration.
3. **`Rational` carrier not at `dsl/std/rational.dag:26`** — if Rational has moved / changed shape since 2026-05-13 grep, **STOP** — Q1-c re-declaration target is wrong.
4. **Variant-name collision** at HEAD — if any of `PolyLogCost` / `ExponentialCost` / `FactorialCost` / `OrderedField` appear from parallel landing, **STOP** for de-duplication.
5. **Algebra rule §5.2 violation tempted** — if Phase D authoring tempts a named (n!)² variant or non-Unknown disposition, **STOP** — anti-pattern #5 fires; the rule disposition is Director-ratified.
6. **PR #2824 not merged at dispatch** OR **PR #2828 not merged at dispatch** — both gates AND; if either is unmerged, **STOP** and surface to Mgr; worker dispatch is blocked.

## §11. 7 anti-patterns (5 Director-enumerated + 2 Mgr-derived per canvas §10)

PR body MUST cite each verbatim + assert receipt-of-compliance:

1. Any Tier 2 variant named without consumer-evidence (premature variants)
2. Any Path B revival (RootCost as separate variant — Practice-4 RED)
3. Linear-Polynomial split decision authored without canvas (substrate-shape question goes through Mgr)
4. Dominance lattice fudging via string-tagged Rational (use real ordered-witness) — §3 Q1-c addresses
5. UnknownCost used for textbook-Tier-1-coverable bounds post-promotion (STOP-SIGNAL violation)
6. `OrderedField<T>` witness landed without strict-mirror of `OrderedRing<T>` (Q1-c structural-mirror discipline)
7. `LinearCost`-consumer paths preserved alongside `PolynomialCost(degree=1)` (Q2-Y atomic-migration; bridge variants violate §P5)

## §12. 5 reviewer ratchets (Director-enumerated for PR review)

1. **Q1-c integrity**: OrderedField MUST strict-mirror OrderedRing record shape (compare/lt/le/gt/ge minimum)
2. **Q2-Y integrity**: NO LinearCost preservation paths alongside PolyCost(degree=1); atomic migration receipt required
3. **Q3 algebra rules**: §5.1 + §5.2 dispositions are load-bearing; reviewers flag deviation
4. **Q4 STOP-SIGNAL text**: must land at `src/v3/std/algebra.dag:69-72` with new variant cap at 11 (10 ratified + 1 trigger)
5. **All 7 anti-patterns enforceable** at PR review

## §13. Verification

- `cargo test --workspace` green
- New hermetic ratchet `symbolic_cost_tier1_carrier_test.rs` (§7) asserts all 4 verification axes (variant set / OrderedField / Rational / algebra rules sample / STOP-SIGNAL text)
- Pre-existing cost-lens behavioral tests still green (Phase F migration must preserve semantic equivalence: `LinearCost(v)` and `PolynomialCost { var: v, degree: 1 }` must produce identical lens output for all consumers)
- PR body cites:
  - Gate #105 closure (Phase G ledger update)
  - Canvas PR #2828 + Director disposition (PM msg_a055c38b) verbatim Q1-Q5 + §8
  - 7 anti-patterns receipt-of-compliance (§11)
  - 5 reviewer ratchets (§12) — explicit assertion-of-compliance per item

## §14. Out of scope

- **Tier 2 variants** (LogLog / InverseAckermann / IteratedLog / HyperExp) — R4-deferred per Director §8. Worker must NOT add these.
- **`Field<T>` consumer migration** beyond cost-lens — Q1-c lazy migration; Field stays in place
- **InverseAckermann / IteratedAlgebra mechanism** — canvas §8 finding accepted; not introduced
- **Cost-lens behavioral changes** — this is a carrier-extension PR, not a semantics change; lens output must be backwards-compatible modulo Linear→Poly(d=1) lossless rewrite
- **`docs/design-symbolic-cost-algebra.md` rewrite** — out of scope; tracked separately as doc-drift sweep

## §15. PR body framing template

```
Closes gate #105 symbolic_cost_textbook_coverage_landed.

Carrier extended per Director-ratified Path A Tier 1 (canvas PR #2828;
ratification PM msg_a055c38b relaying msg_d86a5987 2026-05-13).

Net 7 → 10 SymbolicCost variants (Q2-Y collapse Linear into
PolynomialCost(degree=1)):
[paste §5 variant set verbatim]

Companion substrate (Q1-c):
- OrderedField<T> witness in dsl/std/algebra.dag (strict-mirror of OrderedRing<T>)
- Rational = OrderedField<FieldOfFractions<Int>>

STOP-SIGNAL re-reset to 11 (10 ratified + 1 trigger) at algebra.dag:60-72.

Algebra rules §5/§6 implemented verbatim per canvas; (n!)² → UnknownCost
("(v!)² exceeds Tier 1 — pending R4 named-variant canvas").

7 anti-patterns receipt-of-compliance:
[enumerate each + cite that the implementation does not violate it]

5 reviewer ratchets compliance:
[enumerate each + cite assertion]

§1.8 row #105 updated: CANVAS_RATIFIED → CONSUMER_LANDED.
```

## §16. Reference

- Canvas: PR #2828 / `docs/briefs/r3-substrate-gate-105-symbolic-cost-tier1-canvas.md`
- Director ratification relay: PM msg_a055c38b (relaying Director msg_d86a5987)
- Row anchor: PM PR #2824
- Sibling-witness precedent: `dsl/std/algebra.dag:268-286` (OrderedRing<T>)
- Current SymbolicCost: `src/v3/std/algebra.dag:190-197`
- Current STOP-SIGNAL: `src/v3/std/algebra.dag:60-72`
- Rational: `dsl/std/rational.dag:26`
- `feedback_strict_mirror_vs_novel_substrate_fact` — Q1-c discipline
- `feedback_state_space_vs_behavioral_invariants` — Q2-Y refinement-vs-fold discipline
- `feedback_naming_is_aliasing` — §5.1 PolyLog-vs-Product semantic-collision avoidance
- `feedback_no_short_term_solutions` — Q2-Y absorption-over-coexistence

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
**Dispatch gate**: PR #2824 AND PR #2828 both merged.
