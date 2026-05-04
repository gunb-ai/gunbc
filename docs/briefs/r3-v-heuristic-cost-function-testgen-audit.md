# R3 Verification - Heuristic Cost Function Testgen Audit

**Status:** AUDIT RECEIPT - docs-only. This audits whether the
heuristic-cost-function domain can demonstrate
`integration_testgen_demonstrated_on_at_least_one_domain` against current
substrate. It does not author a `TestClaim`, add a runner predicate, or extend
`SymbolicCost` carriers.

**Scope:** the current SymbolicCost / cost-lens substrate in:

- `src/v3/std/algebra.dag`
- `src/v3/std/dimensions.dag`
- `src/v3/lenses/cost.dag`
- `src/v3/std/verification.dag`
- `docs/design-tests-as-data-completeness.md`

## Contract Restatement

The fifth T-Tests-As-Data-Completeness closure gate is:

```text
integration_testgen_demonstrated_on_at_least_one_domain
```

For this dispatch, the selected domain is heuristic-cost-function. The worked
demonstration should prove that a cost-invariant test can be generated from
structural `.dag` data rather than remaining as a hand-authored Rust assertion.
PR #1430's product-zero bug class is the representative invariant: symbolic
cost multiplication must treat `ConstantCost(0)` as a semiring annihilator, so a
product such as `Linear(n) * 0` normalizes to `ConstantCost(0)`.

That bug class is not an open prerequisite at this audit head: the R3
debt-paydown ledger marks "SymbolicCost semiring annihilation violation" as
Retired by PR #1555, and current `algebra.dag` contains the corresponding
`collapse_on_multiplicative_zero` implementation plus the Rust acceptance test
named below.

The audit question is narrower than "does SymbolicCost currently have tests?"
It asks whether Verification can author a generated `TestClaim` directly
against current substrate, or whether the demonstration must route through
Substrate / Cost-Lens first.

## Current Implementation Audit

`src/v3/std/algebra.dag` has enough structural vocabulary to state the
product-zero invariant:

- `SymbolicCost` is the seven-variant asymptotic carrier and inhabits
  `Semiring<SymbolicCost>`.
- `ProductCost(NonSingletonList<SymbolicCost>)`, `ConstantCost(Int)`, and
  `LinearCost(SizeVariable)` are first-class variants, not strings.
- `iterate(bound, body)` constructs a product and calls `normalize`.
- `collapse_on_multiplicative_zero` preserves zero as an annihilator before
  `drop_multiplicative_one` removes multiplicative identities.

There is also an existing hand-authored Rust test,
`product_with_constant_zero_collapses_to_zero`, in
`lane2_stage_2d_symbolic_cost_test.rs`. It calls `iterate(linear(port),
constant(0))` and asserts `ConstantCost(0)`.

The existing M1.5 testgen harness already emits generated `TestClaim` values
for a cost family: `lens_testgen.rs` computes an integer complexity-lens result
for a named `witness` bind and emits `TestPredicate::CostBounded`. The ignored
spot-checks in `m1_5_testgen_test.rs` verify the representative generated claim
`TestClaim witness has bounded cost`. That is real testgen precedent, but it is
not the same property surface as SymbolicCost product-zero: `CostBounded` sees a
scalar threshold, not a structural `SymbolicCost` expression or semiring law.

The cost lens surface is less complete than the algebra surface:
`src/v3/lenses/cost.dag` can produce `Lookup<SymbolicCost>` per port and the
Rust-side `analyze_symbolic_cost_dimension` can package the result as
`DimensionReport<SymbolicCost>`, but the `.dag`
`data symbolic_cost_dimension: AnalysisDimension<SymbolicCost>` declaration is
explicitly deferred on class-5 data-body lowering. The file also states that no
behavioral cementing test exercises the lens yet.

## Carrier Shape Audit

### SymbolicCost Carrier

**Sufficient for the representative invariant.** The product-zero property is
expressible with existing structural facts:

```text
left: SymbolicCost
right: SymbolicCost
operation: product / iterate
expected: ConstantCost(0)
```

No new `SymbolicCost` variant is needed. In fact, extending the carrier for
this invariant would be the wrong direction: zero and product already live in
the algebra, and `UnknownCost` is explicitly not a failure channel.

### Dimension / Cost-Lens Carrier

**Partially sufficient, but not enough for a generated integration claim.** The
existing `DimensionReport<SymbolicCost>` carrier is the right report surface,
and the cost lens already has the `Lookup<SymbolicCost>` API. The missing piece
is not a report variant; it is an executable `.dag` declaration that can be
referenced by a generated `TestClaim` without routing through Rust-only helper
code.

### Testgen Carrier

**Not sufficient today for the SymbolicCost closure-gate demonstration.**
Current `TestClaim` is enumerated: one `source: String`, one `file_name`, one
`TestPredicate`. The hand-Rust M1.5 generator can materialize `CostBounded`
claims today, but the substrate does not have a predicate that compares
structural `SymbolicCost` expressions or generated symbolic-cost reports.
`docs/design-tests-as-data-completeness.md` also says the broader
property-based/generated surface still requires `ProgramGenerator`,
`ProgramShape`, `Quantifier`, `QuantifiedTestClaim`, and `SuiteClaim` substrate
additions, plus runner dispatch over generated program families.

The current `CostBounded` predicate can check a named bind against a scalar
threshold, but the product-zero invariant is structural equality of a
SymbolicCost expression after normalization. Encoding it through `CostBounded`
would lose the law shape. Encoding it through `OutputEquals` would force
rendered strings or host-side helper output into the predicate identity.

## TestClaim Predicate Sketch

The clean strict-fire shape is a generated or quantified claim that consumes a
structural tuple equivalent to:

```rust
struct SymbolicCostProductZeroCase {
    name: DeclarationRef,
    left: SymbolicCost,
    right: SymbolicCost,          // must include ConstantCost(0)
    operation: CostOperation,     // Product / Iterate; or a MethodRef to iterate
    expected: SymbolicCost,       // ConstantCost(0)
}

struct GeneratedCostInvariantClaim {
    generator: ProgramGenerator,  // produces SymbolicCostProductZeroCase rows
    quantifier: ForAll,
    predicate: SymbolicCostExprEquals {
        actual: DeclarationRef,   // producer for normalize(ProductCost(...))
        expected: SymbolicCost,
    }
}
```

The first concrete case can be singleton-generated:

```text
case: product_zero_linear
left: LinearCost(SizeVariable { source_port: p })
right: ConstantCost(0)
operation: iterate
expected: ConstantCost(0)
```

That is enough to demonstrate integration testgen on one domain once the
testgen substrate can carry generated cases. It is intentionally not the full
L4-L7 semiring-law witness suite; exhaustive law coverage belongs to the
Verification algebraic-law lane.

Required structural facts:

| Fact | Exists today? | Current authority |
|---|---:|---|
| `SymbolicCost` variants | Yes | `src/v3/std/algebra.dag` |
| Product / iterate normalization | Yes | `src/v3/std/algebra.dag` |
| Product-zero hand-Rust acceptance | Yes | `lane2_stage_2d_symbolic_cost_test.rs` |
| `DimensionReport<SymbolicCost>` carrier | Yes | `src/v3/std/dimensions.dag` |
| `.dag` symbolic-cost dimension value | No | deferred in `src/v3/lenses/cost.dag` |
| Generated/quantified TestClaim family | No | designed in `docs/design-tests-as-data-completeness.md`; not landed in `verification.dag` |
| SymbolicCost expression-equality predicate | No | would need substrate decision or reuse after generated producer emits comparable reports |

## Conversion Cost Classification

| Classification | Verdict | Rationale |
|---|---|---|
| **(a) Verification-side rewrite-only** | **No** | Verification can restate the invariant, but cannot honestly author a generated integration claim against current `TestClaim` alone. A rewrite-only PR would either copy the existing Rust assertion into another Rust test or encode equality through strings / helper output. |
| **(b) Substrate carrier extension** | **Yes, for testgen; no for SymbolicCost** | `SymbolicCost` already carries the invariant. The missing carrier is the tests-as-data generated/quantified surface and possibly a typed SymbolicCost equality predicate or generated report-equality producer. |
| **(c) Cross-program Cost-Lens coordination** | **Yes** | If the demonstration is required to flow through `Lens<SymbolicCost>` / `DimensionReport<SymbolicCost>` rather than direct algebra normalization, Cost-Lens/Substrate must land the executable symbolic-cost dimension value or equivalent producer path first. |

## Verdict

**Routing needed before the closure-gate implementation.**

The heuristic-cost-function domain is not too vague: product-zero is a concrete
structural invariant and current `SymbolicCost` algebra can express it without
carrier growth. The obstruction is the generated `TestClaim` / testgen side of
the gate, not the cost algebra.

Recommended routing:

1. **Substrate / T-Tests-As-Data:** land the generated/quantified claim carriers
   from `docs/design-tests-as-data-completeness.md` or an equivalent
   single-authority successor.
2. **Substrate / Cost-Lens:** decide whether the demonstration consumes direct
   `SymbolicCost` expression equality or `DimensionReport<SymbolicCost>`
   equality from a symbolic-cost producer.
3. **Verification:** after those carriers land, author a follow-on
   `TestClaim` PR for the singleton generated product-zero case. That follow-on
   PR, not this audit, is the candidate to close
   `integration_testgen_demonstrated_on_at_least_one_domain`.

## Debt Receipt

This audit does not close a Debt-Paydown row directly and does not directly
close the fifth T-Tests-As-Data-Completeness gate.

Debt found + routed: the heuristic-cost-function domain is viable, but the
implementation needs generated/quantified TestClaim substrate and a typed
SymbolicCost equality/report producer decision before Verification can author
the closure-gate demonstration without a textual or Rust-only bridge.

## Test Plan

- `git diff --check`
