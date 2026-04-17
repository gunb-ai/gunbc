> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 2 Stage 2d

# Design DB-7 — Symbolic cost algebra

**Design blocker:** DB-7
**Consumers:** Lane 2 Stage 2d (symbolic cost lens); informs Lane 4 Stage 4c (space bounds follow the same compositional pattern)
**Status:** Design ready for implementer review.
**Depends on:** DB-3 ([design-dimension-abstraction.md](./design-dimension-abstraction.md)) — symbolic cost is a `Dimension<SymbolicCost>` instance

---

## Problem

Structural cost (Lane 1a's `complexity.dag`) reports concrete op counts: `fold(list, 0, +)` returns `1 + |list| * 1 = O(n)` — but the lens reports this as a constant integer based on the count of Behavior nodes visited, NOT as an asymptotic bound. The thesis promises O(n) vs O(n²) diagnostics (THESIS.md §"KF-1: Symbolic bounds"). That's symbolic.

Concrete vs symbolic:
- Concrete: `sum(list_of_7_ints)` costs 7 Adds + fold overhead = 9
- Symbolic: `fn sum(list)` costs O(|list|) regardless of input size

Symbolic cost needs an algebra over expressions like `O(|list|)`, `O(|list|²)`, `O(|outer| * |inner|)`. The design locks down:
- What `SymbolicCost` values look like (the carrier type)
- How composition works (adding, multiplying, taking max)
- How recursion / folds / maps lower into symbolic cost
- What diagnostic text looks like

---

## Design

### SymbolicCost carrier

```dag
// src/v3/lenses/symbolic_cost.dag (new)
module lenses.symbolic_cost

import std.list { List }
import std.dimensions { Dimension, Witness }

// Symbolic cost expression. Carries a bound with named size variables.
type SymbolicCost
  = ConstantCost(Int)                        // O(1) or a known constant
  | LinearCost(SizeVariable)                 // O(n) over named variable
  | PolynomialCost {                          // O(n^k) over named variable
      var: SizeVariable
      degree: Int
    }
  | ProductCost(List<SymbolicCost>)           // O(n * m) — product of independent terms
  | SumCost(List<SymbolicCost>)               // O(n + m) — dominant term wins asymptotically
  | LogCost(SizeVariable)                     // O(log n) — explicit for balanced trees, sort
  | UnknownCost(String)                       // fallback with diagnostic reason

// Named size variable carrying the binding it refers to.
type SizeVariable {
  name: String                                // user-facing, e.g. "|items|"
  source_port: PortId                          // the port whose size gates the cost
}
```

### Dominance / normalization

Asymptotic costs simplify: `O(n + 1) = O(n)`, `O(n² + n) = O(n²)`. The lens normalizes by dropping dominated terms.

```dag
fn dominates(a: SymbolicCost, b: SymbolicCost) -> Bool {
  // a dominates b iff for all values of their variables, a >= b * constant
  // Practical rules:
  //   PolynomialCost(v, k1) dominates PolynomialCost(v, k2) iff k1 >= k2
  //   LinearCost(v) dominates LogCost(v)
  //   ConstantCost(_) dominates nothing
  //   ProductCost and LinearCost/PolynomialCost: check term by term
  ...
}

fn normalize(c: SymbolicCost) -> SymbolicCost {
  // Drop dominated terms from SumCost
  // Flatten nested SumCost and ProductCost
  // Simplify ConstantCost(0) out of sums
  ...
}
```

### Composition operations

```dag
// Sequential composition: one op follows another
fn sequential(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
  normalize(SumCost([a, b]))
}

// Iteration: operation b runs under iteration bounded by a's size
fn iterate(bound: SymbolicCost, body: SymbolicCost) -> SymbolicCost {
  normalize(ProductCost([bound, body]))
}

// Alternative: max of paths (for Branch)
fn max_path(paths: List<SymbolicCost>) -> SymbolicCost {
  normalize(fold(paths, ConstantCost(0), |acc, p|
    if dominates(p, acc) then p else acc
  ))
}
```

### Lowering per Behavior variant

```dag
fn symbolic_cost_of(d: Dag, behavior: Behavior) -> Witness<SymbolicCost> {
  match behavior {
    Value(v) =>
      Inhabits(ConstantCost(0))

    Transform(t) =>
      match target_is_list_builtin(t.target) {
        Some(Fold(list_port)) =>
          // cost = size(list) * body_cost
          Inhabits(iterate(
            linear_in(list_port),
            body_cost_of_fold_lambda(d, t)
          ))
        Some(Map(list_port)) =>
          Inhabits(iterate(
            linear_in(list_port),
            body_cost_of_map_lambda(d, t)
          ))
        Some(Filter(list_port)) =>
          Inhabits(iterate(
            linear_in(list_port),
            body_cost_of_filter_lambda(d, t)
          ))
        Some(Sort(list_port)) =>
          // cost = O(n log n)
          Inhabits(iterate(
            linear_in(list_port),
            LogCost(size_variable_of(list_port))
          ))
        None =>
          // Non-builtin callable: cost = sum of input costs + 1 (the call itself)
          Inhabits(sequential(
            ConstantCost(1),
            sum_map(t.inputs, |port| cost_of_producer(d, port))
          ))
      }

    Branch(b) =>
      // cost = condition cost + max over paths
      Inhabits(sequential(
        cost_of_producer(d, b.input),
        max_path(map(b.paths, |path| cost_of_producer(d, path.result_port)))
      ))

    Loop(l) =>
      // Recursion with structural descent: bounded by recursion depth
      Inhabits(iterate(
        recursion_depth_bound(d, l),
        body_cost_of_loop(d, l)
      ))

    Bind(bind) =>
      Inhabits(cost_of_producer(d, bind.result_port))
  }
}
```

The key case: **Fold/Map/Filter recognize the enclosed list's size variable**, multiply by the lambda body's cost. When the lambda body itself contains a fold over the SAME (captured) list, you get `|list| * |list|` = polynomial.

### Lambda body cost (the `#[ignore]`d test)

```dag
fn body_cost_of_fold_lambda(d: Dag, fold_transform: TransformNode) -> SymbolicCost {
  // Fold's 3rd input is the lambda (in v3's callable realization)
  let lambda_port = nth(fold_transform.inputs, 2)
  let lambda_bind = resolve_producer(d, lambda_port)
  match lambda_bind {
    Some(Behavior::Bind(b)) =>
      // Cost of executing the lambda's body once
      cost_of_producer(d, b.result_port)
    _ =>
      UnknownCost("fold lambda is not a Bind")
  }
}
```

This is what the current forward-fold `complexity.dag` can't attribute to the enclosing fold because fold's operation count treats the lambda as one unit regardless of body. Symbolic `iterate(bound, body)` gets it right: `|list| * body_cost`.

### Nested fold detection (O(n²) diagnostic)

```dag
fn detect_nested_fold(outer: TransformNode, inner: SymbolicCost) -> SymbolicCost? {
  // If inner cost mentions the same SizeVariable as outer's bound,
  // we have a nested fold over the same list.
  match outer_list_size_variable(outer) {
    Some(outer_var) =>
      if cost_references_variable(inner, outer_var) then
        Some(PolynomialCost { var: outer_var, degree: 2 })
      else
        None
    None => None
  }
}
```

When detected, emit:

```
DIAGNOSTIC at <outer fold span>: O(n²) complexity detected

  let all_pairs = fold(items, 0, |outer_acc, x|
    outer_acc + fold(items, 0, |inner_acc, y| inner_acc + x * y)
  )
                        ^^^^^^^ inner fold captures outer's `items`

Symbolic cost: O(|items|²)
FIX: if your algorithm doesn't require comparing every pair, consider a
single fold. If it does (as here), confirm the quadratic cost is intentional.
```

Uses DB-1 `Correction` for the "confirm intentional" annotation (could be a no-op fix `#[allow(quadratic_complexity)]` if we later add such an annotation).

### Dead work detection (thesis doc example)

```dag
fn detect_dead_work(behavior: Behavior, composed_cost: SymbolicCost) -> Diagnostic? {
  // Sort before commutative fold — the sort is wasted
  if is_sort_before_commutative_fold(behavior) {
    Some(Diagnostic {
      kind: DeadWorkDetected {
        what: "sort_by before commutative fold"
        cost: sort_cost_of(behavior)
      }
      ...
    })
  } else {
    None
  }
}
```

### Dimension<SymbolicCost> wiring

```dag
data symbolic_cost_dimension: Dimension<SymbolicCost> = {
  name: "symbolic_cost"
  witness_of: |d, behavior| symbolic_cost_of(d, behavior)
  compose: |a, b| sequential(a, b)    // default — iteration composition is handled inside witness_of
  identity: ConstantCost(0)
  break_diagnostic: |behavior, composed|
    // Dead work detection fires when individual ops report unnecessary cost
    detect_dead_work(behavior, composed)
}
```

The Dimension abstraction (DB-3) drives the walk; symbolic_cost_of contains the lowering rules per variant.

### Display

```dag
fn render(cost: SymbolicCost) -> String {
  match normalize(cost) {
    ConstantCost(0) => "O(1)"
    ConstantCost(k) => "O(1)"  // constants collapse asymptotically
    LinearCost(v) => "O(" + v.name + ")"
    PolynomialCost { var, degree } => "O(" + var.name + "^" + degree + ")"
    ProductCost(terms) => "O(" + join(map(terms, render_factor), " * ") + ")"
    SumCost(terms) =>
      // after normalize, sums only appear with non-dominated heterogeneous terms
      "O(" + join(map(terms, render), " + ") + ")"
    LogCost(v) => "O(log " + v.name + ")"
    UnknownCost(reason) => "O(?) — " + reason
  }
}
```

Examples:
- `fold(items, 0, +)` renders as `O(|items|)`
- `all_pairs` renders as `O(|items|²)`
- `fold(items, 0, |acc, x| acc + fold(jobs, ...))` renders as `O(|items| * |jobs|)` (ProductCost of two Linear)

---

## Rationale

**Why asymptotic (not concrete)?** Because the thesis claim is about detecting O(n²) vs O(n). Concrete counts vary with input; symbolic bounds are stable. Users write `where cost_bounded(O(n))` — not concrete numbers.

**Why a closed set of variants (ConstantCost, Linear, Polynomial, Product, Sum, Log, Unknown)?** Because these cover every asymptotic bound the thesis actually reasons about:
- Constant = O(1)
- Linear = O(n)
- Polynomial = O(n^k) for integer k
- Product = O(n * m) (separate size variables)
- Sum = O(n + m) (simplifies via normalization)
- Log = O(log n) for divide-and-conquer
- Unknown = "can't prove" — emits diagnostic with reason

More exotic bounds (O(n!), O(2^n)) are either fatal (divergent) or so specific that a dedicated variant per case is clearer than trying to be general. Add as needed.

**Why `SizeVariable` not just `String`?** Because size variables are DERIVED from substrate facts. `|items|` means "the length of the port `items`'s runtime value." The `source_port: PortId` tracks back to the declaration so the compiler can check that different ops referring to `|items|` are really referring to the SAME port. Two folds over different lists of the same declared type have different size variables.

**Why `UnknownCost(reason)`?** Because some programs genuinely can't be bounded symbolically (reflection, unbounded recursion with non-structural descent). Instead of erroring immediately, report "O(?)" and let the implementer decide whether to pay (accept the unknown) or fix (add structural evidence).

**Why separate Sum and Product?** Because they compose differently and have different dominance rules. `SumCost([O(n), O(n²)])` simplifies to `O(n²)`; `ProductCost([O(n), O(n²)])` stays `O(n³)`. Encoding the operator structurally lets normalize do its job.

**Why log is its own variant, not `PolynomialCost(v, 0)` or similar?** Because `log(n)` dominates constants but is dominated by linear; it's on a separate rung. Encoding it as its own variant makes dominance rules clean.

---

## Rejected alternatives

**Single `AsymptoticExpression` string like `"O(|items|^2)"`** — opaque; can't be composed. Every consumer reparses. Rejected.

**Big-O as a separate type class with instances per shape** — overkill. Closed coproduct does the job cheaper. Rejected for now.

**Polynomial with a `Polynomial` struct (coefficients per degree)** — loses the "asymptotic dominant term" abstraction. `O(n² + n)` collapses to `O(n²)`; keeping full polynomials is more precision than needed and harder to compare.

**Track exact constants (O(3n)) not just orders** — not what the thesis proves. Asymptotic is the target. Rejected.

---

## Implementation notes

### Recognizing list-builtins

The `target_is_list_builtin(TransformTarget)` helper identifies whether a Transform's callable is one of `fold`, `map`, `filter`, `sort`, etc. — needs to:
1. Resolve the callable's `DeclarationId` to the std.list declaration
2. Match against a known closed set

This replaces the `v.label == "fold"` string-matching antipattern with a structural lookup.

### Recursion depth bounds

`recursion_depth_bound(d, l)` on a `Loop` node walks the structural-descent evidence to extract the bound variable. If the loop descends on a list: depth = `|list|`, so LinearCost. If the loop descends on a numeric variable with known upper bound: depth = that constant. This reuses Lane 3 Stage 3a's termination evidence work.

### Integration with existing cost lens

`complexity.dag` (structural cost, Lane 1a) stays as-is. `symbolic_cost.dag` (this design, Lane 2d) is a separate lens. Both can run; they report different things. Existing tests (`kf_1_*`) against structural cost keep passing. The `#[ignore]`d `kf_1_lambda_body_cost_contributes_to_fold` unignores when the symbolic cost lens reports correctly attributed lambda cost.

---

## Associations

- **Lane 2 Stage 2d** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — this is that stage's design
- **DB-3 `Dimension` abstraction** ([design-dimension-abstraction.md](./design-dimension-abstraction.md)) — symbolic cost is a Dimension instance
- **DB-1 `Correction` shape** ([design-correction-shape.md](./design-correction-shape.md)) — diagnostics carry fixes
- **`src/v3/lenses/complexity.dag`** (existing) — structural cost stays; symbolic cost is additive
- **Create `src/v3/lenses/symbolic_cost.dag`** — new file
- **Unignores `kf_1_lambda_body_cost_contributes_to_fold`** in m1_3_lens_cost_test.rs
- **Thesis anchor** — THESIS.md §"KF-1: Complexity — symbolic bounds (NOT YET IMPLEMENTED, L2 M1)"

---

## Acceptance (Lane 2 Stage 2d owns)

- [ ] `src/v3/lenses/symbolic_cost.dag` declares `SymbolicCost` carrier + `symbolic_cost_dimension` Dimension instance
- [ ] `kf_1_lambda_body_cost_contributes_to_fold` unignored and passing (symbolic cost correctly attributes lambda body × N iterations)
- [ ] Fixture: thesis-doc `all_pairs` example reports `O(|items|²)`
- [ ] Fixture: `sort_by before fold(+)` emits dead-work diagnostic
- [ ] Dominance / normalization tested: `SumCost([LinearCost, ConstantCost])` normalizes to `LinearCost`
- [ ] `render(cost)` produces the thesis-doc-style `O(|items|²)` strings

---

## Open questions

1. **How many size variables can a single program have?** Probably unbounded (every list in scope). Normalization treats them as independent; dominance within one size var is the common case. Confirmed OK.

2. **Do we model memoization?** If a pure function is called twice with the same input, concrete cost is 2x, symbolic is still "one invocation." Skip for now; defer to a memoization-aware follow-up lens.

3. **Do we want a fail-compile option for "cost exceeds declared bound"?** Yes: `where cost_bounded(O(n))` on a function declaration forces `UnknownCost` or worse-than-declared to fail compile. This is a downstream extension; the lens reports, the declaration enforces. Initial lens just reports.

4. **How does symbolic cost handle Branch where paths have different asymptotic cost?** Current design: `max_path` over the path costs returns the dominant one. If a user branches between O(n) and O(n²), the branch's cost is O(n²) (worst case). Correct for worst-case asymptotic reporting.
