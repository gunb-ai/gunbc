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

Net 7 → **9** SymbolicCost variants:
- **PROMOTE**: `PolynomialCost.degree: DegreeAtLeastTwo` → `PolynomialCost.degree: Rational where degree > 0` (subsumes √n, ∛n, n^(2/3), non-Int 2.373, AND the absorbed Linear case at degree=1)
- **REMOVE**: `LinearCost(SizeVariable)` — atomic migration to `PolynomialCost { var: v, degree: 1 }` (Q2-Y)
- **ADD**: `PolyLogCost { var: SizeVariable, exponent: Int }` for log^k n
- **ADD**: `ExponentialCost { base: Int, var: SizeVariable }` for c^n with c ≥ 2
- **ADD**: `FactorialCost { var: SizeVariable }` for n!

Sibling substrate (Q1 — Rational ordering; **Director RATIFIED Q1-α per msg_676ad4e7 2026-05-13**, retracting prior Q1-c msg_d86a5987):

**Original Q1-c (OrderedField introduction) REJECTED**: `Field<T>` at `dsl/std/algebra.dag:294` already carries `compare: fn(T, T) -> Ordering` (precedent: `OrderedRing<T>.compare` at `:276`). Introducing `OrderedField<T>` would have duplicated Field.compare authority, violating INVARIANTS P1 + row #24 + Q-MachineConstraint-Carrier "no dual representations".

**Director-ratified Q1-α**:
- **NO `OrderedField<T>` introduction**
- **NO `Rational` re-declaration** at `dsl/std/rational.dag:26`
- Cost-lens fold uses `Rational.compare` via existing `Field.compare`
- Cost-lens-local free functions: `rational_lt(a, b) = compare(a, b) == Less` (and `_le`, `_gt`, `_ge`, `_eq`, `_ne` as needed by §6 algebra rules)

## §2. Authority chain (verbatim)

- `src/v3/std/algebra.dag:60-72` — current STOP SIGNAL (will be rewritten per §4)
- `src/v3/std/algebra.dag:190-197` — current 7-variant `SymbolicCost`
- `dsl/std/algebra.dag:268-286` — `OrderedRing<T>` precedent (`compare/lt/le/gt/ge` witness pattern)
- `dsl/std/algebra.dag:287-295` — current `Field<T>` (carries `compare: fn(T, T) -> Ordering` at :294; missing derived predicates lt/le/gt/ge/eq/ne — see §3 Q1-α premise correction)
- `dsl/std/rational.dag:26` — `type Rational = Field<FieldOfFractions<Int>>`
- `docs/design-symbolic-cost-algebra.md` — current algebra
- Canvas: PR #2828 / `docs/briefs/r3-substrate-gate-105-symbolic-cost-tier1-canvas.md` §§3-9
- Ratification: PM msg_a055c38b relaying Director msg_d86a5987

## §3. Phase A — Rational ordering helpers (Q1-α; Director RATIFIED msg_676ad4e7)

**Premise correction landed**: `Field<T>` at `dsl/std/algebra.dag:294` carries `compare: fn(T, T) -> Ordering`. Original Phase A introducing `OrderedField<T>` was based on stale premise (operator BLOCKING canvas:48); Director retracted msg_d86a5987 Q1-c via msg_676ad4e7.

**Under Q1-α (ratified)**:

- **NO `OrderedField<T>` introduction** — Field carries `compare` already
- **NO `Rational` re-declaration** at `dsl/std/rational.dag:26` — stays as `Field<FieldOfFractions<Int>>`
- **Cost-lens-local convenience helpers**: define lt/le/gt/ge as free functions on Rational in cost-lens module, derived from existing `Rational.compare`:

```dag
// In src/v3/lenses/cost.dag (or equivalent cost-lens module).
// Convenience predicates derived from Rational.compare. Single source of
// truth remains Field.compare at dsl/std/algebra.dag:294.

fn rational_lt(a: Rational, b: Rational) -> Bool =
  rational.compare(a, b) == Less

fn rational_le(a: Rational, b: Rational) -> Bool =
  rational.compare(a, b) == Less || rational.compare(a, b) == Equal

fn rational_gt(a: Rational, b: Rational) -> Bool =
  rational.compare(a, b) == Greater

fn rational_max(a: Rational, b: Rational) -> Rational =
  match rational.compare(a, b) {
    Greater => a
    _ => b
  }
```

**Witness realization**: Rational's Field-witness realization (`compare` for `FieldOfFractions<Int>`) uses Int cross-multiplication: `a/b ≶ c/d ⟺ ad ≶ bc` when b·d > 0. If this realization is NOT yet wired at HEAD, Phase A may need to land the data witness; worker grep-verifies before authoring.

Q1-β (extend Field with predicate fields) REJECTED by Director — doubles Field carrier surface (7→13) for predicates derivable from Ordering pattern-match; violates cost-of-change minimization.

Q1-γ (OrderedField as Field-superset via inheritance) REJECTED by Director — DSL grammar inheritance/superset typing R4-scope at earliest; forward-incompatible with R3 timeline.

## §4. Phase B — STOP SIGNAL rewrite (Q4)

Replace `src/v3/std/algebra.dag:60-72` block with (Director-ratified verbatim per canvas §6):

```
// STOP SIGNAL: wanting a 10th variant (or 11th if Tier-2
// IteratedLog/LogLog/InverseAckermann/HyperExp surface). Pause and
// escalate. Tier-1 textbook coverage (gate #105 carrier-extension
// 2026-05-13, PR #<this PR>) lands 9 variants covering ConstantCost /
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
// follows Rational compare via existing Field<FieldOfFractions<Int>>.compare (no OrderedField introduction; Q1-α).
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

9 variants. **LinearCost is REMOVED** (anti-pattern #7: no bridge variants; atomic migration).

**Invariants encoded at carrier level** (Practice 2/6 — NOT fold normalizer):
- `PolynomialCost.degree: PositiveRational` — `degree ≤ 0` is structurally impossible
- `PolyLogCost.exponent: PositiveInt` — `exponent ≤ 0` is structurally impossible
- `ExponentialCost.base: IntAtLeastTwo` — `base ≤ 1` is structurally impossible

Reviewers MUST flag any attempt to use raw `Rational` / `Int` for these fields.

## §6. Phase D — Algebra interaction rules (Q3)

Update sum + product fold logic to implement the Director-ratified rule table (canvas §5):

| Operation | Result |
|---|---|
| `PolyCost(d1) + PolyCost(d2)` | `PolyCost(max(d1, d2))` (uses `Rational.compare` (Q1-α free function `rational_max`)) |
| `PolyCost(d1) · PolyCost(d2)` | `PolyCost(d1 + d2)` (uses `Rational.add` (Field)) |
| `PolyCost(d) · LogCost(v)` | `ProductCost([PolyCost(d), LogCost(v)])` (§5.1: composite, NOT PolyLogCost) |
| `PolyLogCost(v, k1) · PolyLogCost(v, k2)` | `PolyLogCost(v, k1+k2)` |
| `PolyLogCost(v, k1) + PolyLogCost(v, k2)` | `PolyLogCost(v, max(k1, k2))` |
| `PolyCost(d) + ExpCost(c, v)` | `ExpCost(c, v)` (exp dominates poly) |
| `PolyCost(d) · ExpCost(c, v)` | `ProductCost([PolyCost(d), ExpCost(c, v)])` — composite, NOT absorbed (n^d · c^n is NOT O(c^n) strictly; `n^d · c^n / c^n = n^d` is unbounded; multiplicative absorption is unsound per operator BLOCKING worker:140) |
| `ExpCost(c1, v) + ExpCost(c2, v)`, c1 ≤ c2 | `ExpCost(c2, v)` (dominant base) |
| `ExpCost(c1, v) · ExpCost(c2, v)` | `ExpCost(c1·c2, v)` (multiplicative composition) |
| `FactorialCost(v) + anything` | `FactorialCost(v)` (factorial dominates) |
| `FactorialCost(v) · PolyCost(d)` | `ProductCost([FactorialCost(v), PolyCost(d)])` — composite, NOT absorbed (same unsoundness; n! · n^d / n! = n^d unbounded) |
| `FactorialCost(v) · ExpCost(c, v)` | `ProductCost([FactorialCost(v), ExpCost(c, v)])` — composite, NOT absorbed (n! · c^n / n! = c^n unbounded) |
| `FactorialCost(v) · FactorialCost(v)` | `UnknownCost("(v!)² exceeds Tier 1 — pending R4 named-variant canvas")` (§5.2 verbatim) |

Normalization invariants in fold (canvas §5.3):
- `PolyCost(1/2) · PolyCost(1/2)` normalizes to `PolyCost(1)` via `Rational.add` (Field)
- Sum/product dominance applies `Rational.compare` (Field) for all max-style reductions
- The collapsed Linear (`PolyCost(d=1)`) participates uniformly in sum/product

## §7. Phase E — Bootstrap ratchet test

Mirror `src/v3/compiler/tests/integration/cementing/` shape (cf. `complexity_lens_behavioral_completion.rs`):

`src/v3/compiler/tests/integration/cementing/symbolic_cost_tier1_carrier_test.rs` asserting:
- 10 variant count (assert against `cost.dag` or `algebra.dag` source); REMOVED LinearCost
- All 10 variant names + field shapes structurally present
- Phase A Q1-α deliverables present: rational_lt/le/gt/ge/eq/ne free functions in cost-lens module; NO OrderedField type declared
- `Rational = Field<FieldOfFractions<Int>>` UNCHANGED at `dsl/std/rational.dag:26` (Q1-α)
- Algebra rule sample tests (≥6 of §6 rules): assert fold output for representative inputs (e.g., `PolyCost(1/2) · PolyCost(1/2)` produces `PolyCost(1)`; `ExpCost(2,n) · PolyCost(d)` produces `ExpCost(2,n)`; `FactorialCost(n)²` produces `UnknownCost` with the exact §5.2 reason-string)
- STOP-SIGNAL text at `:60-72` contains new "10th variant" wording

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
4. **Variant-name collision** at HEAD — if any of `PolyLogCost` / `ExponentialCost` / `FactorialCost` / `PositiveRational` / `IntAtLeastTwo` / `PositiveInt` appear from parallel landing, **STOP** for de-duplication.
5. **Algebra rule §5.2 violation tempted** — if Phase D authoring tempts a named (n!)² variant or non-Unknown disposition, **STOP** — anti-pattern #5 fires; the rule disposition is Director-ratified.
6. **PR #2824 not merged at dispatch** OR **PR #2828 not merged at dispatch** — both gates AND; if either is unmerged, **STOP** and surface to Mgr; worker dispatch is blocked.

## §11. 8 anti-patterns (6 Director-enumerated + 2 Mgr-derived per canvas §10)

PR body MUST cite each verbatim + assert receipt-of-compliance:

1. Any Tier 2 variant named without consumer-evidence (premature variants)
2. Any Path B revival (RootCost as separate variant — Practice-4 RED)
3. Linear-Polynomial split decision authored without canvas (substrate-shape question goes through Mgr)
4. Dominance lattice fudging via string-tagged Rational (use real ordered-witness) — §3 Q1-α addresses via existing Field.compare
5. UnknownCost used for textbook-Tier-1-coverable bounds post-promotion (STOP-SIGNAL violation)
6. **Director-ratified msg_676ad4e7**: Introducing parallel ordered-algebraic-structure carriers (`Ordered<X>`) when the underlying carrier already provides `compare: fn(T, T) -> Ordering` — lens-local predicate derivation from Ordering pattern-match is the canonical path
7. Multiplicative absorption rules (`X · Y = X`) where one variant absorbs another asymptotically — sound for SUM, NOT PRODUCT (n^d · c^n is NOT O(c^n)); cross-class products MUST be ProductCost composite (per operator BLOCKING worker:140)
8. `LinearCost`-consumer paths preserved alongside `PolynomialCost(degree=1)` (Q2-Y atomic-migration; bridge variants violate §P5)

## §12. 5 reviewer ratchets (Director-enumerated for PR review)

1. **Q1-α integrity**: NO new OrderedField type; Rational ordering uses existing Field.compare via cost-lens-local free functions
2. **Q2-Y integrity**: NO LinearCost preservation paths alongside PolyCost(degree=1); atomic migration receipt required
3. **Q3 algebra rules**: §5.1 + §5.2 dispositions are load-bearing; reviewers flag deviation
4. **Q4 STOP-SIGNAL text**: must land at `src/v3/std/algebra.dag:69-72` with new variant cap at 10 (9 ratified + 1 trigger)
5. **All 8 anti-patterns enforceable** at PR review

## §13. Verification

- `cargo test --workspace` green
- New hermetic ratchet `symbolic_cost_tier1_carrier_test.rs` (§7) asserts all 4 verification axes (variant set / refinement carriers (PositiveRational/PositiveInt/IntAtLeastTwo) / algebra rules sample / STOP-SIGNAL text)
- Pre-existing cost-lens behavioral tests still green (Phase F migration must preserve semantic equivalence: `LinearCost(v)` and `PolynomialCost { var: v, degree: 1 }` must produce identical lens output for all consumers)
- PR body cites:
  - Gate #105 closure (Phase G ledger update)
  - Canvas PR #2828 + Director disposition (PM msg_a055c38b) verbatim Q1-Q5 + §8
  - 8 anti-patterns receipt-of-compliance (§11)
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

Net 7 → 9 SymbolicCost variants (Q2-Y collapse Linear into
PolynomialCost(degree=1)):
[paste §5 variant set verbatim]

Companion substrate (Q1-c):
- Cost-lens-local Rational ordering helpers (Q1-α; NO new OrderedField)
- Rational stays as Field<FieldOfFractions<Int>> (UNCHANGED)

STOP-SIGNAL re-reset to 10 (9 ratified + 1 trigger) at algebra.dag:60-72.

Algebra rules §5/§6 implemented verbatim per canvas; (n!)² → UnknownCost
("(v!)² exceeds Tier 1 — pending R4 named-variant canvas").

8 anti-patterns receipt-of-compliance:
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
