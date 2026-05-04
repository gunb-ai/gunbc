# Complexity Lens — Behavioral Completeness

> Part of: [`docs/r3-structure.md`](r3-structure.md), [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md), [`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md), [`docs/design-symbolic-cost-algebra.md`](design-symbolic-cost-algebra.md), [`../INVARIANTS.md`](../INVARIANTS.md)
>
> **Purpose:** specify the substrate carriers and consumer wiring that take `src/v3/lenses/complexity.dag` from STRUCTURALLY-TERMINAL / BEHAVIORALLY-PROXY to STRUCTURALLY-TERMINAL / BEHAVIORALLY-COMPLETE — the slice-1 deliverable of T-Lens-Behavioral-Parity.
>
> **Authority discipline:** R3 design doc. Implementation lane is **T-Lens-Behavioral-Parity slice 1 (complexity)** ([`docs/r3-structure.md`](r3-structure.md) row 146); cascade-gated on T-E-P-Producer-Broadening, R2-Evaluator, R2-T-Substrate-Lens-Primitive. This doc resolves the substrate-shape questions that block lane dispatch.

## What this document is

The 2026-04-21 audit recorded in [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) found that `src/v3/lenses/complexity.dag` produces `Lookup<Int>` (a per-port integer depth) where v2's `src/v2/complexity.dag` produces `ComplexitySummary { work, span, output_size, certainty }` over a symbolic `CostExpr` algebra. The register flags the delta in the *"What v2 has that v3 drops"* column:

> CostExpr (Sum/Mul/Log/Const), SizeExpr, work/span split, Certainty, asymptotic classification, recurrence bounds.

This is six substrate gaps, not one. Each gap is a fact v2 carried through `ComplexitySummary` that v3's depth proxy collapses. Closing the gap requires substrate-introducing each fact as a typed carrier, wiring producers (per T-E-P-Producer-Broadening), and rewriting the lens consumer to read those carriers — then a v2-oracle cementing test asserts equivalence on the same source.

Per [`../INVARIANTS.md`](../INVARIANTS.md) P1 *Modeling Faithfulness* — a `STRUCTURALLY TERMINAL` + `BEHAVIORALLY PROXY` lens whose name implies COMPLETE is an authored faithfulness failure. Per P5 *Progress Is Dissolution*, the depth-proxy is a scaffold awaiting the dissolution this design names.

## §1. Substrate carriers — the six dropped facts

The register names six facts. Each maps to a substrate authority. The **DFS-the-concept-DAG** check (per MODELING.md M9 / [`../INVARIANTS.md`](../INVARIANTS.md) P1 substrate-fact introduction procedure) identifies the parent for each.

### §1.1 Symbolic CostExpr full algebra (Sum/Mul/Log/Const)

**Status:** v3 has `SymbolicCost` declared in `src/v3/std/algebra.dag` (DB-7) with seven variants (`ConstantCost | LinearCost | PolynomialCost | ProductCost | SumCost | LogCost | UnknownCost`). The substrate carrier exists. **What is missing** is the lens-consumer wiring (`lenses/cost.dag` reads it; `lenses/complexity.dag` does not — it still walks `Lookup<Int>` per port).

**DAG-ancestor check (P1 Step 1):** The parent concept is `Lens<C>` with `C = SymbolicCost`, an instance of the lens framework. No new substrate type. The complexity lens's output type changes from `Lookup<Int>` to `Lookup<ComplexitySummary>` (defined in §1.6).

**Coproduct-vs-coordinate check (P1 Step 2):** `SymbolicCost`'s seven variants are alternatives (a single bound is one of these shapes), not coordinates. Confirmed sum type. (See [`docs/design-symbolic-cost-algebra.md`](design-symbolic-cost-algebra.md) §"Four-pattern dissolution receipt" Q4 — Pattern 3 confirmed terminal.)

**Substrate change:** none for the algebra itself. The complexity lens imports `SymbolicCost` from `std.algebra` (already done by `lenses/cost.dag`).

### §1.2 SizeExpr — named size variables with value semantics

v2's `SizeExpr` carries `SizeVar { name: String }` — a *named* size variable that two cost expressions can reference symbolically (e.g., `O(n²)` where both `n`'s name the same thing). v3's current `SizeVariable { source_port: PortId }` (in `src/v3/std/algebra.dag`) drops the name, breaking same-name-equality reasoning.

**DAG-ancestor check:** `SizeVariable` already exists in `src/v3/std/algebra.dag`. The fix is *enrichment*, not a new type.

**Coproduct-vs-coordinate check:** `name` and `source_port` are coordinates of one size variable (every size variable has both a name and a source port). Confirmed record extension, not sum-type variant.

**Substrate change:**

```dag
// src/v3/std/algebra.dag — SizeVariable gains display_name field

type SizeVariable {
  source_port: PortId           // structural backing — which port's runtime size
  display_name: String?         // user-facing name; populated by parser at authoring sites
}

// Equality on source_port only (display_name is presentation, not identity).
fn size_variable_eq(a: SizeVariable, b: SizeVariable) -> Bool =
  port_id_eq(a.source_port, b.source_port)
```

**Why `display_name: String?` field** (aligned with [`docs/design-cost-lens-sizevar-dimension-wiring.md`](design-cost-lens-sizevar-dimension-wiring.md) §1.2): v3 does NOT have a `intern_table::name_of(port_id) -> String` query landed (per `src/v3/std/algebra.dag:143` "InternTable lookup the lens doesn't yet run"). v3 has some InternTable machinery (per `project_intern_table` memory + PR #367 Phase 1) but the port-id-to-authored-name query is not wired. Assuming an unlanded query would lock substrate-target discipline. The structural-field path is what v3 currently supports. Single-authority discipline (P2) is preserved by making `display_name` the only authority — no parallel InternTable lookup; the renderer reads `display_name` directly from the carrier.

**Why `String?`, not `String`**: per [`../INVARIANTS.md`](../INVARIANTS.md) C-9 (no fabrication), when no user-authored name exists the field is `None`; the renderer derives a fresh label from `source_port`. Never invent a fake name and stash it in the field.

**Why this is not a separate `SizeExpr` carrier:** `SizeExpr` in v2 was a coproduct (`SizeConst | SizeVar | SizeLen | SizeAdd | SizeMax`). In v3, `SymbolicCost` already covers `SizeConst` (`ConstantCost(Int)`), `SizeAdd` (`SumCost`), `SizeMax` (the dominance ordering), and `SizeLen` (`LinearCost(SizeVariable)`). The only "missing fact" the capability register names is the user-facing-name surface — closed by the `display_name` field (above), not by adding a new carrier. Reusing `SymbolicCost` for both cost and size dissolves v2's parallel `SizeExpr` ↔ `CostExpr` authorities into one — same fact-flow-forward shape. (Sustainability invariant: cost-of-change=1 when one type subsumes two.)

### §1.3 Work/span dimension split

v2's `ComplexitySummary` carries `work: CostExpr` and `span: CostExpr` separately. Work = total operations (sequential cost); span = critical-path length (parallel cost). In a sequential composition `f; g`, work composes additively (`work(f) + work(g)`) and span composes additively (`span(f) + span(g)`). In a *parallel* composition `f || g`, work composes additively but span composes via `max`. The two dimensions diverge whenever parallelism appears.

**DAG-ancestor check:** This is two `Dimension<SymbolicCost>` instances per [`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md) (DB-3) — one for work, one for span — sharing the `SymbolicCost` carrier but composing via different monoid operations. The parent is `AnalysisDimension<SymbolicCost>` (already declared in `src/v3/std/dimensions.dag`).

**Coproduct-vs-coordinate check:** Work and span are coordinates of one `ComplexitySummary` (every program has both a work bound and a span bound simultaneously). Confirmed record, not sum type. **NB:** at the *Dimension* layer they are distinct instances (each has its own `compose` monoid); at the *Summary* layer they are coordinates of a single carrier.

**Substrate change:**

```dag
// src/v3/lenses/complexity.dag — declare both dimensions

import std.algebra { Monoid, SymbolicCost, sequential_monoid, max_path_monoid, ConstantCost }
import v3.std.dimensions { AnalysisDimension, Witness, Inhabits, Violates }

// Work dimension: sequential composition is additive; parallel is also additive.
// (Both branches' work is performed regardless of which one's critical path dominates.)
data work_dimension: AnalysisDimension<SymbolicCost> = {
  name: "work"
  witness_of: |d, behavior| witness_work(d, behavior)
  compose: sequential_monoid              // Monoid<SymbolicCost>: SumCost op + ConstantCost(0) identity
  break_diagnostic: |behavior, composed| no_diagnostic_for_work()
}

// Span dimension: sequential composition is additive; parallel composition is max.
// The compose monoid is the SEQUENTIAL one by default; the dimension's
// witness_of injects max-of-paths for Branch nodes.
data span_dimension: AnalysisDimension<SymbolicCost> = {
  name: "span"
  witness_of: |d, behavior| witness_span(d, behavior)
  compose: sequential_monoid              // sequential default; Branch witness uses max_path_monoid internally
  break_diagnostic: |behavior, composed| no_diagnostic_for_span()
}
```

**Why two `AnalysisDimension` instances rather than one with both fields:** per DB-3 §"Algebraic constraints", `compose` is a `Monoid<Carrier>` (carrying the binary op + identity together — F2 dispatch / PR #1607). Work and span have *different monoid witnesses* (work-on-Branch is sum-of-paths; span-on-Branch is max-of-paths). Keeping them as two `AnalysisDimension<SymbolicCost>` instances with two distinct `Monoid<SymbolicCost>` values honors that algebraic distinction structurally.

**Cascade gate for the data declarations:** the `data work_dimension: AnalysisDimension<SymbolicCost> = { ... }` form requires class-5 record bodies in `data` declarations (see `src/v3/std/dimensions.dag` header — currently deferred). Until that grammar lands, the Rust execution authority is `v3_compiler::analyze_symbolic_cost_dimension` per existing pattern. The substrate-carrier spec stays here; the data-declaration receipt lands when class-5 ships. (Per [`../INVARIANTS.md`](../INVARIANTS.md) P5 — scaffold dissolution trigger named.)

### §1.4 Asymptotic classification — the BoundedLattice

v2's `ComplexitySummary` does not directly carry an "asymptotic class" enum, but consumers project a `BigOClass` from `CostExpr` for diagnostic display and contract checking (per [`docs/design-lens-application-surface.md`](design-lens-application-surface.md) §2 `ComplexityBudget = AsymptoticClass` and [`docs/design-lens-framework.md`](design-lens-framework.md):119). This projection is currently scattered across consumers; per `feedback_lattice_consolidation` (memory) the *6+ ad-hoc merge functions are unnamed lattice meets; declare in std/*.

**DAG-ancestor check:** Asymptotic classes form a total order under dominance (`O(1) ≤ O(log n) ≤ O(n) ≤ O(n log n) ≤ O(n²) ≤ O(n^k) ≤ O(2^n) ≤ O(unknown)`). Total orders are bounded lattices: meet = `min` (less-dominant), join = `max` (more-dominant), top = `O(unknown)`, bottom = `O(1)`. The parent is `BoundedLattice<T>` (`dsl/std/algebra.dag:263`).

**Coproduct-vs-coordinate check:** The asymptotic classes are alternatives (a bound belongs to exactly one class). Confirmed sum type.

**Primitive-vs-lens-extensible check (P1 Step 3):** The asymptotic-class set is *substrate-declared* (every program's complexity falls in one of these classes — they are intersubjective mathematical consensus, not a domain-specific extension point). Confirmed substrate primitive.

**Substrate change:**

```dag
// src/v3/std/algebra.dag — declare AsymptoticClass + its BoundedLattice instance

// 🟢 TERMINAL coproduct. Total order under asymptotic dominance.
//
// External authority: Knuth (1976) "Big Omicron and big Omega"
// https://en.wikipedia.org/wiki/Big_O_notation
//
// The closed eight-variant set covers the asymptotic surface the thesis
// reasons about. Anything else collapses to ClassUnknown (top). Adding
// a ninth variant is a STOP signal — same discipline as SymbolicCost.

type AsymptoticClass
  = ClassConstant       // O(1)
  | ClassLog            // O(log n)
  | ClassLinear         // O(n)
  | ClassLinearithmic   // O(n log n)
  | ClassQuadratic      // O(n²)
  | ClassPolynomial { degree: PositiveDescentAmount }   // O(n^k) for k ≥ 3
  | ClassExponential    // O(2^n)
  | ClassUnknown        // top — analyzer cannot prove tighter

// BoundedLattice<AsymptoticClass> instance — meets the ROADMAP P2
// dissolution path (four hand-rolled BoundedLattice<T> instances at
// ROADMAP:362-365). This declaration replaces every ad-hoc meet/join
// over asymptotic classes scattered across cost/complexity consumers.
//
// Laws (per dsl/std/algebra.dag:263 BoundedLattice):
//   meet(top, a) = a   ✓ (ClassUnknown meet a = a)
//   join(bottom, a) = a ✓ (ClassConstant join a = a)
//   commutative, associative, absorptive ✓ (total order: meet=min, join=max)
//
// CRITICAL (per ROADMAP:534 SubValueRelation receipt): verify
// meet(top, a) = a IS satisfied, not violated. Test: `meet_top_is_identity`.
data asymptotic_class_lattice: BoundedLattice<AsymptoticClass> = {
  meet: meet_asymptotic_class
  join: join_asymptotic_class
  top: ClassUnknown
  bottom: ClassConstant
}

fn meet_asymptotic_class(a: AsymptoticClass, b: AsymptoticClass) -> AsymptoticClass =
  if asymptotic_dominates(a, b) then b else a   // less-dominant wins

fn join_asymptotic_class(a: AsymptoticClass, b: AsymptoticClass) -> AsymptoticClass =
  if asymptotic_dominates(a, b) then a else b   // more-dominant wins

// SymbolicCost → AsymptoticClass projection: structural fold.
// This dissolves the 6+ ad-hoc projections in current cost-display code.
fn classify(cost: SymbolicCost) -> AsymptoticClass = ...    // structural match per variant
```

**Why `BoundedLattice` and not just `Lattice`:** every program has both a least-dominant possible class (`ClassConstant` — bottom) and a worst-case fallback (`ClassUnknown` — top). The bounded form is the structurally honest carrier; per `feedback_state_space_vs_behavioral_invariants` the type makes "no top" unrepresentable.

**Cascade gate:** the `data asymptotic_class_lattice` form requires class-5 record bodies (same blocker as `work_dimension` in §1.3). Same deferral pattern; same dissolution trigger.

### §1.5 Certainty — proof confidence carrier

v2's `Certainty = Proven | Conservative` annotates a `ComplexitySummary` with whether its bound is exact (Proven — the worst case actually occurs) or merely an upper bound (Conservative — the bound is sound but possibly not tight). Cost-display consumers use this to choose between `O(n)` (conservative) and `Θ(n)` (proven, both upper and lower).

**DAG-ancestor check:** This is a structural fact about the proof, not the cost itself. Parent: `DescentEvidence` in `src/v3/std/termination.dag` already partitions evidence into `Strict | NonIncreasing | DescentUnknown`, but `Certainty` is one level up — a fact about the *bound's* tightness, not the *step's* descent. They share lineage (both encode proof confidence) but are distinct facts.

**Coproduct-vs-coordinate check:** Two alternatives (a bound is either Proven or Conservative). Confirmed sum type. (No third "Unknown" because that case is already absorbed by `ClassUnknown` in §1.4 — when even the upper bound is unknown, certainty is moot.)

**Substrate change:**

```dag
// src/v3/lenses/complexity.dag — declare Certainty in the complexity lens
// (it is complexity-lens-specific; not promoted to std/algebra.dag yet)

// Certainty: how tight is the bound?
//   Proven       — bound is exact (Θ-equivalent; lower = upper)
//   Conservative — bound is a sound upper bound, possibly loose
//
// The carrier composes via meet under sequential composition: a chain
// is Proven only when every link is Proven. (One Conservative link
// poisons the chain to Conservative.) Lattice-shaped, like DescentEvidence.

type Certainty = Proven | Conservative
```

**No `BoundedLattice<Certainty>` declaration**. Earlier drafts proposed a lattice with `meet = "conservative wins under composition"` and `join = "proven wins under alternative"`. Both semantics are **cost-unaware** and would let a `Proven` arm hide a `Conservative` arm that carries the actual worst-case bound — violating P1 (modeling faithfulness): the result's certainty would no longer faithfully describe the bound it qualifies.

**Certainty composition is cost-aware, not lattice-fold.** When two cost-and-certainty pairs `(c₁, k₁)` and `(c₂, k₂)` compose:

- The cost composes structurally (sequential / iterate / branch-max per `SymbolicCost` semantics).
- The certainty of the result depends on **which input dominated the cost**:
  - If both contributions survive composition: certainty is `meet(k₁, k₂)` (any unproven contribution makes the whole result unproven).
  - If `c₁` dominates `c₂` (e.g., branch-max picks `c₁` as worst-case): the result's certainty is `k₁` — `k₂` does not enter, because the bound being qualified is `c₁`'s bound.
  - Symmetric for `c₂` dominating `c₁`.

This is the `certainty_of_surviving_per_dim` projection in §3.1 (`compose_summary` family), applied per-dimension across both work and span (and any future dimensions per DB-3). A `BoundedLattice<Certainty>` declaration would suggest certainty composes independently of cost — exactly the bug the §3.1 design rejects. Certainty stays a 2-variant sum without an associated lattice instance; the composition lives at `ComplexitySummary` level via the cost-aware, per-dimension `compose_summary_*` functions.

**Tightness ordering**: `Certainty` does have a natural ordering (`Proven ≥ Conservative` under tightness). That ordering is implicit when projecting to a single certainty value, but it is NOT a composition operation. Composition consumes cost-and-certainty pairs jointly per §3.1.

**Why declare it in `lenses/complexity.dag`, not promote to `std/algebra.dag`:** per MODELING.md M10, new concepts get proper homes. `Certainty` is a complexity-analysis fact today; if other lenses (effects, parallelism) later need bound-tightness, it promotes to std/. Premature promotion couples unrelated consumers. Lens-local for now; promotion trigger named in the file header.

### §1.6 Recurrence bounds — the Master Theorem path

v2's complexity analyzer derives bounds for divide-and-conquer recurrences (`T(n) = a·T(n/b) + O(n^d)`) via the Master Theorem. The recurrence carrier and the theorem implementation already exist in `src/v3/std/induction.dag` as `RecurrenceForm` and `master_theorem` (E-I lane). What is missing is: (a) the producer wiring from `Loop` nodes to `RecurrenceForm` (depends on T-E-P-Producer-Broadening), and (b) the lens consumption that reads `master_theorem` output back into `SymbolicCost`.

**DAG-ancestor check:** `RecurrenceForm` and `CostBound` are declared in `src/v3/std/induction.dag`. Parent exists; no new substrate type for the recurrence shape.

**Substrate change:**

```dag
// src/v3/std/induction.dag — add CostBound → SymbolicCost projection

import std.algebra { SymbolicCost, ConstantCost, LinearCost, PolynomialCost, ProductCost, LogCost }

// Bridge from structurally-proven CostBound (induction.dag authority)
// to the SymbolicCost coproduct (algebra.dag authority that lens
// consumers walk). This is a structural projection, not a parallel
// representation: CostBound is the *proof object*; SymbolicCost is
// the *display/composition algebra*. Two layers of one fact.
//
// (Per feedback_parallel_representation_debt: when a canonical source
// exists, consume it. CostBound IS canonical for the proof; the
// projection forwards facts into the lens-display layer.)
fn cost_bound_to_symbolic(bound: CostBound) -> SymbolicCost =
  match bound {
    ConstantBound => ConstantCost(0)
    AtomicBound { cost: c } => atomic_cost_to_symbolic(c)
    ProductBound { factors: fs } => product_to_symbolic(fs)
    SumOfProductsBound { terms: ts } => sop_to_symbolic(ts)
    SumBound { terms: ts } => sum_to_symbolic(ts)
    ForeverBound => UnknownCost("forever — see ForeverBound in induction.dag")
    ErrorBound => UnknownCost("invalid recurrence — see ErrorBound in induction.dag")
  }
```

**Why a projection, not a unification:** `CostBound` carries discharge-proof structure (every term traces to a recurrence shape). `SymbolicCost` carries normalization/composition algebra. Forcing them to be one type would either (a) bloat `SymbolicCost` with proof-object fields lens consumers don't need, or (b) bloat `CostBound` with composition operators induction.dag doesn't need. Projection forward keeps each authority focused. (Per `feedback_projections_must_compose_facts`.)

### §1.7 ComplexitySummary — the unified output type

The complexity lens output type changes from `Lookup<Int>` to `Lookup<ComplexitySummary>`:

```dag
// src/v3/lenses/complexity.dag — the lens output

import std.algebra { SymbolicCost, AsymptoticClass }
import v3.std.lookup { Lookup }

type ComplexitySummary {
  work: SymbolicCost              // total operation count (sequential cost)
  span: SymbolicCost              // critical path length (parallel cost)
  asymptotic_class: AsymptoticClass    // projection of `work` for diagnostic display
  work_certainty: Certainty       // proof tightness OF the work bound
  span_certainty: Certainty       // proof tightness OF the span bound
}

fn complexity_of(d: Dag, port_id: PortId) -> Lookup<ComplexitySummary> = ...
```

**Why per-coordinate certainty, not a single global field**: per cursor BLOCKING on PR #1488 sha 75a6ab57 — collapsing per-dimension certainties into one global `certainty` field loses the fact that work might be Proven while span is Conservative (or vice versa). Downstream display/enforcement consumers need to know per-coordinate proof tightness independently. Per dimension has its own dominance (different inputs to composition) and therefore its own certainty; storing them separately preserves all the proof-tightness facts the lens computes.

**Why these are coordinates of one record:** every port has all five facts simultaneously. Confirmed record (not sum type). Per [`../INVARIANTS.md`](../INVARIANTS.md) P1 Step 2 — single inhabitant carries values for all coordinates. The `asymptotic_class` is a projection of `work`; its certainty is `work_certainty`. (No separate `class_certainty` — the class is derived from work, not an independent fact.)

## §2. Producer wiring — gated on T-E-P-Producer-Broadening

The lens consumer reads `SymbolicCost`/`Certainty`/`AsymptoticClass` per port. Those facts come from per-call evidence (`DescentEvidence`, `CallPattern`, `SubValueRelation`) at every recursive call site. T-E-P-Producer-Broadening (R3 row 37) is the lane that broadens producer coverage from the current "first slice" (recursive self-call + arithmetic-descent) to *full `ExprCall.descent_evidence` parity at live call sites*.

**This design depends on T-E-P-Producer-Broadening landing first.** Per [`docs/r3-structure.md`](r3-structure.md) row 146, T-Lens-Behavioral-Parity gates on T-E-P-Producer-Broadening. The complexity-lens-consumer rewrite reads producer-wired facts; without producer coverage, the rewrite has nothing to consume.

The producer-broadening work itself does not extend the substrate (per the register's E-P partial receipt — `TransformNode` stays unwidened); it broadens the *side-table producer* `v3_compiler::dag::per_call_descent_evidence` in `src/v3/compiler/src/dag.rs` to cover every `ExprCall` site. The lens reads through the typed query surface `per_call_pattern_at(d: Dag, call_site: NodeId) -> CallPattern?` exposed from `std.computation` (per [`docs/design-cost-lens-sizevar-dimension-wiring.md`](design-cost-lens-sizevar-dimension-wiring.md) §3.2 + §8.4 — single-authority for cost+complexity per P2; the lens does *not* reach into `per_call_descent_evidence` storage directly).

**Cascade-gate sequence:**

1. **T-E-P-Producer-Broadening lands** — `per_call_descent_evidence` covers every recursive call site, not just the self-call/arithmetic slice.
2. **Substrate carriers land** (this design's §1.1–§1.7 deltas).
3. **Lens consumer rewrites** (§3 below).
4. **Cementing test asserts equivalence** (§4 below).

The four steps are sequential: each consumes the previous. Internal parallelism inside each step is fine.

## §3. Lens consumer rewrite

The `complexity.dag` lens body changes from per-port `Int` accumulation to per-port `ComplexitySummary` accumulation. The structural shape (forward fold over `d.nodes`, fail-closed on missing producers, `Lookup` carrier) is preserved per the existing depth-proxy file's authority — same catamorphism, richer carrier.

```dag
// src/v3/lenses/complexity.dag — rewritten body (sketch)

module lenses.complexity

import std.list { List, empty, fold, cons }
import std.substrate { Dag, Behavior, BranchPath, LoopNode, NodeId, PortId, node }
import std.algebra {
  SymbolicCost, AsymptoticClass, classify,
  sequential, iterate, max_path,
  ConstantCost, LinearCost,
  miss_complexity_summary_lookup, hit_complexity_summary_lookup
}
import std.induction { CostBound, master_theorem, cost_bound_to_symbolic }
import v3.std.lookup { Lookup }

type ComplexitySummary {
  work: SymbolicCost
  span: SymbolicCost
  asymptotic_class: AsymptoticClass
  work_certainty: Certainty       // per-coordinate per §1.7
  span_certainty: Certainty
}

type ComplexityEntry {
  port: PortId
  summary: Lookup<ComplexitySummary>
}

fn complexity_of(d: Dag, port_id: PortId) -> Lookup<ComplexitySummary> =
  lookup_summary(compute_summaries(d), port_id)

fn compute_summaries(d: Dag) -> List<ComplexityEntry> =
  fold(d.nodes, seed_bind_params(d.nodes), |acc, behavior|
    cons(entry_for(d, acc, behavior), acc)
  )

// Per-Behavior-variant lowering. The shape mirrors the existing
// cost.dag fold; the carrier is richer.
fn entry_for(d: Dag, acc: List<ComplexityEntry>, behavior: Behavior) -> ComplexityEntry =
  match behavior {
    Value(v)     => leaf_entry(v.result_port)              // O(1) Proven
    Transform(t) => transform_entry(d, acc, t)             // sequential of inputs + 1
    Branch(b)    => branch_entry(acc, b)                    // input + max_path of arms
    Loop(l)      => loop_entry(d, acc, l)                   // recurrence-bound via E-I
    Bind(bind)   => bind_entry(acc, bind)                   // forward result_port summary
  }

// Loop entry: read CostBound from per-call descent evidence (E-P producer)
// and project to SymbolicCost. This is where T-E-P-Producer-Broadening's
// side-table feeds in.
fn loop_entry(
  d: Dag,
  acc: List<ComplexityEntry>,
  l: LoopNode
) -> ComplexityEntry = {
  let cost_bound = recurrence_bound_for(d, l)              // reads via per_call_pattern_at typed query (wraps per_call_descent_evidence side-table)
  let bound_cert = certainty_of(cost_bound)
  let work = cost_bound_to_symbolic(cost_bound)
  let body_summary = body_complexity_at(d, acc, l.body)
  // Joint composition: cost and certainty compose as ONE unit per
  // compose_summary (§3.1) — certainty must be cost-aware so dominated
  // components don't propagate stale Conservative facts to the final bound.
  let outer = ComplexitySummary {
    work: work
    span: work
    asymptotic_class: classify(work)
    work_certainty: bound_cert
    span_certainty: bound_cert         // outer's span = work, so same cert
  }
  ComplexityEntry {
    port: l.result_port
    summary: hit_complexity_summary_lookup(
      compose_summary_iterate(outer, body_summary)
    )
  }
}
```

### §3.1 Joint composition — `compose_summary` (cost-aware certainty)

Per gpt-5-5-pro / codex BLOCKING (sha 98f2fc4f): `Certainty` cannot compose independently from cost dominance. If composition drops a cost component via dominance (`O(n) + O(n²) → O(n²)`), naively meeting the dropped component's `Conservative` certainty into the result would produce a `Conservative` summary even when the surviving component is `Proven`. **`(cost, certainty)` must compose as one unit.**

```dag
fn compose_summary_iterate(outer: ComplexitySummary, body: ComplexitySummary) -> ComplexitySummary {
  let composed_work = iterate(outer.work, body.work)
  let composed_span = iterate(outer.work, body.span)            // span uses outer's work for iteration count
  let composed_class = classify(composed_work)
  // Per-coordinate cost-aware certainty. Each dimension's surviving-
  // contributor certainty stays on that dimension; no global collapse
  // (per cursor BLOCKING on PR #1488 sha 75a6ab57 — collapsing loses
  // proof-tightness facts when work is Proven but span Conservative).
  let work_cert = certainty_of_surviving_per_dim(outer.work, body.work,
                                                  outer.work_certainty, body.work_certainty,
                                                  composed_work)
  let span_cert = certainty_of_surviving_per_dim(outer.work, body.span,
                                                  outer.work_certainty, body.span_certainty,
                                                  composed_span)
  ComplexitySummary {
    work: composed_work
    span: composed_span
    asymptotic_class: composed_class
    work_certainty: work_cert
    span_certainty: span_cert
  }
}

// Per-dimension surviving certainty: walks the dominance outcome on a
// specific cost dimension (work, span, or future dimensions per DB-3),
// identifies which input contributors survived, meets their per-dimension
// certainties. Contributors dropped by dominance on this dimension do
// not enter this dimension's certainty (but stay on their own dimension
// in the composed result).
//
// Takes the per-dimension certainty INPUTS explicitly (outer_cert,
// body_cert) — the caller passes in `outer.work_certainty`,
// `body.span_certainty`, etc., depending on which dimension is being
// composed. No global `outer.certainty` lookup; dimensions are
// independent.
fn certainty_of_surviving_per_dim(
  outer_dim: SymbolicCost,
  body_dim: SymbolicCost,
  outer_cert: Certainty,
  body_cert: Certainty,
  composed_dim: SymbolicCost
) -> Certainty =
  match dominance_outcome(outer_dim, body_dim, composed_dim) {
    BothSurvive => meet_pair(outer_cert, body_cert)
    OuterDominates => outer_cert
    BodyDominates => body_cert
  }

// Inline pairwise meet — used ONLY when both contributions survive cost
// composition for a specific dimension. NOT a free-standing lattice
// operation (no certainty_lattice instance per §1.5 — see that section
// for why).
fn meet_pair(a: Certainty, b: Certainty) -> Certainty =
  match a {
    Proven => b               // any Conservative contribution makes the result Conservative
    Conservative => Conservative
  }

// Same per-dimension cost-aware certainty pattern for sequential and
// branch composition: each computes work_cert + span_cert via
// certainty_of_surviving_per_dim against the per-dimension dominance
// outcome. NO meet across dimensions — the result keeps work_certainty
// and span_certainty independent on the output ComplexitySummary.
// Sequential: work composes additively (sum), span composes additively.
// Branch: work composes via max_path (worst-case arm — only one arm
// executes at runtime, so the arm with maximum work dominates the cost
// for that branch); span composes via max_path (same reasoning — span
// is critical-path, branch's worst-case-arm sets the path). This is
// the "branch-max per SymbolicCost semantics" referenced at §1.4. Both
// dimensions honor per-dimension surviving-contributor accounting
// AND per-coordinate certainty independence.
fn compose_summary_sequential(a: ComplexitySummary, b: ComplexitySummary) -> ComplexitySummary = ...
fn compose_summary_branch(arms: List<ComplexitySummary>) -> ComplexitySummary = ...
```

**Why this matters for behavioral parity**: v2's `ComplexitySummary` composition is implicitly cost-aware (its diagnostic surfaces report `Θ(n²) Proven` rather than `O(n²) Conservative` when the inner `O(n)` term was conservative but dominated). A cost-unaware certainty composition would diverge from v2 on the cementing fixture corpus, failing the closure gate. The cost-aware composition is the correct shape, ratcheted by the cementing test.

**Why the same forward-fold shape, just richer carrier:** the existing depth-proxy is structurally honest (catamorphism over `d.nodes`, fail-closed on `Miss`, parameter pre-seeding). Those properties are independent of the carrier type — they hold for `Lookup<Int>`, `Lookup<SymbolicCost>`, and `Lookup<ComplexitySummary>` identically. The behavioral-completeness work changes *what* each port carries (and how summaries compose jointly), not *how* the lens walks. Per [`../INVARIANTS.md`](../INVARIANTS.md) P5 — the structural change is forward (richer fact + joint composition), not lateral (parallel walker).

## §4. v2-oracle cementing test

Per TESTING.md *Cementing tests (Band C — lens subsumption)* and [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) Discipline rule 6, every `BEHAVIORALLY COMPLETE` claim with a non-`N/A` v2 counterpart requires a cementing test that runs **the same minimal fixture through both implementations** and asserts semantic equality on the published carrier shape (or a documented projection).

**Cross-lens cementing format alignment**: this Rust cementing test is the staged form. **Dissolution trigger**: at T-Tests-As-Data-Completeness step 5 (per [`docs/design-tests-as-data-completeness.md`](design-tests-as-data-completeness.md) §6 step 5 — *cementing dispatch port*), this Rust cementing test ports to a `.dag` `TestClaim`/`QuantifiedTestClaim` declaration alongside the lens-capability register migration. All three behavioral-parity lenses (complexity / cost / effect-enumeration) follow the same staging — Rust cementing today, port to .dag at the migration step. This per-doc consistency is recorded in [`docs/design-r3-lens-substrate-index.md`](design-r3-lens-substrate-index.md) §"Cementing-test format" and matches the cross-lane sequencing in tests-as-data §8.3.

### §4.1 Mechanical shape

The test lives at `src/v3/compiler/tests/integration/cementing/complexity_v2_v3_oracle_test.rs` and is registered from `src/v3/compiler/tests/integration.rs` per `#[path = "integration/cementing/complexity_v2_v3_oracle_test.rs"]` (per the existing dispatch ratchet `cementing_test_modules_exist_for_escalated_v2_complete_registry_claims`).

```rust
//! Cementing test: v2 complexity analyzer vs v3 complexity lens on shared source.
//!
//! Per TESTING.md "Cementing tests (Band C — lens subsumption)" + 
//! docs/v3-lens-capability-register.md Discipline rule 6.

use v2_compiler::complexity::{analyze_complexity as v2_analyze, ComplexitySummary as V2Summary};
use v3_compiler::compile_to_dag;
use v3_compiler::lens_complexity::{complexity_of, ComplexitySummary as V3Summary};

/// Each fixture is a small `.dag` source plus the port whose summary is the claim subject.
struct CementingFixture {
    name: &'static str,
    source: &'static str,
    target_function: &'static str,
}

const FIXTURES: &[CementingFixture] = &[
    CementingFixture {
        name: "constant_value",
        source: "fn one() -> Int = 1",
        target_function: "one",
    },
    CementingFixture {
        name: "linear_fold",
        source: "fn sum(xs: List<Int>) -> Int = fold(xs, 0, |acc, x| acc + x)",
        target_function: "sum",
    },
    CementingFixture {
        name: "nested_fold_quadratic",
        source: "fn all_pairs(xs: List<Int>) -> Int = fold(xs, 0, |acc, x| acc + fold(xs, 0, |b, y| b + x * y))",
        target_function: "all_pairs",
    },
    CementingFixture {
        name: "binary_search_log",
        source: include_str!("fixtures/binary_search.dag"),
        target_function: "binary_search",
    },
    CementingFixture {
        name: "mergesort_nlogn",
        source: include_str!("fixtures/mergesort.dag"),
        target_function: "mergesort",
    },
];

#[test]
fn complexity_v2_v3_oracle_equivalent_on_corpus() {
    for fixture in FIXTURES {
        let v2_summary = run_v2_analyzer(fixture);
        let v3_summary = run_v3_lens(fixture);
        assert_summaries_equivalent(
            &v2_summary,
            &v3_summary,
            fixture.name,
        );
    }
}

/// Structural equivalence on the published carrier shape:
///   - asymptotic_class: exact match (both v2 and v3 project to AsymptoticClass)
///   - work: SymbolicCost projects to the same AsymptoticClass
///   - span: SymbolicCost projects to the same AsymptoticClass
///   - certainty: exact match
///
/// We compare *projections* not raw CostExpr/SymbolicCost carriers because the
/// internal cost expressions may normalize to structurally-different but
/// semantically-equivalent forms. The asymptotic projection is the published
/// behavioral contract per the register's "what v2 has that v3 drops" column.
fn assert_summaries_equivalent(v2: &V2Summary, v3: &V3Summary, fixture: &str) {
    // Asymptotic-class equivalence per dimension (v3 publishes one
    // class derived from work; v3_classify_span derives span class
    // from v3.span structurally — same projection v2_classify applies
    // to v2.span).
    let v2_class = v2_classify(&v2.work);
    let v3_class = v3.asymptotic_class;
    assert_eq!(v2_class, v3_class, "{fixture}: work asymptotic class mismatch");

    let v2_span_class = v2_classify(&v2.span);
    let v3_span_class = v3_classify_span(&v3);
    assert_eq!(v2_span_class, v3_span_class, "{fixture}: span asymptotic class mismatch");

    // Per-coordinate certainty equivalence per §1.7's
    // ComplexitySummary { ..., work_certainty, span_certainty }.
    // v2.certainty is a single global field; v3 carries per-coordinate
    // facts. The documented projection is:
    //
    //     v2.certainty := meet_pair(v3.work_certainty, v3.span_certainty)
    //
    // (any unproven dimension makes the v2-equivalent global certainty
    // unproven; if both v3 dimensions are Proven, v2 is Proven).
    //
    // The test asserts the projection equality — NOT v2.certainty ==
    // each per-coordinate fact independently. Asserting independent
    // equality would reject legitimate per-coordinate divergence
    // (work_certainty=Proven, span_certainty=Conservative is a
    // structurally-valid v3 summary that projects to v2.certainty=
    // Conservative; a legitimate stronger-on-work claim would fail
    // a per-coordinate-equality test against v2's coarser global field).
    let v3_global_certainty = meet_pair(v3.work_certainty, v3.span_certainty);
    assert_eq!(v2.certainty, v3_global_certainty,
               "{fixture}: certainty projection mismatch (v3.work={:?}, v3.span={:?}, projected meet={:?}, v2.certainty={:?})",
               v3.work_certainty, v3.span_certainty, v3_global_certainty, v2.certainty);
}
```

### §4.2 What the test pins

Per TESTING.md *"behavior-driven, not implementation-driven"*: the test pins **asymptotic-class equivalence (per dimension) and certainty-projection equivalence**, not raw `CostExpr`/`SymbolicCost` structural equality. v2's normalization may produce a syntactically different `CostExpr` than v3's `SymbolicCost` for the same program (e.g., `SumCost([LinearCost, ConstantCost(1)])` vs `LinearCost`). Both project to `ClassLinear`. The behavioral contract is the projection, not the syntax.

This is a *documented projection* per TESTING.md *"or assert a documented, reviewed projection when the types differ but the claim is about a specific homomorphism"*. The homomorphisms:
- `v2.work → AsymptoticClass` ↔ `v3.work → v3.asymptotic_class` (the lens's own projection)
- `v2.span → AsymptoticClass` ↔ `v3.span → v3_classify_span(v3.span)`
- `v2.certainty (single global)` ↔ `meet_pair(v3.work_certainty, v3.span_certainty)` (homomorphism: v2's coarser global certainty IS the meet of v3's per-coordinate certainties — any unproven v3 dimension makes the projection Conservative)

The certainty mapping is the meet projection: v2's single `certainty` field IS the meet of v3's per-coordinate `work_certainty` + `span_certainty`. The cementing test asserts v2.certainty matches the v3-meet (NOT v2.certainty == each v3 coordinate independently — that would reject legitimate per-coordinate divergence like `work_certainty=Proven, span_certainty=Conservative`, which validly projects to `v2.certainty=Conservative`). The test's failure surface names both v3 per-coordinate facts plus their meet, so the divergence is debuggable: v3 might have a stronger per-coordinate claim than v2 (e.g., v3 proved one dimension, v2 didn't) — that surfaces as a meet-equivalence pass with per-coordinate richness, NOT as a test failure.

### §4.3 Anti-pattern guard

Per TESTING.md *"Don't compile a full source to test a single lens"*: this test legitimately compiles full sources because the cementing claim *spans the pipeline* (v2's analyzer reads its own AST, v3's lens reads `compile_to_dag` output). Per TESTING.md *"Scope clarifier"* — boundary tests / pipeline-spanning thesis claims are the legitimate `compile_to_dag` users. Cementing falls in this category.

The fixtures are minimal (single function, no setup boilerplate). The test asserts on lens output, not pipeline intermediates.

### §4.4 Cementing fixture corpus shape

The corpus covers each `AsymptoticClass` variant at least once, with at least one Master-Theorem fixture (mergesort or binary search) to exercise the recurrence-bound path. Per [`../INVARIANTS.md`](../INVARIANTS.md) P3 *Fail-Closed* — when v3 cannot derive a bound a v2-oracle delivers, the test fails (no silent skip, no `xfail`). Closure of the fixture is itself the closure gate.

**Closure gate:** `complexity_lens_behaviorally_complete` (per [`docs/r3-structure.md`](r3-structure.md) row 146). The gate fires when the cementing test passes on the full corpus AND the register's `complexity.dag` row updates from `BEHAVIORALLY PROXY` to `BEHAVIORALLY COMPLETE`.

## §5. Implementation order

The dependency chain (per §2 cascade-gate sequence) drives a strict ordering:

1. **T-E-P-Producer-Broadening lands** (separate lane; cascade prerequisite).
2. **Substrate carriers land** in one PR per §1: `SizeVariable.display_name: String?` field add (per cost-lens §1.2; v3 has no PortId-to-authored-name InternTable query, so the substrate-field path is the single authority), `AsymptoticClass` + lattice declaration (§1.4), `Certainty` 2-variant sum **without** an associated lattice instance (§1.5 — composition is cost-aware via `compose_summary_*` per §3.1, not lattice-fold), `cost_bound_to_symbolic` projection (§1.6), `ComplexitySummary` carrier (§1.7), `Dimension<SymbolicCost>` data declarations (§1.3 — gated on class-5).
3. **Lens consumer rewrite** in one PR per §3.
4. **Cementing test + fixture corpus** in one PR per §4.
5. **Register update** to `BEHAVIORALLY COMPLETE` in the same PR as the cementing test.

Steps 2 and 3 may overlap if the substrate and lens authors are different workers. Step 4 requires steps 2+3 complete (the cementing test asserts on the new lens output). Step 5 lands strictly with step 4 per the existing register Discipline rule 6 (*Promoting a row to BEHAVIORALLY COMPLETE with a non-N/A v2 counterpart without landing the cementing module in the same PR is a process failure*).

**Sizing per the lane row (R3 row 146):** L-XL for the full T-Lens-Behavioral-Parity lane; this slice 1 (complexity) is M within that.

## §6. Cross-program coordination

Per [`docs/r3-structure.md`](r3-structure.md) row 146, T-Lens-Behavioral-Parity is **cross-program** between Substrate Manager and Verification Manager:

- **Substrate Manager owns:** carriers in `src/v3/std/algebra.dag` (§1.4 `AsymptoticClass`, §1.2 `SizeVariable` enrichment), `src/v3/std/induction.dag` (§1.6 projection), `src/v3/lenses/complexity.dag` (§1.5 `Certainty`, §1.7 `ComplexitySummary`, §3 lens body), `src/v3/std/dimensions.dag` data declarations (§1.3, gated on class-5).
- **Verification Manager owns:** the `complexity_v2_v3_oracle_test.rs` cementing module + fixture corpus (§4); the register-update PR (§5); coordination with the existing `cementing_lens_registry_dispatch_test` ratchet.

The split mirrors T-CostLens-Composition's existing precedent: substrate authors carriers + lens body; Verification asserts cementing.

## §7. Resolved design questions

Per `feedback_design_before_implement` — resolve all design questions before implementation. The below resolutions answer the substantive questions surfaced during authoring.

### §7.1 Should work and span be one carrier or two? — RESOLVED: two `AnalysisDimension` instances, one `ComplexitySummary` record

(Resolved in §1.3.) At the *Dimension* layer they are two instances with different `compose` monoids (sum-on-Branch for work; max-on-Branch for span). At the *Summary output* layer they are coordinates of `ComplexitySummary`. Both framings are honest because the relationship is "two dimensions feed one summary" — same shape as DB-3 §"Dimension evaluation" describes for `DimensionReport`.

### §7.2 Should `SizeVariable` carry a name field? — RESOLVED: yes, `display_name: String?` field; substrate field is single authority

(Resolved in §1.2.) Iterations:
1. **First wave** (gpt-5-5-pro at sha ef21e1a0): a `display_name` field PLUS an InternTable lookup would create two sources of truth.
2. **Second wave** (codex BLOCKING at sha 37f3bc62): v3 has no `intern_table::name_of(port_id)` query landed (per `src/v3/std/algebra.dag:143`). Assuming an unlanded query violates substrate-target discipline.

**Final resolution**: add `display_name: String?` to `SizeVariable` as the single substrate authority for the user-facing name. There's no parallel authority (InternTable name-lookup query isn't landed; the structural-field path is what v3 supports). Single-authority discipline (P2) preserved. Aligned with cost-lens §1.2.

### §7.3 Should `Certainty` promote to `std/algebra.dag`? — RESOLVED: stays lens-local until a second consumer needs it

(Resolved in §1.5.) Per MODELING.md M10 — concepts get proper homes when they are needed. `Certainty` is complexity-analysis-specific today. Promotion trigger: when a second lens (effects, parallelism) needs bound-tightness, promote to `std/`. Premature promotion couples unrelated consumers.

### §7.4 Cost ordering on `CostBound` vs `AsymptoticClass`? — RESOLVED: `BoundedLattice` lives on `AsymptoticClass`; `CostBound` projects through `cost_bound_to_symbolic` → `classify`

(Resolved in §1.4 and §1.6.) Per `induction.dag` end-of-file note: *"Cost ordering is intentionally not provided here. A correct dominance relation on CostBound requires structural comparison, not a lossy Int surrogate."* `CostBound` stays a proof object without an ordering. `AsymptoticClass` is the lossy projection where ordering is well-defined (total order = bounded lattice). This dissolves the would-be "cost ordering" parallel authority into one place.

### §7.5 What happens when v3 derives a tighter bound than v2? — RESOLVED: cementing fails (forces investigation)

(Resolved in §4.) Per [`../INVARIANTS.md`](../INVARIANTS.md) P3 — the cementing test asserts equivalence, not subsumption. If v3 derives `O(n)` where v2 derives `O(n²)`, the test fails. The investigation is: (a) is v2 wrong (overconservative)? — fix the test to assert v3's tighter bound and document the v2-improvement; (b) is v3 wrong (unsoundly tight)? — fix v3's lens. Either way, the divergence is investigated rather than papered over. Silent improvement is a faithfulness failure.

### §7.6 What about the four hand-rolled `BoundedLattice<T>` instances in ROADMAP? — RESOLVED: this design dissolves one (FermiDepth-adjacent class-of-bounds), validates the pattern for the rest

(Resolved by §1.4's discipline.) The ROADMAP P2 entry at `:362-365` flags four ad-hoc lattices: `FermiDepth`, `Encoding`, `DescentEvidence`, `SubValueRelation`. This design declares **one** `BoundedLattice<AsymptoticClass>` instance per `dsl/std/algebra.dag:263` (no `BoundedLattice<Certainty>` — `Certainty` composition is cost-aware per §1.5 + §3.1, not lattice-fold). The `meet(top, a) = a` law (per ROADMAP:534 SubValueRelation receipt) is *verified by construction* — `meet_asymptotic_class(ClassUnknown, a) = a` is structurally trivial because `ClassUnknown` is the dominance top. The new `BoundedLattice<AsymptoticClass>` instance is algebraically honest from day one, validating the dissolution path for the four pre-existing instances. Its dissolution stays under T-V-L4-L7-Direct's `l7_algebraic_laws_witnessed` gate (per [`docs/r3-structure.md`](r3-structure.md):155).

## §8. Relationship to existing authority

This design **extends:**

- [`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md) (DB-3) — the `AnalysisDimension<C>` carrier; this design declares two instances of it (work, span) for the complexity lens.
- [`docs/design-symbolic-cost-algebra.md`](design-symbolic-cost-algebra.md) (DB-7) — the `SymbolicCost` algebra; this design adds `AsymptoticClass` as the projection target.
- [`docs/design-lens-application-surface.md`](design-lens-application-surface.md) — `ComplexityBudget = AsymptoticClass`; this design supplies the carrier the lens-application surface uses for budgets.
- [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) — the register row this design upgrades from `BEHAVIORALLY PROXY` to `BEHAVIORALLY COMPLETE`.
- [`../INVARIANTS.md`](../INVARIANTS.md) P1 *Modeling Faithfulness* — load-bearing for the structural-completeness motivation.
- [`../INVARIANTS.md`](../INVARIANTS.md) P3 *Fail-Closed* — load-bearing for §4 (no silent skip in cementing).
- [`../INVARIANTS.md`](../INVARIANTS.md) P5 *Progress Is Dissolution* — load-bearing for the depth-proxy → behaviorally-complete dissolution.
- TESTING.md *Cementing tests (Band C)* — load-bearing for §4 mechanics.

This design **does NOT modify:**

- The existing `Lens<C>` carrier shape (R2 work — already complete).
- The existing `SymbolicCost` algebra in `src/v3/std/algebra.dag` (DB-7 — algebra is stable).
- The existing E-T/E-C/E-I substrate (carriers staged; this design consumes them).
- The existing `cost.dag` lens — that gets its own cementing test under T-Lens-Behavioral-Parity slice 2 (cost), parallel to this slice 1 work.

## §9. Cascade gates (summary)

Internal cascade (for this slice 1 work):

1. **T-E-P-Producer-Broadening COMPLETE** — full per-call descent evidence available via `per_call_descent_evidence` side table.
2. **R2-Evaluator landed** — lens runtime execution available.
3. **R2-T-Substrate-Lens-Primitive landed** — `Lens<C>` shape available (already done).
4. **Class-5 record bodies in `data` declarations** — required for `data work_dimension` / `data span_dimension` / `data asymptotic_class_lattice`. (No `data certainty_lattice` — `Certainty` does NOT have an associated lattice instance per §1.5; composition is cost-aware via `compose_summary_*` per §3.1, not lattice-fold.) This is *not* a hard cascade gate (the Rust execution authority bridges via `analyze_symbolic_cost_dimension` per existing pattern); when class-5 lands, the data declarations replace the Rust bridges. Dissolution trigger named per [`../INVARIANTS.md`](../INVARIANTS.md) P5.

External cascade: standard R3 worker-dispatch precondition (R2-Evaluator landed).

**Closure gate per [`docs/r3-structure.md`](r3-structure.md) row 146:** `complexity_lens_behaviorally_complete`. Contributes to (alongside the other three slices): `lens_capability_register_zero_proxy_zero_stub` — the register status updated to ZERO PROXY / ZERO STUB at R3 close.

---

**This document is a design spec, not a ship target.** It resolves the structural design questions blocking T-Lens-Behavioral-Parity slice 1 (complexity) lane dispatch. The lane runs once the cascade gates clear (T-E-P-Producer-Broadening COMPLETE + R2-Evaluator landed). All §7 design questions resolved in-doc; no Director ratification required before substrate authoring begins.
