# Cost Lens — SizeVar Value Semantics + Dimension<SymbolicCost> Wiring

> Part of: [`docs/r3-structure.md`](r3-structure.md) row 146 (T-Lens-Behavioral-Parity slice 2 — cost), [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) `cost.dag` row, [`docs/design-symbolic-cost-algebra.md`](design-symbolic-cost-algebra.md) (DB-7), [`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md) (DB-3), [`../INVARIANTS.md`](../INVARIANTS.md)
>
> **Purpose:** specify the substrate-shape upgrades that take `src/v3/lenses/cost.dag` from BEHAVIORALLY PROXY to BEHAVIORALLY COMPLETE. Three moves: (1) `SizeVariable.display_name: String?` field add — single substrate authority for the user-facing name (per §1.2; v3 has no InternTable name-lookup query, so the substrate-field path is the single authority); (2) `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost>` materializes per DB-3 once grammar gaps close; (3) lens consumes the same per-call `DescentEvidence` / `CallPattern` / `SubValueRelation` producer foundation as complexity, with a cementing TestClaim against the v2 oracle.
>
> **Authority discipline:** this is an R3 design doc; the implementation lane is **T-Lens-Behavioral-Parity slice 2 (cost)** under Substrate Manager + Verification Manager (cross-program). This doc resolves the design questions blocking that slice's worker dispatch.

## What this document is

[`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) row for `cost.dag` reads:

> `cost.dag` | TERMINAL | **PROXY** | … | Named `SizeVar` with value semantics (v3's `SizeVariable` carries only `source_port: PortId`). `Dimension<SymbolicCost>` wiring deferred on grammar gaps. **E-I carriers are present and E-P has a first narrow producer slice;** broader producer coverage, cost/complexity lens consumption, and the same-source v2/v3 cementing test remain pending.

Three concrete debts surface from that row:

1. **`SizeVariable` lacks user-facing name** — v3's `SizeVariable { source_port: PortId }` (in [`src/v3/std/algebra.dag`](../src/v3/std/algebra.dag)) carries structural identity but no user-facing label for diagnostic rendering. v2's `SizeVar { name: String }` rendered "O(|items|)" against the user's binding name; v3 currently has no surface label until an InternTable name resolution lands. (The other v2 `SizeExpr` facets — SizeAdd/SizeMax — are already expressible via DB-7's `SymbolicCost` algebra: `SumCost`, the dominance ordering. Descent semantics like `n - 1` live in `std.computation::CallPattern`, not in size expressions. See §1.1 for the per-facet mapping.)
2. **Dimension<SymbolicCost> data declaration deferred** — [`src/v3/lenses/cost.dag`](../src/v3/lenses/cost.dag) lines 264–305 explicitly mark the `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost> = { … }` declaration as blocked on class-5 grammar gaps (record literals inside `data` bodies + `fn X { body }` block-bodied definitions with structural carriers). The Rust trampoline `v3_compiler::analyze_symbolic_cost_dimension` carries the execution today; the `.dag`-as-authority migration waits on grammar.
3. **Producer foundation not consumed** — `src/v3/std/induction.dag::SubValueRelation` + `std.computation::CallPattern` + `v3_compiler::dag::per_call_descent_evidence` (the E-I + E-P producers) are staged but the cost lens does not yet consume them at call sites. Recursive bodies receive a structural-depth proxy, not a recurrence-derived bound.

This document specifies the substrate shape and the lane-internal sequencing that resolves all three. It is the cost-slice analogue of [`docs/design-lens-application-surface.md`](design-lens-application-surface.md): a design spec whose substrate carriers + closure gates feed back into the lane row's checklists.

## §1. SizeVariable display-name enrichment — substrate-shape upgrade

### §1.1 The current shape and what it loses

```dag
// src/v3/std/algebra.dag:146-148 (current)
type SizeVariable {
  source_port: PortId
}
```

Two cost expressions correlate via `same_size_variable(a, b) = a.source_port == b.source_port`. That suffices for the dominance MVP DB-7 ships. What it loses relative to v2:

- **Named-binding semantics**: v2's `SizeVar { name: String }` lets the diagnostic surface render "O(|items|)" against the user's binding name. v3's port-id-only form has no surface label until an InternTable name resolution is wired (per [`src/v3/std/algebra.dag`](../src/v3/std/algebra.dag) line 144 comment: "When a render consumer pins the missing piece, `SizeVariable` grows a `name` field").

The other v2 facets (size arithmetic, size-max across branches) are NOT lost — they're already expressible in v3:

- **`SizeAdd`** ↔ `SumCost(NonSingletonList<SymbolicCost>)` — already in DB-7's `SymbolicCost`.
- **`SizeMax`** ↔ the dominance ordering already in `src/v3/std/algebra.dag::dominates` — `ProductCost`/`SumCost` of `LinearCost`s with distinct sizes naturally collapse via dominance.
- **Descent semantics ("n-1" recurrence step)** — these are NOT a property of size expressions; they live in `std.computation::CallPattern` (`ArithmeticSubtractCall` for `n-1`, `ArithmeticDivideCall` for `n/2`). The cost lens reads `CallPattern` to derive the recurrence shape; size expressions never carry "n-1" as a shape (`O(n-1) ≡ O(n)` asymptotically; the descent fact is a call-site property, not a size property).

This is the load-bearing single-authority discipline (P2 + DB-7 lock): `SymbolicCost` is the unified algebra; `CallPattern` is the descent-evidence carrier; `SizeVariable` is the size-identity carrier. Three orthogonal facts, three orthogonal carriers — no parallel `SizeExpr` algebra.

### §1.2 Target shape — `SizeVariable.display_name: String?` enrichment (single authority on substrate field)

Two iterations of reviewer feedback shaped this resolution:

1. **First wave** (gpt-5-5-pro at sha ef21e1a0): a `display_name: String?` field PLUS an InternTable name lookup would create two sources of truth.
2. **Second wave** (codex BLOCKING at sha 37f3bc62): the doc previously assumed an `intern_table::name_of(port_id) -> String` query that does NOT exist in v3 substrate. `src/v3/std/algebra.dag:143` explicitly says "InternTable lookup the lens doesn't yet run". v3 has InternTable machinery (per PR #367 Phase 1) but the port-id-to-authored-name query is not wired. Assuming a query that doesn't exist is "lock substrate assumptions the current .dag/v3 surface cannot express".

**Resolution: `SizeVariable.display_name: String?` substrate field is the single authority.** No InternTable lookup query is assumed.

```dag
// target shape — src/v3/std/algebra.dag (single field addition)

// 🟢 TERMINAL. Names the runtime size of a DAG port's value.
// Equality is on `source_port` (structural identity); `display_name`
// is a presentation slot (NOT identity — same source_port with
// different authored names is the same size variable).
//
// CONSTRUCTION INVARIANT: SizeVariable is constructed exclusively by
// the parser at authoring sites, against a per-DAG `PortId → String?`
// name table populated at parse time. Every SizeVariable instance
// referencing source_port `p` carries the canonical display_name for
// `p` (or None if no authored name was given). The parser is the
// single point of construction; SymbolicCost payloads cannot be
// constructed with mismatched display_name values for the same
// source_port. This is a parser-level invariant, not a type-level
// guarantee, but the parser is the only construction site for
// SizeVariable in the substrate (no user-facing `SizeVariable { ... }`
// literal at authoring time — SizeVariables emerge from let-binding
// references during lowering).
type SizeVariable {
  source_port: PortId
  display_name: String?         // canonical display_name keyed by source_port (parser-derived)
}

fn size_variable_eq(a: SizeVariable, b: SizeVariable) -> Bool =
  port_id_eq(a.source_port, b.source_port)
  // display_name comparison would be redundant per the construction
  // invariant — same source_port always carries same display_name.
```

`SymbolicCost` (DB-7 locked) is unchanged — it continues to carry `SizeVariable` in `LinearCost`/`PolynomialCost`/`LogCost` payloads. The change is a single field addition. **Per the construction invariant above, `SymbolicCost` payloads carrying SizeVariable copies cannot have divergent labels for the same source_port**: the parser canonicalizes display_name from the per-DAG name table, so all SizeVariable instances for port_id `p` carry the same display_name (or all None). User-facing labels are single-authority by construction.

**Why `String?`, not `String`**: per [`../INVARIANTS.md`](../INVARIANTS.md) C-9 (no fabrication), when no user-authored name exists the field is `None`; the renderer derives a fresh label (e.g., `"|port_42|"`) from `source_port`. We never invent a fake name and stash it in the field.

**Future hardening**: when v3 lands a substrate-level `port_id → name` query (currently unwired per `src/v3/std/algebra.dag:143`), `display_name` can be retired from the carrier — the renderer would look up the name via the query at render time, eliminating the field entirely. This is a future cleanup; for now the parser-canonicalized field is what v3 supports.

### §1.3 Why the unified `SymbolicCost` algebra is correct (not a parallel `SizeExpr`)

Per `feedback_parallel_representation_debt` and [`../INVARIANTS.md`](../INVARIANTS.md) P2/P5: don't ship two algebras for the same fact-flow. `SymbolicCost` is the unified expression algebra (DB-7 lock); the v2 `SizeExpr ↔ CostExpr` distinction was v2's design accident, not a structural necessity. v3's substrate is correct as-is for the algebra; only the `display_name` field on `SizeVariable` is the missing fact.

A `SizeExpr` coproduct would:
- Duplicate the SumCost/ProductCost/dominance facts already in `SymbolicCost`.
- Bake in an `n - k` shape that asymptotically collapses to `n` (no information gain at the size level).
- Move descent semantics out of `CallPattern` (the canonical site per E-C vocabulary) into a parallel size carrier — same parallel-authority pattern P2 forbids.

The descent-semantics question is answered by the existing `CallPattern` query surface (per §3.2 below): the lens dispatches on `per_call_pattern_at(d, call_site)` to read the recurrence shape; size expressions never need to carry descent.

### §1.4 Migration shape — additive field, no carrier rename

The migration is a single field addition (per §1.2):

1. Add `display_name: String?` to `SizeVariable` in `src/v3/std/algebra.dag`.
2. Update the Rust mirror in `src/v3/compiler/src/dag.rs` (single field add).
3. Wire the parser to populate `display_name` from authored binding names where present (`Some(...)`); leave `None` for inferred sizes.
4. Update render-side surfaces (`Display` impls; `compute_symbolic_costs` rendering) to prefer `display_name` over `port_id`-derived labels.

No new types. No parallel carriers. No deletion. No assumed-but-unlanded substrate queries.

## §2. Dimension<SymbolicCost> wiring — closing the deferred declaration

### §2.1 Current state — the deferral receipt

[`src/v3/lenses/cost.dag`](../src/v3/lenses/cost.dag) lines 264–305 are the authoritative receipt for the deferral. Two grammar gaps named explicitly:

1. **Class-5 gap #3**: record literals inside `data X: T = { … }` bodies. The lowerer reports "data has an opaque body — user code cannot yet use record / list / map literals inside data bodies."
2. **Block-body grammar**: record literals / match / lambdas inside `fn X { body }` definitions. Constructing `Witness::Violates { reason, at }` — the MissingCost-to-violation shim DB-3's `witness_of` contract requires — hits the same restriction.

Today, [`v3_compiler::analyze_symbolic_cost_dimension`](../src/v3/compiler/src/) packages the same `compute_symbolic_costs` algebra as a `DimensionReport<SymbolicCost>` (per [`src/v3/std/dimensions.dag`](../src/v3/std/dimensions.dag)), but the `data` value remains in Rust.

### §2.2 Target shape per DB-3 + DB-7

```dag
// target shape — src/v3/lenses/cost.dag (post-grammar-gap close)

import std.dimensions { AnalysisDimension, Witness, Inhabits, Violates,
                        OptionalDiagnostic, NoDiagnostic, SomeDiagnostic }
import std.algebra { Monoid, SymbolicCost, ConstantCost, sequential_monoid }
import v3.std.diagnostics { Diagnostic, MissingCostKind }

// 🟢 TERMINAL. The cost dimension instance per DB-3.
data symbolic_cost_dimension: AnalysisDimension<SymbolicCost> = {
  name: "symbolic_cost"
  witness_of: symbolic_cost_witness_of   // Behavior → Witness<SymbolicCost>
  compose: sequential_monoid              // Monoid<SymbolicCost>: SumCost op + ConstantCost(0) identity (collapsed from prior compose+identity field pair, F2 / PR #1607)
  break_diagnostic: symbolic_cost_break_diagnostic
}

fn symbolic_cost_witness_of(d: Dag, behavior: Behavior) -> Witness<SymbolicCost> =
  match behavior_cost_lookup(d, behavior) {
    Miss => Violates {
      reason: "missing per-port cost in compute_symbolic_costs"
      at: behavior
    }
    Hit(cost) => Inhabits(cost)
  }

fn symbolic_cost_break_diagnostic(behavior: Behavior, composed: SymbolicCost) -> OptionalDiagnostic =
  match composed {
    UnknownCost(reason) => SomeDiagnostic {
      value: cost_unknown_diagnostic(behavior, reason)
    }
    _ => NoDiagnostic
  }
```

The four monoid-shape obligations (`compose.op`, `compose.identity`, monoid laws, break-diagnostic semantics — collapsed onto `compose: Monoid<SymbolicCost>` per F2 / PR #1607) are satisfied:

- **Identity**: `ConstantCost(0)` is the additive identity; `sequential(ConstantCost(0), c) = normalize(SumCost([0, c])) = c` after `drop_zero` strips the zero (per `src/v3/std/algebra.dag::drop_zero`).
- **Associativity**: `sequential(sequential(a, b), c) = SumCost([SumCost([a, b]), c])` and `sequential(a, sequential(b, c)) = SumCost([a, SumCost([b, c])])`. After `normalize` flattening (see §4 below for the full-flattening dependency), both produce the same flat sum modulo order. Currently `normalize` does not fully flatten nested sums; that gap is a separate landed-in-this-slice obligation.
- **Commutativity** (NOT required for general monoid; required for cost): `sequential(a, b)` versus `sequential(b, a)` produce the same `SumCost` *after* `drop_dominated` runs. This holds for `SymbolicCost` because `drop_dominated` is order-insensitive (it walks pairwise comparisons against the dominance lattice).
- **Break-diagnostic semantics**: `UnknownCost(reason)` in the composed result fires `SomeDiagnostic`; everything else returns `NoDiagnostic`. This matches DB-3's "diagnostic when composition breaks" contract — `UnknownCost` is the "the analyzer could not prove a tighter bound" failure mode (per `src/v3/std/algebra.dag` lines 22–33).

### §2.3 Why this lands now (versus continuing to defer)

The grammar gaps the deferral receipt names are not blockers for the *cost* slice specifically; they are blockers for the *uniform* `data … : AnalysisDimension<C>` declaration shape across all dimensions. Two paths:

- **Path A (uniform-grammar-first)**: wait for class-5 record-body lowering across all `data` declarations. Then every dimension lands its `data` value at once.
- **Path B (cost-slice-first)**: declare a narrow dispatch surface inside `lenses/cost.dag` that constructs the `AnalysisDimension<SymbolicCost>` value via fn-level builders (no record literal in `data` body), and migrate when class-5 lands.

Path B is the bridge-as-steady-state pattern (per [`../INVARIANTS.md`](../INVARIANTS.md) P5 + `feedback_dissolve_bridges`). **This design selects Path A** with a sequencing constraint: cost slice 2 ships its substrate carriers (SizeVariable.display_name field add + producer-consumption rewiring) and the closure-gate cementing test, but the `data symbolic_cost_dimension` declaration itself lands as part of T-Tests-As-Data-Completeness's grammar-gap retirement (since the grammar gap is shared across multiple lanes — record literals in `data` bodies block Dimension instances for parallelism, idempotency, effect_enumeration, AND user-declared dimensions per Lane 2 Stage 2f).

The cost lane's closure gate `cost_lens_behaviorally_complete` is therefore satisfied by:

1. SizeVariable.display_name field add landed.
2. Producer-foundation consumption landed (per §3 below).
3. Cementing TestClaim against v2 oracle landed (per §5 below).
4. **Not** the `data symbolic_cost_dimension` declaration landing — that landing is `cost_dimension_data_declaration_landed`, scoped to the grammar lane, and tracked separately. The Rust trampoline `analyze_symbolic_cost_dimension` remains the dispatch surface in the interim *with a named dissolution trigger* (per [`../INVARIANTS.md`](../INVARIANTS.md) P5 scaffold-discipline): "when class-5 record-body grammar lands, this trampoline retires."

This is not a regression vs the old comment in `cost.dag` — it makes the dissolution trigger explicit and per-gate.

## §3. Producer foundation shared with complexity

### §3.1 The shared E-I + E-P substrate

Per [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) §"Substrate carriers vs per-call producers":

- `std.termination::DescentEvidence` (Strict | NonIncreasing | DescentUnknown) — staged at `src/v3/std/termination.dag`.
- `std.computation::CallPattern` (ChildAccessorCall | FoldBodyCall | SameArgumentCall | ArithmeticSubtractCall | ArithmeticDivideCall) — staged at `src/v3/std/computation.dag`.
- `std.induction::SubValueRelation` (PreservedValue | StrictSubValue | IteratedSubValue | ArithmeticDescent | SubValueUnknown) — staged at `src/v3/std/induction.dag`.
- `v3_compiler::dag::per_call_descent_evidence` — first E-P producer slice covering recursive self-call + arithmetic-descent at call sites.

The complexity lens (slice 1 of T-Lens-Behavioral-Parity) consumes these to compute work/span dimensions and asymptotic class. The cost lens (slice 2) consumes the *same* facts, projected through a different fold:

| Per-call fact | Complexity lens consumes as | Cost lens consumes as |
|---|---|---|
| `DescentEvidence::Strict` | "this call descends; recurrence terminates" | same — terminates the cost recurrence |
| `CallPattern::ArithmeticSubtractCall { factor: UnitShrink }` | "`n → n-1` recurrence; depth = n" | `iterate(LinearCost(SizeVariable { source_port: arg }), body_cost)` — n iterations of body |
| `CallPattern::ArithmeticDivideCall { factor: ProportionalShrink(2) }` | "`n → n/2` recurrence; depth = log n" | `iterate(LogCost(SizeVariable { source_port: arg }), body_cost)` — log n iterations of body |
| `SubValueRelation::IteratedSubValue { field }` | "iterates over a list field; depth = |list|" | `iterate(LinearCost(SizeVariable { source_port: field }), body_cost)` |

The shared consumption pattern is one function per call site that takes the per-call fact and returns a `SymbolicCost`:

```dag
fn call_pattern_to_iter_bound(pattern: CallPattern, arg_port: PortId) -> Lookup<SymbolicCost> =
  match pattern {
    ArithmeticSubtractCall { factor: UnitShrink } =>
      hit_symbolic_cost_lookup(LinearCost(SizeVariable { source_port: arg_port }))
    ArithmeticSubtractCall { factor: ConstantShrink(_) } =>
      hit_symbolic_cost_lookup(LinearCost(SizeVariable { source_port: arg_port }))
    ArithmeticDivideCall { factor: ProportionalShrink(_) } =>
      hit_symbolic_cost_lookup(LogCost(SizeVariable { source_port: arg_port }))
    ChildAccessorCall { … } =>
      hit_symbolic_cost_lookup(LinearCost(SizeVariable { source_port: arg_port }))
    SameArgumentCall =>
      miss_symbolic_cost_lookup()  // not a recurrence — same arg means no descent
    FoldBodyCall { … } =>
      hit_symbolic_cost_lookup(LinearCost(SizeVariable { source_port: arg_port }))
  }
```

### §3.2 Where the consumer attaches in the lens

The cost lens's existing `entry_for` dispatch on `Behavior` (in [`src/v3/lenses/cost.dag`](../src/v3/lenses/cost.dag) lines 102–118) gets one new case for the recursive-call sub-case of `Transform`:

```dag
// extension to entry_for
Transform(t) =>
  match per_call_pattern_at(d, t.call_site) {
    None =>
      // base case — non-recursive transform; existing behavior
      { port: t.result_port, cost: transform_cost(acc, t.inputs) }
    Some(pattern) =>
      // recursive call — use the per-call fact to derive the recurrence cost
      { port: t.result_port, cost: recursive_transform_cost(acc, t, pattern) }
  }
```

`per_call_pattern_at(d: Dag, call_site: NodeId) -> CallPattern?` is the structural-query surface (per [`../INVARIANTS.md`](../INVARIANTS.md) L-7: lenses consume declared substrate query functions). The lens does *not* reach into `per_call_descent_evidence` storage directly; the substrate exposes `per_call_pattern_at` as the typed query.

### §3.3 Why complexity and cost share producer wiring

This is exactly the pattern row 146 names: "same producer foundation as complexity." Both lenses are `Lens<C>` instances per R2-T-Substrate-Lens-Primitive — complexity with `C = AsymptoticClass`, cost with `C = SymbolicCost` — folding over the same Behavior structure with the same per-call evidence. The ergonomic move: factor the per-call query into `std.computation::per_call_pattern_at` (already implied by the staged producer side-table); both lenses become consumers.

**Cross-design compatibility — resource-threaded signatures**: per [`docs/design-effect-enumeration-resource-threading.md`](design-effect-enumeration-resource-threading.md) §2.4, callable signatures thread their resources explicitly (e.g., `read(fs: Filesystem, path) → (fs: Filesystem, content)`). `per_call_pattern_at` reads its evidence from the *threaded arrow signature* (not retired ambient transport metadata). The producer broadening per T-E-P-Producer-Broadening covers both pre-resource-threading signatures (current state) and post-threading signatures (post-effect-enumeration migration); the cost lens's consumption of `CallPattern` is signature-shape-agnostic — it reads only the descent-evidence facts, which derive from substrate-level recursion structure rather than transport identity.

Per [`docs/design-lens-application-surface.md`](design-lens-application-surface.md) §1.2 (the section-ref pattern): each lens has its own budget type (`AsymptoticClass` vs `SymbolicCost`) but consumes the same structural facts. The substrate authority is the per-call query surface; the per-lens algebra is private.

## §4. SymbolicCost commutative-semiring discipline (product-zero bug class)

### §4.1 The bug class and why it surfaces here

Per `docs/r3-structure.md` §"Plus 3 fold-ins":

> **T-V-L4-L7-Direct exhaustive witness coverage** — per-(algebra, inhabitant, law) witness coverage ensures bug class like SymbolicCost product-zero (PR #1430 §A) is structurally caught by `l7_algebraic_laws_witnessed` gate at all inhabitants.

The bug class: `SymbolicCost`'s `normalize` does not satisfy the *annihilation law* of a commutative semiring, `a * 0 = 0`. Concretely, `normalize(ProductCost([LinearCost(n), ConstantCost(0)]))` does not collapse to `ConstantCost(0)` because `drop_zero` (`src/v3/std/algebra.dag` lines 416–424) strips zero terms from *both* `Sum` and `Product` lists with the *same* function — but the semiring distinguishes:

- **Sum identity (`ConstantCost(0)` in `SumCost`)**: drop the zero. `O(n) + 0 = O(n)`. Correct.
- **Product annihilator (`ConstantCost(0)` in `ProductCost`)**: collapse the *whole product* to zero. `O(n) · 0 = 0`. Currently broken — `drop_zero` strips the zero from the product list, leaving `LinearCost(n)`, which is wrong.

The bug is two distinct semiring laws sharing one helper because `SymbolicCost` was modeled as a single coproduct without declaring its semiring inhabitance.

### §4.2 The fix — declare `SymbolicCost` as `Semiring<SymbolicCost>`

Per [`../INVARIANTS.md`](../INVARIANTS.md) P1 Step 1 (DAG-ancestor check): the fix is not to special-case the helper but to declare the algebra structure that mandates the right semantics. `Semiring<T>` already exists at `dsl/std/algebra.dag:190-195`:

```dag
type Semiring<T> {
  add: fn(T, T) -> T
  zero: T
  mul: fn(T, T) -> T
  one: T
}
```

The fix:

1. Declare a `Semiring<SymbolicCost>` instance with:
   - `zero = ConstantCost(0)`
   - `add = sequential` (already exists)
   - `one = ConstantCost(1)`
   - `mul = iterate` (already exists, but renamed below)
2. Rename `iterate` → `multiply_costs` to match the algebra-field semantics. `iterate` is a domain operation (loop iteration); `multiply_costs` is the algebra primitive that `iterate` becomes a special case of.
3. Replace `drop_zero` with two separate helpers: `drop_additive_zero` (for `SumCost` lists) and `collapse_on_multiplicative_zero` (for `ProductCost` lists; if any term is `ConstantCost(0)`, the whole product is `ConstantCost(0)`).
4. Update `reduce_product` to call `collapse_on_multiplicative_zero` first.

The shape:

```dag
fn collapse_on_multiplicative_zero(terms: List<SymbolicCost>) -> List<SymbolicCost> =
  if any_zero(terms) then
    cons(ConstantCost(0), empty())   // single-element list — reduce_product unwraps to ConstantCost(0)
  else
    terms

fn any_zero(terms: List<SymbolicCost>) -> Bool =
  match terms {
    Empty => False
    Cons(payload) => is_zero_constant(payload.head) || any_zero(payload.tail)
  }
```

Then `reduce_product`'s top:

```dag
fn reduce_product(terms: List<SymbolicCost>) -> SymbolicCost =
  let collapsed = collapse_on_multiplicative_zero(terms) in
  match collapsed { … existing match … }
```

### §4.3 Why this is in the cost-lens design doc (and not a separate algebra fix)

The cementing TestClaim against v2 oracle (§5 below) will fail today on programs containing dead-product surfaces (`fold(items, 0, |acc, x| 0 * acc) ≡ O(1)` per v2; v3 reports `O(n)`). The fix has to land *inside* the cost-lens behavioral-parity slice or the cementing test cannot pass. The structural fix (declare semiring inhabitance) is the right shape; the slice-inline patch (rewrite `drop_zero`) without declaring the algebra would be a heuristic by [`../INVARIANTS.md`](../INVARIANTS.md) P1.

The semiring declaration is also load-bearing for `feedback_lattice_consolidation` ("ad-hoc lattices need to dissolve into BoundedLattice declarations") — the symbolic-cost case is the same pattern at a different algebra. Declaring the inhabitance makes the laws structurally checkable at L7-witnessing time per the T-V-L4-L7-Direct fold-in.

### §4.4 Operator-dispatch fold-in

Per `project_algebra_operator_dispatch` (memory): "BinOp emission reads operand type algebra for correct symbol." Once `Semiring<SymbolicCost>` is declared, code that writes `a + b` against two `SymbolicCost` values should resolve through the Semiring's `add` field, picking up `sequential` rather than asking the consumer to spell `sequential(a, b)`. This is downstream of slice 2 (it requires v3's `+` operator dispatch to walk algebra inhabitance for non-numeric types) but the substrate fact landing here is what unblocks it. No code-change in slice 2 for this; the design captures the linkage.

## §5. Cementing TestClaim against v2 oracle

### §5.1 The TestClaim shape

Per row 146 closure language ("cementing test against v2 oracle on same source") and per [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) discipline ("Every claim of 'v3 replaces v2 X' requires a behavioral cementing test"), the cost slice ships:

Cementing-test format aligns with sibling lens designs (complexity §4, effect-enumeration §5.3): **Rust cementing test today**, lives at `src/v3/compiler/tests/integration/cementing/cost_v2_v3_oracle_test.rs`, registered via the existing `cementing_test_modules_exist_for_escalated_v2_complete_registry_claims` ratchet. Same Band-C discipline as complexity-lens cementing.

**Dissolution trigger**: at T-Tests-As-Data-Completeness step 5 (per [`docs/design-tests-as-data-completeness.md`](design-tests-as-data-completeness.md) §6 step 5 — *cementing dispatch port*), this Rust cementing test ports to a `.dag` `TestClaim`/`QuantifiedTestClaim` declaration alongside the lens-capability register migration. Per the cross-lane sequencing in tests-as-data §8.3, Rust cementing is the staged form; the .dag port lands together for all lenses when the register migrates. Cost-lens shipping its closure gate does NOT block on the migration; the Rust cementing test is the per-PR receipt today.

The Rust test runs the v3 lens on a fixture and the v2 oracle on the same fixture (`src/v2/complexity.dag::analyze`), asserting **structural equivalence on the `SymbolicCost`/`CostExpr` carrier** (the published Band-C parity claim — full carrier shape, not a projection). Asymptotic-class projection is checked separately as a downstream consequence (when both expressions normalize to the same `SymbolicCost`/`CostExpr` shape, their `AsymptoticClass` projections necessarily agree); the cementing test does NOT permit asymptotic-class equivalence as a substitute for full-carrier equivalence — that would be a documented homomorphism, but this test commits to the stronger claim.

Where `structural_equivalent(a: SymbolicCost, b: CostExpr) -> Bool` is the v2/v3 isomorphism test — both expressions are normalized then walked structurally:

```dag
fn structural_equivalent(v3: SymbolicCost, v2: CostExpr) -> Bool =
  match (normalize(v3), v2_simplify(v2)) {
    (ConstantCost(k_v3), CostConst { value: k_v2 }) => k_v3 == k_v2
    (LinearCost(SizeVariable { source_port: p }), CostMul { left, right }) =>
      // v2 spells O(n) as `n * 1` after simplify; check the structural shape matches
      … structural walk …
    … // all (variant, variant) pairs covered exhaustively
    _ => False  // any mismatched shape fails
  }
```

The Rust cementing test runs the v3 lens on `compile_v3(p)` and the v2 oracle on `compile_v2(p)` (where `compile_v2` invokes `src/v2/complexity.dag::analyze`) for every fixture in the corpus, asserting structural equivalence on the **full `SymbolicCost`/`CostExpr` carrier shape** (per the Band-C parity claim at line 316 — not a projection). Test fails on first non-equivalent fixture; the failure surfaces which fixture diverged and where in the structural walk the mismatch occurred.

### §5.2 Corpus shape

Per [`docs/design-lens-application-surface.md`](design-lens-application-surface.md) §4 (4 worked examples per Director ratification on orthogonal axes), the cost cementing test ships a corpus covering:

1. **Constant cost**: `let x = 1` → `O(1)` on both. Smoke test for the trivial-equivalence case.
2. **Linear cost via fold**: `fold(items, 0, +)` → `O(|items|)` on both. Smoke test for SizeVariable reading.
3. **Polynomial via nested folds**: `fold(items, 0, |acc, x| acc + fold(items, 0, +))` → `O(|items|²)` on both. Smoke test for `combine_linear_with` collapsing two LinearCosts on the same port to PolynomialCost.
4. **Logarithmic via halving recursion**: a recursive function with `ArithmeticDivideCall { factor: ProportionalShrink(2) }` → `O(log n)` on both. Smoke test for the producer-foundation E-P consumption.
5. **Unrelated sizes**: `fold(items, 0, |acc, x| acc + fold(jobs, 0, +))` → `O(|items| · |jobs|)` on both. Smoke test for distinct `SizeVariable.source_port` correlation.
6. **Branch-max**: `if cond then fold(items, 0, +) else fold(items, 0, |acc, x| fold(items, …))` → `O(|items|²)` on both (max of branches). Smoke test for `max_path` and dominance.
7. **Annihilation law**: `fold(items, 0, |acc, x| 0 * acc)` → `O(1)` on both. Smoke test for the §4 product-zero fix.
8. **Identity-on-empty**: empty body → `ConstantCost(0)` on v3, `CostConst { value: 0 }` on v2. Smoke test for AnalysisDimension identity.

This corpus is *not* exhaustive over the cost algebra — exhaustiveness comes from L7 algebraic-law witnessing (per the T-V-L4-L7-Direct fold-in). The cementing test demonstrates equivalence on a representative spread of cost shapes; the L7 witnesses prove the laws hold for all inhabitants.

### §5.3 Where the corpus lives

Per [`../INVARIANTS.md`](../INVARIANTS.md) P2 (every fact in one place) and `feedback_no_generated_code_on_disk`: the corpus lives as `.dag` source files under `src/v3/test/cost_lens_corpus/` with a `TestClaim` referencing them via `DeclarationId`. Both `compile_v3` and `compile_v2` consume the same source (v2 reads `.dag` directly; v3 reads `.dag` directly through the v3 parser). The cementing test does not maintain parallel v2 and v3 fixtures.

This avoids the parallel-representation debt named in `feedback_parallel_representation_debt` — the corpus is one set of `.dag` programs, not two.

## §6. Cross-program coordination

This slice is **cross-program** between Substrate Manager and Verification Manager (per [`docs/r3-structure.md`](r3-structure.md) row 146):

- **Substrate Manager owns**: the SizeVariable.display_name field add in `src/v3/std/algebra.dag` (per §1.2; aligned single-authority with complexity-lens design); the `Semiring<SymbolicCost>` declaration and `collapse_on_multiplicative_zero` fix; the `per_call_pattern_at` typed query surface (or its equivalent — already implied by the E-P side table); the consumer rewiring inside `src/v3/lenses/cost.dag`.
- **Verification Manager owns**: the `cost_lens_v2_oracle_equivalence_demonstrated` TestClaim corpus + structural-equivalence harness; coordination with T-V-L4-L7-Direct on the per-(algebra, inhabitant, law) witness coverage that catches the product-zero class of bugs structurally.

The split mirrors slice 1 (complexity) — Substrate authors carriers + lens consumption; Verification authors the cementing TestClaim.

## §7. Cascade gates

Per [`docs/r3-structure.md`](r3-structure.md):

- **Internal cascade (within T-Lens-Behavioral-Parity)**: cost slice 2 cannot dispatch until T-E-P-Producer-Broadening (the foundational lane that broadens producer coverage from the first narrow slice to full `ExprCall.descent_evidence` parity) is COMPLETE. Reason: per-call `CallPattern` lookups must be authoritative for every recursive call site, not just the first slice. If the producer is partial, the lens consumer falls back to `UnknownCost` for uncovered call sites, the cementing test against v2 fails on those programs, and slice 2 cannot close.
- **Internal cascade (within slice 2)**: SizeVariable.display_name field add lands first; producer-consumption rewiring lands second (consumes the `SizeVariable` shape); cementing test lands third (depends on both). The §4 product-zero fix is ordering-independent and can land in parallel.
- **External cascade**: R2-Evaluator landed (per the standard R3 worker-dispatch precondition); R2-T-Substrate-Lens-Primitive landed (the `Lens<C>` framework cost.dag inhabits).

Pre-cascade design-doc work is permitted (this doc); pre-cascade substrate work waits.

## §8. Resolved design questions

Six design questions surfaced during authoring. Per `feedback_design_before_implement`, each is resolved here.

### §8.1 Parallel `SizeExpr` algebra vs unified `SymbolicCost` — RESOLVED: unified `SymbolicCost`, no parallel `SizeExpr`

**Question:** introduce a parallel `SizeExpr` coproduct (5 variants: SizePort/SizeConst/SizeAdd/SizeMax/SizeShrink) alongside the existing `SymbolicCost`, or stay with the unified `SymbolicCost` algebra without the parallel coproduct?

**Resolved:** unified `SymbolicCost` (DB-7 lock); no parallel `SizeExpr`. Single-authority discipline (P2). The user-facing-name fact lives on `SizeVariable.display_name: String?` (per §1.2 — v3 has no InternTable name-lookup query, so the substrate-field path is the single authority); no parallel `SizeExpr` carrier.

**Why:** v2's `SizeExpr ↔ CostExpr` distinction was a v2 design accident. v3's `SymbolicCost` already covers SizeAdd (via `SumCost`) and SizeMax (via the `dominates` ordering). Descent semantics like `n - 1` live in `std.computation::CallPattern` (the canonical E-C site per existing substrate), not in size expressions — a `SizeShrink` variant would duplicate that fact in a parallel carrier. Asymptotically `O(n - 1) ≡ O(n)`; the descent fact is a call-site property, not a size-shape property. Per `feedback_parallel_representation_debt` and DB-7 lock: don't ship two algebras for the same fact-flow.

### §8.2 Names on `SizeVariable` — RESOLVED: `display_name: String?` field; single substrate authority

**Question:** how should `SizeVariable` carry user-facing names for diagnostic rendering?

**Resolved:** add `display_name: String?` field to `SizeVariable`; the field is the single substrate authority for the user-facing name. Single-authority discipline (P2): one carrier, one source.

**Why `display_name: String?`, not InternTable lookup**: v3 does NOT have a `intern_table::name_of(port_id) -> String` query landed (per `src/v3/std/algebra.dag:143` "InternTable lookup the lens doesn't yet run"). v3 has some InternTable machinery (per `project_intern_table` memory + PR #367 Phase 1) but the port-id-to-authored-name query specifically is not wired. Assuming an unlanded query would lock substrate-target discipline. The structural-field path is what v3 currently supports.

**Why `String?`, not `String`**: per [`../INVARIANTS.md`](../INVARIANTS.md) C-9 (no fabrication), when no user-authored name exists the field is `None`; the renderer derives a fresh label (e.g., `"|port_42|"`) from `source_port`. Never invent a fake name and stash it in the field.

**Implementation note:** the renderer reads `display_name` directly from the carrier; equality on `SizeVariable` is on `source_port` only (`display_name` is a presentation slot, not identity — same `source_port` with different authored names is the same size variable).

### §8.3 `Dimension<SymbolicCost>` declaration scope for slice 2 — RESOLVED: Path A, declaration deferred to grammar lane

**Question:** does slice 2 ship the `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost> = { … }` value, or does it wait for the class-5 grammar gap to close?

**Resolved:** wait for the grammar gap. Slice 2's closure gate `cost_lens_behaviorally_complete` is defined to NOT require the `data` declaration landing; the dissolution trigger is named explicitly (`when class-5 record-body grammar lands, the trampoline retires`), and the trampoline `analyze_symbolic_cost_dimension` is preserved with that explicit dissolution trigger per [`../INVARIANTS.md`](../INVARIANTS.md) P5 scaffold-discipline.

**Why:** the grammar gap is shared across multiple Dimension instances (idempotency, parallelism, effect_enumeration, user-declared dimensions per Stage 2f). Solving it in slice 2 only would either duplicate the work (per-lane gap fix) or defer the cross-lane fix to a later lane that has to redo it. The clean structural answer is: grammar work happens once, then every dimension's `data` value lands.

The cementing test (§5) does NOT require the `data` declaration — it dispatches through `analyze_symbolic_cost_dimension` directly, treating that Rust function as the interim authority *named with its dissolution trigger*. When the trampoline retires, the cementing test rebinds to the `data` declaration; no test rewrite, just the analyze entrypoint changes.

### §8.4 Producer query surface name — RESOLVED: `per_call_pattern_at(d, call_site) -> CallPattern?`

**Question:** what's the typed query surface name for the per-call evidence the lens consumes?

**Resolved:** `per_call_pattern_at(d: Dag, call_site: NodeId) -> CallPattern?` exposed from `std.computation`. Following the same shape as `node(d: Dag, NodeId) -> Behavior?` (the existing substrate accessor — see `src/v3/lenses/cost.dag` line 157).

**Why:** `Option`-typed query surfaces are the established pattern in v3's substrate (per L-7). The query returns `None` for non-recursive call sites, `Some(pattern)` for recursive ones; the consumer dispatches on the option. This avoids fail-closed-panic for the common non-recursive case while making the per-call presence/absence structurally typed.

**Cross-cut:** complexity slice 1 consumes the same query. The shared query surface IS the "same producer foundation" of row 146. No per-lens query duplication.

### §8.5 SymbolicCost semiring declaration shape — RESOLVED: `Semiring<SymbolicCost>`, not `CommutativeSemiring<SymbolicCost>`

**Question:** declare `SymbolicCost` as `Semiring` (left-distributive only) or `CommutativeSemiring` (multiplication commutative)?

**Resolved:** `Semiring<SymbolicCost>` — the multiplicative side does NOT enforce commutativity in the type. Practical reason: `iterate(bound, body)` semantically means "body runs `bound` times in sequence"; reordering to `iterate(body, bound)` semantically means "bound runs `body` times" — different operationally. For asymptotic-class purposes the orders coincide (`O(n) · O(m) = O(n · m)` either way), but the substrate should not promise more than the operational shape supports.

**Why:** the v2 oracle's `CostMul` is *also* not commutative-typed; declaring v3's algebra as commutative would introduce a parallel-authority claim on the algebra structure that v2 does not make. Per [`../INVARIANTS.md`](../INVARIANTS.md) P1, the algebra declaration should match the operational fact.

**Note:** for the asymptotic-equivalence cementing test, `dominates(O(n)·O(m), O(m)·O(n))` returns true in both directions (per `src/v3/std/algebra.dag::dominates` ProductCost branch — "any child dominates" symmetry). The cementing test's structural-equivalence harness reflects this — it normalizes both sides before comparing.

### §8.6 Cementing-test corpus authoring location — RESOLVED: `src/v3/test/cost_lens_corpus/`, single `.dag` source serves both v2 and v3

**Question:** corpus as parallel v2 + v3 fixtures, or single source consumed by both?

**Resolved:** single `.dag` source under `src/v3/test/cost_lens_corpus/`. Both v2 and v3 parse and analyze the same files.

**Why:** parallel fixtures are parallel-representation debt (per `feedback_parallel_representation_debt`); they drift. The `.dag` parsers in v2 and v3 are both authoritative on the same source language; the cementing test exploits exactly that — same source, two analyzers, structural equivalence.

**Implementation note:** v2's parser is `src/v2/parser/...` (existing); v3's is `src/v3/compiler/src/parse.rs`. The cementing-test harness calls both on the same `.dag` file path.

---

All six questions resolved. Implementation can proceed without further Director ratification. Cascade gates (T-E-P-Producer-Broadening COMPLETE for §3 consumption; R2-Evaluator landed for §5 test execution) and external dependencies (class-5 grammar gap for §2.3 `data` declaration deferred to a different lane) remain as the only outstanding preconditions on the *full* cost-lens behavioral parity; the slice-2 closure gate is satisfied without those.

## §9. Relationship to existing authority

This design doc extends:

- [`docs/design-symbolic-cost-algebra.md`](design-symbolic-cost-algebra.md) (DB-7) — the SymbolicCost coproduct shape. **No payload changes**: `LinearCost`/`PolynomialCost`/`LogCost` continue to carry `SizeVariable` (DB-7 lock preserved). This design adds one field to `SizeVariable` (`display_name: String?` per §1.2) and adds the `Semiring<SymbolicCost>` inhabitance declaration.
- [`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md) (DB-3) — the `Dimension<C>` framework. **No changes to existing carrier**; this doc names how `AnalysisDimension<SymbolicCost>` instantiates and the deferral discipline for the `data` value.
- [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) — the lens-capability register tracking PROXY/STUB/PARTIAL/COMPLETE status per lens. **Updates**: `cost.dag` row "What v2 has that v3 drops" column collapses on landing of slice 2 (named SizeVar surface label: now carried by `SizeVariable.display_name: String?` per §1.2 — single substrate-field authority; producer wiring: consumed via `per_call_pattern_at`; cementing test: shipped per §5). The "Dimension<SymbolicCost> wiring deferred on grammar gaps" remainder gets a named dissolution trigger pointing to the class-5 grammar lane.
- [`../INVARIANTS.md`](../INVARIANTS.md) C-8 (fail-closed compilation) — load-bearing for §4 (the product-zero fix removes a fabrication-shaped behavior; v3 was not failing closed on the annihilation-law violation, it was returning a wrong-but-plausible cost).
- [`../INVARIANTS.md`](../INVARIANTS.md) P1 (modeling faithfulness) — load-bearing for §4.2 (the fix declares the algebra structure rather than special-casing the helper).
- [`../INVARIANTS.md`](../INVARIANTS.md) P2 (boundary discipline) — load-bearing for §3.2 (the `per_call_pattern_at` query surface as single authority).
- [`../INVARIANTS.md`](../INVARIANTS.md) P5 (progress is dissolution) — load-bearing for §8.1 (atomic SizeVariable retirement, no bridge), §8.3 (named dissolution trigger for the trampoline), §8.6 (no parallel corpus).
- `feedback_lenses_not_passes` — load-bearing for §3 (zero heuristics; the lens is a fold over substrate facts, never derives them).
- `feedback_state_space_vs_behavioral_invariants` — load-bearing for §1.2 (single name authority via `SizeVariable.display_name: String?` field; consumers cannot disagree on the user-facing label for a given `source_port`).
- `project_lattice_consolidation` (memory) — this slice's declaration of `Semiring<SymbolicCost>` is one of the named ad-hoc lattices/algebras the project memory tracks; declaring inhabitance is the consolidation move.
- `project_algebra_operator_dispatch` (memory) — Semiring declaration enables `+`/`*` operator dispatch over `SymbolicCost`.

This document does NOT modify:

- The seven-variant `SymbolicCost` coproduct shape itself (per DB-7's STOP SIGNAL on an eighth variant).
- The `AnalysisDimension<C>` substrate carrier (per DB-3 — sibling, not replacement).
- v2's `complexity.dag` (it is the oracle, not a target of change).

## §10. Implementation order (sketch)

Within T-Lens-Behavioral-Parity slice 2 (cost) per [`docs/r3-structure.md`](r3-structure.md) closure gates:

1. **`SizeVariable.display_name` field add** (`sizevariable_displayname_landed`). Add `display_name: String?` field to `SizeVariable` in `src/v3/std/algebra.dag` (per §1.2); update Rust mirror in `src/v3/compiler/src/dag.rs` (single field add); wire parser to populate from authored binding names. Single substrate authority for the user-facing name. Atomic.
2. **Semiring<SymbolicCost> declaration + product-zero fix** (`symbolic_cost_semiring_inhabitance_landed`). Declare the `Semiring<SymbolicCost>` instance; add `collapse_on_multiplicative_zero`; update `reduce_product`. Atomic with the dispatch.
3. **`per_call_pattern_at` substrate query surface** (`per_call_pattern_query_surface_landed`). Expose `per_call_pattern_at(d, call_site) -> CallPattern?` from `std.computation`; the query reads `v3_compiler::dag::per_call_descent_evidence`. Co-owned with complexity slice 1 (same query).
4. **Cost lens producer consumption** (`cost_lens_consumes_per_call_pattern`). Extend `entry_for` to dispatch on `per_call_pattern_at`; add `recursive_transform_cost`; add `call_pattern_to_iter_bound`. Depends on steps 1–3.
5. **Cementing TestClaim against v2 oracle** (`cost_lens_v2_oracle_equivalence_demonstrated`). Author the corpus under `src/v3/test/cost_lens_corpus/`; author the structural-equivalence harness; wire as a `TestClaim` referencing `analyze_symbolic_cost_dimension` and v2's `analyze`. Depends on steps 1–4.
6. **Capability-register row update** (`cost_lens_capability_register_complete`). Update `docs/v3-lens-capability-register.md` row to BEHAVIORALLY COMPLETE. The "Dimension<SymbolicCost> wiring deferred on grammar gaps" residual collapses onto the class-5 grammar lane's dissolution trigger.

Steps 1–3 are parallel-dispatchable (independent substrate authorings). Steps 4–5 are sequential. Step 6 is the closure receipt.

Total estimate (per L-XL sizing in the lane row): substrate carriers + producer-query surface + lens rewiring = M-L; corpus + cementing harness = M; capability-register update = S. End-to-end: 3-5 weeks worker time at standard R3 cadence post-cascade.

---

**This document is a design spec, not a ship target.** It resolves the structural design questions blocking T-Lens-Behavioral-Parity slice 2 (cost) dispatch. The slice itself runs once cascade gates clear (T-E-P-Producer-Broadening COMPLETE + R2-Evaluator landed). All §8 design questions resolved in-doc; no Director ratification required before substrate authoring begins.
