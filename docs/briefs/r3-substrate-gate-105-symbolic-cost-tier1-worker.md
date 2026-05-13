---
status: dispatchable-on-cascade (worker brief; ratified shape per canvas PR #2828 Director-ratified 2026-05-13 via composite ratification — PM msg_a055c38b relaying Director msg_d86a5987 (Q2-Q5 + §8 base ratification) AS RECONCILED BY Director msg_676ad4e7 (Q1-α supersession of prior Q1-c); dispatch gates on PR #2824 row #105 anchor + PR #2828 canvas landing — both AND)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #105 `symbolic_cost_textbook_coverage_landed`
parent canvas: PR #2828 / `docs/briefs/r3-substrate-gate-105-symbolic-cost-tier1-canvas.md` — Q1-Q5 + §8 Tier-2 RATIFIED
ratification anchor: composite — PM msg_a055c38b relaying Director msg_d86a5987 (Q2-Q5 + §8) AS RECONCILED BY Director msg_676ad4e7 (Q1-α supersedes prior Q1-c)
row anchor: PM PR #2824
---

# Gate #105 — SymbolicCost Tier 1 carrier-extension worker brief

## §0. Status — DISPATCH-READY (cascade-gated)

Director ratified all Q1-Q5 + §8 Tier-2 dispositions per **composite ratification**: PM msg_a055c38b relaying Director msg_d86a5987 (Q2-Q5 + §8 base) AS RECONCILED BY Director msg_676ad4e7 (Q1-α supersedes prior Q1-c). Worker dispatch gates on:
1. **PR #2824 merge** — §1.8 row #105 authority anchor lands
2. **PR #2828 merge** — canvas authored shape lands (both PRs may merge in parallel)

This brief encodes the ratified Tier-1 4-addition + 1-collapse + 1-promotion structure as a single coordinated PR. **No sub-phase merges independently** (mirrors F-β.2 atomic-PR receipt; gate-row STOP discipline).

## §1. Ratified scope summary

Net 7 → **9** SymbolicCost variants:
- **PROMOTE**: `PolynomialCost.degree: DegreeAtLeastTwo` → `PolynomialCost.degree: Rational` (signed; **no `where` refinement** per Director msg_2c1bfb0e scope-extension) — subsumes positive (√n, ∛n, n^(2/3), n^2.373, absorbed Linear at degree=1) AND negative (1/n = n^(-1), 1/n² = n^(-2), asymptotic-decay coverage per operator directive 2026-05-13)
- **REMOVE**: `LinearCost(SizeVariable)` — atomic migration to `PolynomialCost { var: v, degree: 1 }` (Q2-Y)
- **ADD**: `PolyLogCost { var: SizeVariable, exponent: PolyLogExponent }` for log^k n (PolyLogExponent = Rational > 1 refinement; supports log^7.5 n cited Tier-1 AKS case)
- **ADD**: `ExponentialCost { base: ExponentialBase, var: SizeVariable }` for c^n with c ≥ 2 (ExponentialBase = Int ≥ 2 refinement)
- **ADD**: `FactorialCost { var: SizeVariable }` for n!

Sibling substrate (Q1 — Rational ordering; **Director RATIFIED Q1-α per msg_676ad4e7 2026-05-13**, retracting prior Q1-c msg_d86a5987):

**Original Q1-c (OrderedField introduction) REJECTED**: `Field<T>` at `dsl/std/algebra.dag:294` already carries `compare: fn(T, T) -> Ordering` (precedent: `OrderedRing<T>.compare` at `:276`). Introducing `OrderedField<T>` would have duplicated Field.compare authority, violating INVARIANTS P1 + row #24 + Q-MachineConstraint-Carrier "no dual representations".

**Director-ratified Q1-α**:
- **NO `OrderedField<T>` introduction**
- **NO `Rational` re-declaration** at `dsl/std/rational.dag:26`
- Cost-lens fold uses `Rational.compare` via existing `Field.compare`
- Cost-lens-local free functions: `rational_lt(a, b) = compare(a, b) == Less` (and `_le`, `_gt`, `_ge`, `_eq`, `_ne` as needed by §6 algebra rules)

## §2. Authority chain (verbatim)

- `src/v3/std/algebra.dag:69-72` — current STOP SIGNAL (will be rewritten per §4)
- `src/v3/std/algebra.dag:190-197` — current 7-variant `SymbolicCost`
- `dsl/std/algebra.dag:268-286` — `OrderedRing<T>` precedent (`compare/lt/le/gt/ge` witness pattern)
- `dsl/std/algebra.dag:287-295` — current `Field<T>` (carries `compare: fn(T, T) -> Ordering` at :294; missing derived predicates lt/le/gt/ge/eq/ne — see §3 Q1-α premise correction)
- `dsl/std/rational.dag:26` — `type Rational = Field<FieldOfFractions<Int>>`
- `docs/design-symbolic-cost-algebra.md` — current algebra
- Canvas: PR #2828 / `docs/briefs/r3-substrate-gate-105-symbolic-cost-tier1-canvas.md` §§3-9
- Ratification (composite): PM msg_a055c38b relaying Director msg_d86a5987 (Q2-Q5 + §8) RECONCILED BY Director msg_676ad4e7 (Q1-α supersedes prior Q1-c)

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

Replace `src/v3/std/algebra.dag:69-72` block — and ONLY that 4-line block — with (Director-ratified verbatim per canvas §6):

**DO NOT** touch lines `:49-67`: those carry the 4-pattern dissolution receipt (Pattern 1/2/3/4 commentary) and are out-of-scope for this PR. Phase B is a surgical replacement of the STOP-SIGNAL paragraph only; the dissolution-receipt prose updates (if any) belong to a separate canvas + PR (codex BLOCKING 10904 explicit warning).

```
// STOP SIGNAL: wanting a 10th variant (or 11th if Tier-2
// IteratedLog/LogLog/InverseAckermann/HyperExp surface). Pause and
// escalate. Tier-1 textbook coverage (gate #105 carrier-extension
// 2026-05-13, PR #<this PR>) lands 9 variants covering ConstantCost /
// PolynomialCost { degree: Rational } (signed per Q6) / PolyLogCost { exponent:
// PolyLogExponent (Rational > 1) } / LogCost / ProductCost / SumCost /
// ExponentialCost { base: ExponentialBase (Int ≥ 2) } / FactorialCost /
// UnknownCost — sufficient
// for the asymptotic surface that R3-load-bearing lenses reason about.
// Tier-2 (LogLog / InverseAckermann / IteratedLog / HyperExp) is
// R4-DEFERRED per Director ratification msg_d86a5987; new variants in
// R4 require consumer-evidence-justified canvas. UnknownCost("reason: ...")
// remains algebra-top, but reviewer-tier STOP-SIGNAL fires if a
// Tier-1-coverable bound is collapsed to Unknown — that is anti-pattern
// #5 per gate #105.
```

Cite gate #105 + PR #2828 + composite ratification (msg_d86a5987 base RECONCILED BY msg_676ad4e7 Q1-α supersession) in the comment block.

## §5. Phase C — SymbolicCost carrier reshape (Q2-Y + variant additions)

### §5.0 — Type-level refinement carriers (per codex BLOCKING on PR #2828)

**Refinement-mechanism path (CORRECT per PM msg_a52ed981)**: the substrate refinement-mechanism `type X = Y where predicate` is **already ratified** at HEAD per gunbc#828 issuecomment-4390333451 (Path 3) + Director Option 2 ratification at gunbc#828 issuecomment-4390199218. Precedent: `dsl/std/integer.dag:181` — `type PositiveInt = Nat where gt_zero`. The `KNOWN_PREDICATES` registry at `src/v3/compiler/src/lower.rs:798-862` carries: `range` / `non_empty` / `brand` / `gt_zero` / `unicode_scalar`.

This **invalidates** the prior product-shape carriers (codex BLOCKING 014544f4 finding #1 + operator BLOCKING worker:104). Canonical shape uses refinement over existing carriers:

```dag
// ExponentialBase: Int ≥ 2. range(min: 2) — `range` predicate at
// lower.rs:817 with allowed_carriers Int + Nat.
type ExponentialBase = Int where range(min: 2)

// PositiveRational: Rational > 0. REQUIRES KNOWN_PREDICATES extension:
// add `Rational` to gt_zero's allowed_carriers (currently Nat + Int).
// Worker authors atomic with carrier landing per Phase A.
// (PositiveRational DROPPED per Director msg_2c1bfb0e scope-extension —
//  PolynomialCost.degree is plain signed Rational; admits negative-degree decay.
//  No gt_zero allowed_carriers extension needed for PolynomialCost.)

// PolyLogExponent: Rational > 1. REQUIRES KNOWN_PREDICATES extension:
// add NEW `gt_one` predicate (allowed_carriers: Rational + Int) to
// the registry. Mirrors `gt_zero` shape. Worker authors atomic with
// carrier landing per Phase A.
type PolyLogExponent = Rational where gt_one
```

**Phase A KNOWN_PREDICATES extensions** (Mgr-tier scope; required for refinement-mechanism authority):
1. Extend `gt_zero`'s `allowed_carriers` to include `Rational` (was `Nat + Int`)
2. Add new `gt_one` predicate (allowed_carriers: `Rational + Int`; arg_shape: `Bare`)
3. Both extensions land in same PR as carrier-shape changes — atomic per §P5

**ZERO new authority introduced**: PolyLogExponent is a refinement of canonical Rational; ExponentialBase is a refinement of canonical Int. PolynomialCost.degree uses plain signed Rational (no refinement; Q6 scope-extension). Practice 4 / P1 / Q-MachineConstraint-Carrier hard constraint "no dual representations" all satisfied.

**Authority chain for refinement mechanism**:
- gunbc#828 issuecomment-4390333451 (Path 3 RATIFIED)
- gunbc#828 issuecomment-4390199218 (Director Option 2)
- `dsl/std/integer.dag:171-181` precedent (`PositiveInt = Nat where gt_zero`)
- `src/v3/compiler/src/lower.rs:798-862` KNOWN_PREDICATES registry

These refinements make `exponent ≤ 1` (PolyLogCost) and `base ≤ 1` (ExponentialCost) structurally impossible to construct — no fold-time enforcement required, no new authority. PolynomialCost.degree intentionally has no such refinement: Q6 signed Rational admits negative degrees for asymptotic-decay coverage.

**HARD STOP**: do NOT author PolyLogExponent / ExponentialBase as fresh records / inductive sums when refinement over canonical carrier is available. That pattern is codex BLOCKING 014544f4 finding #1 + operator BLOCKING worker:104 anti-pattern (now §10 #8 below). (PositiveRational is no longer in scope per Q6 scope-extension; PolynomialCost.degree is plain signed Rational.)

### §5.1 — Replace SymbolicCost variant set

Replace `src/v3/std/algebra.dag:190-197` with:

```dag
type SymbolicCost inhabits Semiring<SymbolicCost>
  = ConstantCost(Int)
  | PolynomialCost { var: SizeVariable, degree: Rational }   // Q2-Y: absorbs LinearCost via degree=1; Q6 signed Rational (admits decay)
  | PolyLogCost { var: SizeVariable, exponent: PolyLogExponent }     // NEW: log^k n; exponent > 1 by carrier (admits 2, 3/2, 7.5; rejects 0/1 collapses)
  | ProductCost(NonSingletonList<SymbolicCost>)
  | SumCost(NonSingletonList<SymbolicCost>)
  | LogCost(SizeVariable)
  | ExponentialCost { base: ExponentialBase, var: SizeVariable }     // NEW: c^n with c ≥ 2 by carrier
  | FactorialCost { var: SizeVariable }                              // NEW: n!
  | UnknownCost(String)
```

9 variants. **LinearCost is REMOVED** (anti-pattern #7: no bridge variants; atomic migration).

**Invariants encoded at carrier level** (Practice 2/6 — NOT fold normalizer):
- `PolynomialCost.degree: Rational` — **signed, no refinement** (Q6); negative degrees admitted for asymptotic-decay coverage. Asymptotic-dominance rule (Q6) is encoded in the algebra fold layer via `Field.compare` with reverse-sign-convention.
- `PolyLogCost.exponent: PolyLogExponent` — `exponent ≤ 1` is structurally impossible (excludes 0=ConstantCost-collapse + 1=LogCost-collapse semantic dups); supports rational exponents like log^7.5 n (AKS primality cited Tier-1 case)
- `ExponentialCost.base: ExponentialBase` — `base ≤ 1` is structurally impossible (excludes 0=degenerate + 1=ConstantCost-collapse)

Reviewers MUST flag any attempt to use raw `Rational` / `Int` for these fields.

## §6. Phase D — Algebra interaction rules (Q3 + Q6 + Q7)

### §6.0 — Q7 canonical-form preservation (Director RATIFIED msg_2c1bfb0e)

**SymbolicCost preserves the full expression.** The same-variable algebra fold rules below apply to **derived-operation** projection (`dominant_term: SymbolicCost -> SymbolicCost`), NOT to canonical-form construction. Sum-canonicalization sorts terms by Q6 dominance ordering and preserves every term:

```
canonicalize(SumCost([n + log(n) + 1/n]))
  = SumCost([PolyCost(n, 1), LogCost(n), PolyCost(n, -1)])  // dominance-sorted, all terms preserved
```

Big-O projection is a separate function applied by consumers:

```
fn dominant_term(c: SymbolicCost) -> SymbolicCost {
  // Apply §6.1 dominance rules ONLY here, NOT during SumCost construction.
}
fn asymptotic_class(c: SymbolicCost) -> ComplexityClass { ... }
```

Worker MUST implement BOTH `canonicalize` (preserves all terms) and `dominant_term` (applies §6.1 dominance fold). Tests assert canonical-form preservation under mixed-sign (e.g., `n + 1/n` canonicalizes to 2-term SumCost, not 1-term PolyCost(n, 1)).

### §6.1 — Q6 Asymptotic-dominance ordering with signed degrees (Director RATIFIED msg_2c1bfb0e)

Dominance rule (used by `dominant_term`, NOT by canonicalize):

- Positive degrees a, b > 0: `n^a > n^b` iff `a > b`
- Positive vs negative: `n^a > n^(-b)` for a, b > 0 (positive grows; negative decays)
- Constant vs negative: `1 > n^(-a)` for any a > 0 (constant dominates decay)
- Two negatives: `n^(-a) > n^(-b)` iff `a < b` (least-negative dominates; 1/n > 1/n²)

**Implementation**: encode as a derived ordering over `(SizeVariable, Rational)` pairs using `Field.compare` for the underlying Rational comparison — Q1-α authority. NO separate ordered carrier introduced.

### §6.2 — Same-variable algebra fold rules (applied by `dominant_term`)

Update sum + product fold logic to implement the Director-ratified rule table (canvas §5).

**Variable-scoping precondition** (per codex BLOCKING 014544f4 finding #3): the rules below assume **same-variable operands**. Different-variable operations (e.g., `PolyCost(n, d1) + PolyCost(m, d2)` where n ≠ m) are NOT folded by dominance — they preserve as `SumCost` / `ProductCost` composite. Algebra dominance is variable-local; cross-variable dominance is undefined within Tier-1 substrate (requires Tier-2 or polynomial-multivariate ratification post-R3).

Same-variable rules (operands share `SizeVariable`):

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
| `FactorialCost(v) + FactorialCost(v)` | `FactorialCost(v)` (same-variable absorption) |
| `FactorialCost(v) + ExpCost(c, v)` | `FactorialCost(v)` (factorial dominates exp, same-variable) |
| `FactorialCost(v) + PolyCost(d, v)` | `FactorialCost(v)` (factorial dominates poly, same-variable) |
| `FactorialCost(v) + PolyLogCost(v, k)` | `FactorialCost(v)` (factorial dominates polylog, same-variable) |
| `FactorialCost(v) + LogCost(v)` | `FactorialCost(v)` (factorial dominates log, same-variable) |
| `FactorialCost(v) + ConstantCost(c)` | `FactorialCost(v)` (factorial dominates constant) |
| `FactorialCost(v) + UnknownCost(r)` | `SumCost([FactorialCost(v), UnknownCost(r)])` — composite; UnknownCost is conservative-top per `src/v3/std/algebra.dag` documentation; NEVER absorbed (operator BLOCKING worker:158) |
| `FactorialCost(v) + FactorialCost(w)` (v ≠ w) | `SumCost([FactorialCost(v), FactorialCost(w)])` — composite; cross-variable dominance undefined per §6 precondition |
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
- 9 variant count (assert against `cost.dag` or `algebra.dag` source); REMOVED LinearCost (PolynomialCost.degree promotion to Rational is not a new variant)
- All 9 variant names + field shapes structurally present
- Phase A Q1-α deliverables present: rational_lt/le/gt/ge/eq/ne free functions in cost-lens module; NO OrderedField type declared
- `Rational = Field<FieldOfFractions<Int>>` UNCHANGED at `dsl/std/rational.dag:26` (Q1-α)
- Algebra rule sample tests (≥6 of §6 rules): assert fold output for representative inputs (e.g., `PolyCost(1/2) · PolyCost(1/2)` produces `PolyCost(1)`; `ExpCost(2,n) · PolyCost(d)` produces `ProductCost([ExpCost(2,n), PolyCost(d)])` — multiplicative cross-class is NOT absorbed per §6 + anti-pattern #9 (asymptotic absorption is sound for SUM but unsound for PRODUCT); `ExpCost(2,n) + PolyCost(d)` produces `ExpCost(2,n)` — additive cross-class absorption is sound; `FactorialCost(n)²` produces `UnknownCost` with the exact §5.2 reason-string)
- STOP-SIGNAL text at `:69-72` contains new "10th variant" wording

## §8. Phase F — Consumer migration (atomic)

Migrate cost-lens consumers from `LinearCost(v)` → `PolynomialCost { var: v, degree: 1 }` in the **same PR**. No bridge variant; no `LinearCost`-fallback path (anti-pattern #7).

Inventory required (worker greps at HEAD before authoring):
- `git grep -nE "\\bLinearCost\\b" src/v3/ dsl/` — all consumer sites
- For each site, replace with `PolynomialCost { var: <existing-var>, degree: rational_from_int(1) }` (or canvas-ratified helper name)
- The fold algebra under §6 ensures correctness: `PolyCost(1) + PolyCost(1) = PolyCost(1)`; `PolyCost(1) · PolyCost(1) = PolyCost(2)` etc.

## §9. Phase G — §1.8 row #105 ledger update

After Phase A-F land + tests green, update `docs/r3-program-plan.md` §1.8 row #105 from DECLARED (or CANVAS_RATIFIED if PM ledger-maintenance landed first) → **CONSUMER_LANDED** with cite to this PR + canvas PR #2828 + composite ratification (Director msg_d86a5987 base + msg_676ad4e7 Q1-α supersession).

## §10. STOP conditions

1. **`OrderedRing<T>` shape drift** at HEAD — if `dsl/std/algebra.dag:268-286` no longer carries the exact 14-field signature this brief mirrors, **STOP** and surface — strict-mirror authority broken.
2. **Existing `LinearCost`-consumer surface differs from canvas assumption** — if grep reveals consumer paths that can't migrate to `PolynomialCost(degree=1)` losslessly (e.g., type-level dispatches on LinearCost variant-tag), **STOP** — anti-pattern #7 atomic-migration discipline requires lossless migration.
3. **`Rational` carrier not at `dsl/std/rational.dag:26`** — if Rational has moved / changed shape since 2026-05-13 grep, **STOP** — Q1-α refinement target is wrong.
4. **Variant-name collision** at HEAD — if any of `PolyLogCost` / `ExponentialCost` / `FactorialCost` / `ExponentialBase` / `PolyLogExponent` appear from parallel landing, **STOP** for de-duplication. (`PositiveInt` already exists at `dsl/std/integer.dag:181` — reuse. `PositiveRational` is OUT of scope per Q6 — if encountered at HEAD as a parallel landing, that's an anti-pattern #7 fire.)
5. **Algebra rule §5.2 violation tempted** — if Phase D authoring tempts a named (n!)² variant or non-Unknown disposition, **STOP** — anti-pattern #5 fires; the rule disposition is Director-ratified.
6. **PR #2824 not merged at dispatch** OR **PR #2828 not merged at dispatch** — both gates AND; if either is unmerged, **STOP** and surface to Mgr; worker dispatch is blocked.

## §11. 10 anti-patterns (7 Director-enumerated/pending + 3 Mgr-derived per canvas §10)

PR body MUST cite each verbatim + assert receipt-of-compliance:

1. Any Tier 2 variant named without consumer-evidence (premature variants)
2. Any Path B revival (RootCost as separate variant — Practice-4 RED)
3. Linear-Polynomial split decision authored without canvas (substrate-shape question goes through Mgr)
4. Dominance lattice fudging via string-tagged Rational (use real ordered-witness) — §3 Q1-α addresses via existing Field.compare
5. UnknownCost used for textbook-Tier-1-coverable bounds post-promotion (STOP-SIGNAL violation)
6. **Director-ratified msg_676ad4e7**: Introducing parallel ordered-algebraic-structure carriers (`Ordered<X>`) when the underlying carrier already provides `compare: fn(T, T) -> Ordering` — lens-local predicate derivation from Ordering pattern-match is the canonical path
7. **Director ratified per operator BLOCKING PR #2824:333**: Tier-1 variant constructed with raw Int/Rational exponent/base admitting illegal collapse values (exponent=0/1 for PolyLogCost; base=0/1 for ExponentialCost) bypassing refinement type — PolyLogExponent + ExponentialBase required at carrier level (Practice 2/6). **PolynomialCost.degree is excluded** per Director msg_2c1bfb0e scope-extension — signed Rational degrees intentionally admit negative values for asymptotic-decay coverage (Q6).
8. **PM-grep-corrected per msg_a52ed981 + codex 014544f4 finding #1**: Parallel rational-number carriers (`PositiveRational { num: PositiveInt; denom: PositiveInt }`, inductive `PolyLogExponentSuccessor | PolyLogExponentFractional`, or any fresh record/sum shape) when refinement over canonical `Rational = Field<FieldOfFractions<Int>>` carrier is available via ratified `type X = Y where predicate` mechanism (gunbc#828 issuecomment-4390333451 Path 3 RATIFIED; precedent `PositiveInt = Nat where gt_zero` at `dsl/std/integer.dag:181`). Anti-pattern fires on ANY fresh-carrier shape when refinement is available.
9. Multiplicative absorption rules (`X · Y = X`) where one variant absorbs another asymptotically — sound for SUM, NOT PRODUCT (n^d · c^n is NOT O(c^n)); cross-class products MUST be ProductCost composite (per operator BLOCKING worker:140)
10. `LinearCost`-consumer paths preserved alongside `PolynomialCost(degree=1)` (Q2-Y atomic-migration; bridge variants violate §P5)
11. **Director-added msg_2c1bfb0e**: Introducing parallel `InverseCost(SymbolicCost)` / `ReciprocalCost` / `DecayCost` variants when carrier-extension via signed `degree: Rational` is structurally clean. Same Q1-α / Q1-c lesson class — don't bridge-wrap when carrier-extension dissolves the question (`feedback_dissolve_bridges` + `feedback_no_metadata_markers`).

## §12. 5 reviewer ratchets (Director-enumerated for PR review)

1. **Q1-α integrity**: NO new OrderedField type; Rational ordering uses existing Field.compare via cost-lens-local free functions
2. **Q2-Y integrity**: NO LinearCost preservation paths alongside PolyCost(degree=1); atomic migration receipt required
3. **Q3 algebra rules**: §5.1 + §5.2 dispositions are load-bearing; reviewers flag deviation
4. **Q4 STOP-SIGNAL text**: must land at `src/v3/std/algebra.dag:69-72` with new variant cap at 10 (9 ratified + 1 trigger)
5. **All 10 anti-patterns enforceable** at PR review

## §13. Verification

- `cargo test --workspace` green
- New hermetic ratchet `symbolic_cost_tier1_carrier_test.rs` (§7) asserts all 4 verification axes (variant set / refinement carriers (PositiveInt/ExponentialBase/PolyLogExponent — NOT PositiveRational; PolynomialCost.degree is plain signed Rational per Q6) / algebra rules sample / STOP-SIGNAL text)
- **INVARIANTS P5 receipt for the new hand-Rust test file** (per claude APPROVE 10773 + codex BLOCKING 014544f4 finding #4): authoring `symbolic_cost_tier1_carrier_test.rs` adds new hand-Rust under `src/v3/compiler/tests/`. Per P5 "Pure Bootstrap" discipline, this PR's body MUST cite **exactly ONE P5 receipt category** with concrete path + LOC count:
  - (a) **hand-Rust deletion of equivalent or greater LOC**: cite specific deleted file/lines + LOC count
  - (b) **SG-0 census shrink receipt**: cite specific SG-0 cell + shrink delta
  - (c) **named-lane T-PB-B ROADMAP row deferral**: cite the ROADMAP row by ID + explicit dissolution-trigger condition

  **Worker MUST pick exactly one and document with concrete numbers, NOT narrative.** Phase F (LinearCost variant removal + fallback dispatch collapse) is the LIKELY (a) source — but worker measures actual LOC at authoring time. If Phase F deletion LOC < new test file LOC, worker MUST pick (b) or (c). Codex 014544f4 BLOCKING explicitly: "feature migration as debt receipt" is NOT a clean P5 receipt — concrete numbers required.
- Pre-existing cost-lens behavioral tests still green (Phase F migration must preserve semantic equivalence: `LinearCost(v)` and `PolynomialCost { var: v, degree: 1 }` must produce identical lens output for all consumers)
- PR body cites:
  - Gate #105 closure (Phase G ledger update)
  - Canvas PR #2828 + composite Director ratification verbatim Q1-Q5 + §8 (PM msg_a055c38b relaying msg_d86a5987 Q2-Q5/§8 base RECONCILED BY msg_676ad4e7 Q1-α supersession of prior Q1-c)
  - 10 anti-patterns receipt-of-compliance (§11)
  - 5 reviewer ratchets (§12) — explicit assertion-of-compliance per item

## §14. Out of scope

- **Tier 2 variants** (LogLog / InverseAckermann / IteratedLog / HyperExp) — R4-deferred per Director §8. Worker must NOT add these.
- **`Field<T>` consumer migration** beyond cost-lens — Q1-α; Field stays in place unchanged (compare already present)
- **InverseAckermann / IteratedAlgebra mechanism** — canvas §8 finding accepted; not introduced
- **Cost-lens behavioral changes** — this is a carrier-extension PR, not a semantics change; lens output must be backwards-compatible modulo Linear→Poly(d=1) lossless rewrite
- **`docs/design-symbolic-cost-algebra.md` rewrite** — out of scope; tracked separately as doc-drift sweep

## §15. PR body framing template

```
Closes gate #105 symbolic_cost_textbook_coverage_landed.

Carrier extended per Director-ratified Path A Tier 1 (canvas PR #2828;
ratification composite: PM msg_a055c38b relaying Director msg_d86a5987 (Q2-Q5 + §8 base) RECONCILED BY Director msg_676ad4e7 (Q1-α supersedes prior Q1-c) 2026-05-13.

Net 7 → 9 SymbolicCost variants (Q2-Y collapse Linear into
PolynomialCost(degree=1)):
[paste §5 variant set verbatim]

Companion substrate (Q1-α):
- Cost-lens-local Rational ordering helpers (NO new OrderedField; derived from existing Field.compare)
- Rational stays as Field<FieldOfFractions<Int>> (UNCHANGED)

STOP-SIGNAL re-reset to 10 (9 ratified + 1 trigger) at algebra.dag:69-72.

Algebra rules §5/§6 implemented verbatim per canvas; (n!)² → UnknownCost
("(v!)² exceeds Tier 1 — pending R4 named-variant canvas").

10 anti-patterns receipt-of-compliance:
[enumerate each + cite that the implementation does not violate it]

5 reviewer ratchets compliance:
[enumerate each + cite assertion]

§1.8 row #105 updated: CANVAS_RATIFIED → CONSUMER_LANDED.
```

## §16. Reference

- Director scope-extension ratification msg_2c1bfb0e (Q6 signed-Rational + Q7 SymbolicCost preserves expression + anti-pattern #11)

- Canvas: PR #2828 / `docs/briefs/r3-substrate-gate-105-symbolic-cost-tier1-canvas.md`
- Director ratification (composite): PM msg_a055c38b relaying Director msg_d86a5987 (Q2-Q5 + §8 base) RECONCILED BY Director msg_676ad4e7 (Q1-α supersedes prior Q1-c)
- Row anchor: PM PR #2824
- Sibling-witness precedent: `dsl/std/algebra.dag:268-286` (OrderedRing<T>)
- Current SymbolicCost: `src/v3/std/algebra.dag:190-197`
- Current STOP-SIGNAL: `src/v3/std/algebra.dag:69-72`
- Rational: `dsl/std/rational.dag:26`
- `feedback_strict_mirror_vs_novel_substrate_fact` — Q1-α discipline (strict-mirror of existing Field.compare; novel only at lens-local helper layer)
- `feedback_state_space_vs_behavioral_invariants` — Q2-Y refinement-vs-fold discipline
- `feedback_naming_is_aliasing` — §5.1 PolyLog-vs-Product semantic-collision avoidance
- `feedback_no_short_term_solutions` — Q2-Y absorption-over-coexistence

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
**Dispatch gate**: PR #2824 AND PR #2828 both merged.
