# CX: Complexity Analysis Design

Stable principles for complexity analysis. If a change violates
one of these, stop and update this doc first — then assess the impact
on the codebase to determine if the direction is viable.

See also: `cx-computation-model.md` (migration state),
`cx-violation-triage.md` (current violation snapshot).

---

## Foundational thesis: complexity is a conserved quantity

In a decidable language where all data is finite (Bit/Word64), all
iteration is bounded (fold/descend/repeat), and composition preserves
boundedness (INVARIANTS.md), complexity is not a property to be
*analyzed* — it is a property that *emerges from the model*.

The physics analogy: in a simulator, energy conservation is not
checked by a separate analysis pass after the simulation runs. Every
force application produces a delta-E at the point of application.
The bookkeeping is intrinsic. If you have to run a second pass to
figure out where the energy went, the model has a hole.

The three iteration primitives each have known costs:
- `fold`: O(|collection| x per_element)
- `descend`: O(|tree| x per_node)
- `repeat`: O(N x per_step)

Composition rules (INVARIANTS.md):
- Sequential: cost(a; b) = cost(a) + cost(b)
- Nested: cost(fold(list, f)) = |list| x cost(f)
- Conditional: cost(if c then a else b) = cost(c) + max(cost(a), cost(b))

If user code composes from these primitives, and each primitive has
a known cost, the total cost is determined by the composition structure.
**No post-hoc analysis should be needed.**

---

## The construct-discard-reconstruct anti-pattern

The current implementation violates this thesis. The compiler computes
structural relationships during inference, discards them in the IR,
then a 3000+ line analysis pass (`complexity.dag` + `annotate_descent`
in `04_infer.dag`) tries to reconstruct them via string matching and
heuristic tables.

This is the compiler equivalent of running a physics simulation,
throwing away the force vectors, then running a second pass over the
trajectories to infer what forces must have been applied.

### Where information is constructed and discarded

The discard points all follow the same pattern: `TypeBinding` stores
only `{ name: String, resolved: Node }` — the type. Everything else
the compiler knows at binding-creation time is thrown away.

**1. Let-binding provenance** (`04_infer.dag:1649-1685`)

When inference processes `let child = node.left`, it computes the full
typed expression. It KNOWS that `child` is a sub-value of `node` via
field `left`. It stores: `TypeBinding { name: "child", resolved: Node }`.
The structural relationship is gone.

**2. Lambda body opacity** (`04_infer.dag:745`)

When a lambda parameter is bound, inference stores:
`TypeBinding { name: p, resolved: type_variable_node("lambda_param") }`.
The parameter gets a placeholder type. The compiler knows the full
lambda body — in a decidable language all lambdas are statically known —
but the binding carries none of this.

**3. Match arm provenance** (`04_infer.dag:773-799`)

When `match param { Cons { head, tail } => ... }` creates bindings,
inference knows that `tail` is a strict sub-value of `param` via
variant `Cons`, field `tail`. It stores: `TypeBinding { name: "tail",
resolved: <type> }`. The variant name, field name, and structural
relationship are gone.

**4. For-each element provenance** (`04_infer.dag:1915-1933`)

When `children |> map(c => ...)` creates the binding for `c`,
inference knows `c` is an iterated element of `children`. It stores:
`TypeBinding { name: "c", resolved: <elem_type> }`. The collection
reference and iteration relationship are gone.

**5. Cross-call argument alignment** (`04_infer.dag:2801-2829`)

When caller passes `texpr` to a callee that names its parameter `expr`,
the structural relationship is determined by argument position, not name.
But `build_call_evidence` aligns by string name. Names don't match
→ `SubValueUnknown`. The compiler had the positional binding; the
name-based reconstruction fails.

**6. ShrinkFactor loss** (`std/induction.dag:206-219`)

`sub_value_to_call_pattern` converts `StrictSubValue { field, factor }`
to `ChildAccessorCall { accessor: field_name }`. The factor
(`UnitShrink`, `ConstantShrink`, `ProportionalShrink`) is discarded.
`n/2` (O(log n) steps) becomes indistinguishable from `n-1` (O(n) steps).
The Master theorem becomes unreachable.

### Where information is reconstructed

The discarded information is then reconstructed by 33 distinct
heuristic points across two files:

| Reconstruction method | Count | Examples |
|-----------------------|-------|---------|
| String name matching | 13 | Variable names, method names, field names, type names |
| Hardcoded lookup tables | 8 | `is_child_accessor_in_model`, `function_size_effects`, `type_iteration_dimension` |
| Thread-through maps | 4 | `param_names`, `sub_value_vars`, `size_aliases`, `descent_vars` |
| Type introspection | 5 | `inductive_fields_for`, `element_type` extraction |
| Heuristic assumptions | 3 | Lambda params assumed `IteratedSubValue`, callback transparency |

Key reconstruction functions and what they re-derive:

- `classify_argument` (04_infer.dag:2633) — re-derives whether an
  expression is a sub-value of a parameter by pattern-matching AST
- `classify_let_value` (04_infer.dag:2413) — re-derives let-binding
  provenance by analyzing the value expression
- `classify_collection_shrink` (04_infer.dag:2258) — re-derives
  collection operations by matching method names ("take", "skip")
- `build_call_evidence` (04_infer.dag:2801) — re-derives argument-to-
  parameter alignment by string name matching
- `annotate_descent` lambda heuristic (04_infer.dag:2870-2897) —
  guesses that lambda params receive sub-values by looking for a
  tracked parameter in the same call's other arguments
- `is_child_accessor_in_model` (stage0) — hardcoded table of function
  names that are known to be structural accessors
- `function_size_effects` (stage0) — hardcoded table mapping function
  names to their size effects (TreeSizeReducing, PropertyContraction)
- `construct_termination_proof` (complexity.dag:2721) — re-derives
  termination from scratch using yet another classification system

**Every one of these reconstruction points dissolves if bindings
carry provenance.** The information exists at binding-creation time.
The reconstruction exists because the IR discards it.

### Why this matters beyond CX

This is not just a CX problem. The construct-discard-reconstruct
pattern would repeat for any cross-cutting property:

- **Tracing/observability:** To know which operations contribute to
  a request's latency, you'd face the same problem — the IR doesn't
  carry cost information, so you'd need a reconstruction pass.
- **Space analysis:** To bound memory, you'd need to re-derive
  allocation provenance from the typed AST.
- **Effect tracking:** To know which operations have side effects,
  you'd need to scan the tree after effects were already determined.

The fix for CX establishes the pattern for all of these: **make the
IR carry compositional facts at the point they're computed.**

---

## Principles

### P1: Descent is a type-level fact

Structural descent is a property of the **type definition**, not of
any particular function or code pattern. If `Node` has
`children: List<Node>`, then iterating over `children` is structural
descent — period. This is derivable from the type.

The complexity analyzer should never need to know about specific
accessor functions (`let_value`, `field_access_base`, `child_type_node`).
It reads from the type that certain fields carry recursive sub-structure.
The accessor is just a projection — the descent fact lives in the type.

**Implication:** Hand-maintained tables like `function_size_effects`,
`expr_child_roles`, `node_field_roles` should dissolve into facts
derived from type declarations. Adding a new accessor function should
require zero changes to the complexity analyzer.

### P2: The pipeline preserves facts — never constructs, discards, and reconstructs

Every structural fact the compiler computes must be preserved in the
IR from the point of computation to the point of consumption. The
complexity analyzer is a **consumer** of structural facts, not a
producer or reconstructor.

**The invariant:** at no point in the pipeline should information be
computed, stored in a lossy representation, and then re-derived
downstream via heuristics. If a stage computes a fact, it either:
1. Stores it in the IR for downstream consumers, or
2. Is the final consumer (no downstream re-derivation needed)

This is the same principle as fail-closed diagnostics
(INVARIANTS.md): if the compiler knows something, it must not
discard it and hope a later stage can figure it out.

**Current violation:** `TypeBinding { name, resolved }` discards
provenance. `annotate_descent_evidence` reconstructs it. This is
the root cause of the 33 reconstruction heuristics and 424 CX
violations.

### P3: Every pattern has a bound — but Forever and Unknown are distinct

Every **structurally lowerable** call pattern has a finite bound.
`self(same_arg)` -> `repeat(Forever)`. The analyzer computes bounds
for patterns it can lower.

**Fail-closed:** If the analyzer cannot classify a call pattern —
because descent evidence is missing, not because the pattern is
`repeat(Forever)` — compilation fails with a hard error. This is
`Unknown`, not `Forever`. They are categorically different:

- `Forever` = concrete bound for an actual repeat loop (2^63-1).
  The code IS structurally lowerable. The bound is honest.
- `Unknown` = the analyzer cannot prove a bound. Hard error.
  The code may need restructuring, or the analyzer is incomplete.

**Forever factoring:** A bound like `Forever x n` decomposes into:
- **Termination character:** runs forever (server loop, event loop)
- **Per-iteration scalability:** O(n) in tree size

The bound algebra supports factoring. Forever is honest (not hidden
or rejected) but separated from the scalability analysis.

### P4: Global descent, not strict monotonicity

The current analyzer requires every self-call to strictly decrease
some measure. Real programs take detours — a function may temporarily
preserve or even increase a measure before eventually decreasing it.

**What matters:** over every complete cycle of the computation,
some well-founded measure decreases. Individual steps may not
decrease, as long as no infinite non-decreasing sequence exists.

**Phasing:** Start with strict descent (handles the vast majority
of real patterns). Design the abstraction so it can be relaxed later.
The interface between pipeline and analyzer should be an opaque proof
object that accommodates lexicographic, size-change, or amortized
proofs.

### P5: Complexity classes are emergent

Linear, quadratic, Forever are emergent properties of the bound
algebra. The analyzer doesn't classify — it computes a bound, and
the complexity class falls out.

**Implication:** No `classify_complexity` function. The bound IS the
classification. `O(n)` emerges from `bound = TreeSize(param)`.
`O(n*m)` emerges from nested iteration bounds.

### P6: Recursion is emergent

Recursion is not a first-class concept to model or analyze. It falls
out of functions calling each other. Only iteration primitives
(fold/descend/repeat) are modeled explicitly because they are
intentional developer constructs.

The analyzer never tries to "prove recursion terminates" — it computes
the tightest bound it can from structural facts about data and
operations.

---

## Two viable paths

Both paths eliminate the construct-discard-reconstruct anti-pattern.
They differ in WHERE the iteration primitive identity is captured.

### Option A: Desugar recursive syntax to primitives

The language already says "recursive syntax is sugar"
(`std/iteration.dag:53`). Option A commits to this: the parser (or
an early compiler pass) rewrites recursive functions to explicit
primitive form.

```
// User writes:
fn simplify(expr: CostExpr) -> CostExpr {
  match expr {
    CostAdd { left: l, right: r } =>
      CostAdd { left: simplify(l), right: simplify(r) }
    _ => expr
  }
}

// Compiler lowers to:
descend(expr, f: (node, children) =>
  match node {
    CostAdd { left: _, right: _ } =>
      CostAdd { left: children[0], right: children[1] }
    _ => node
  }
)
```

**What dissolves:** The entire CX-L2/L3 pipeline. There is no
recursive call to analyze — the primitive IS the cost fact.
`descend` costs O(|tree|). `fold` costs O(|collection|). Done.

**Pros:**
- Simplest possible model. Cost is a property of the primitive, not
  a derived analysis.
- Zero reconstruction. The desugared form carries its own cost.
- Aligns with `std/iteration.dag`'s stated position.

**Cons:**
- Requires a desugaring pass that correctly classifies every
  recursive pattern (same classification problem, different location).
- Changes internal representation significantly — all downstream
  passes (emit, ownership) see desugared form.
- User error messages must map back to original syntax.
- The desugaring pass itself needs the same structural knowledge
  (which fields are recursive, which calls descend).

**Viability concern:** The desugaring pass needs to solve the same
classification problem that CX-L2 currently solves. The difference
is that it solves it ONCE (at desugar time) rather than reconstructing
it later. This is strictly better — but the classification work
doesn't disappear, it moves.

### Option B: Provenance on bindings (preferred)

Every binding carries not just its type but its structural
relationship to the function's inputs. Provenance is computed at
binding-creation time and preserved through the IR.

```
type TypeBinding {
  name: String
  resolved: Node
  provenance: Provenance   // NEW: structural relationship to inputs
}

type Provenance
  = Parameter { index: Int }
  | SubValueOf { source: String, field: InductiveField, factor: ShrinkFactor }
  | IteratedElementOf { source: String, field: InductiveField }
  | ArithmeticOf { source: String, factor: ShrinkFactor }
  | ConstructedValue    // new value, not derived from any input
  | Unknown             // fail-closed
```

**What dissolves:**

| Current reconstruction | Why it dissolves |
|------------------------|------------------|
| `classify_argument` (13 heuristics) | Read `provenance` from binding |
| `classify_let_value` (5 heuristics) | Provenance set at let-bind time |
| `classify_collection_shrink` | Method semantics carry shrink info |
| `build_call_evidence` name matching | Provenance flows with value, not name |
| Lambda transparency guessing | Lambda params inherit provenance from caller |
| `is_child_accessor_in_model` table | Field access sets provenance from type |
| `function_size_effects` table | Function return provenance declared on function |
| `sub_value_vars` map threading | Provenance IS the binding, not a side map |
| `size_aliases` map threading | Arithmetic provenance tracks factors |
| SCC evidence re-derivation | Read annotated provenance from call args |

**Pros:**
- No reconstruction. Provenance preserved from computation to consumption.
- Aligns with how the compiler already handles types (types flow
  through bindings; provenance would too).
- Incremental: can be added to TypeBinding without rewriting the
  rest of the pipeline.
- Generalizes: the same provenance field serves CX, space analysis,
  effect tracking, tracing.
- Preserves recursive syntax as user writes it (no desugaring).

**Cons:**
- Every binding-creation site must compute provenance (there are ~10
  sites in 04_infer.dag).
- Provenance must compose through control flow (if/match/block) —
  requires worst-case merging.
- Lambda parameters need special handling: their provenance depends
  on the call site, not the definition site.
- Cross-call provenance requires knowing the callee's parameter
  provenance (function signatures need provenance contracts).

**Key design question for Option B:** Lambda parameter provenance.

When `f(texpr: texpr, callback: child => self(texpr: child))` is
called, the lambda param `child` should carry
`SubValueOf(texpr, field: children)`. But this provenance comes from
what `f` does internally — it iterates over `texpr.children` and
passes each to `callback`.

Two sub-options:
1. **Callee contracts:** Functions declare what they pass to callback
   parameters. `f` declares: "callback receives iterated elements of
   param `texpr`." The compiler checks the contract at definition
   time and applies it at call sites.
2. **Inline lambda bodies:** Since all lambdas are statically known,
   the compiler can inline the lambda at the call site and propagate
   provenance through the body directly.

Sub-option 1 is more modular (works for any higher-order function).
Sub-option 2 is simpler but doesn't generalize to separate
compilation.

### Decision framework

Option B is preferred because it preserves the user's syntax,
generalizes to other cross-cutting properties, and aligns with the
compiler's existing binding model.

If Option B proves unviable (e.g., lambda provenance composition is
too complex to get right), Option A is the fallback — it eliminates
the same anti-pattern by moving classification to a single early pass
rather than reconstructing it later.

**Option A is a named limitation, not a competing architecture.**
Option A and Option B solve the same classification problem — "what
is this value's structural relationship to the function's inputs?"
The difference is where the answer lives:

- Option A: in the desugared primitive (the primitive IS the answer)
- Option B: in the binding's provenance field

The limiting factor for Option A is: **TypeBinding doesn't carry
provenance.** That's the one structural gap. Option A works around it
by classifying at a boundary (desugar pass) instead of at each
binding site. If TypeBinding is ever extended with provenance, the
desugar pass dissolves — the information is already on the bindings,
so there's nothing left to desugar. Option A is a stepping stone
toward Option B, not a fork.

**The non-negotiable:** the construct-discard-reconstruct pattern
must be eliminated regardless of which option is chosen. A 3000+ line
reconstruction pass that grows with every new code pattern is not
sustainable.

---

## Current state: what works and what doesn't

### Sound foundation (keep)

| Asset | Role | Status |
|-------|------|--------|
| `std/iteration.dag` | fold/descend/repeat primitives | Sound. These ARE the cost vocabulary. |
| `std/induction.dag` | InductiveField, SubValueRelation, ShrinkFactor | Sound types. SubValueRelation is close to the right Provenance vocabulary. |
| `std/computation.dag` | CallPattern -> LoweringTarget (exhaustive table) | Sound. 7/7 variants, no heuristics. |
| `std/termination.dag` | DescentEvidence, RankingDimension | Sound proof vocabulary. |
| CX-L1: type-level detection | InductiveField from type declarations | Working. Type → recursive field mapping is structural. |
| Composition algebra | INVARIANTS.md composition rules | Sound. add/multiply/max compose correctly. |

### Partially working (needs restructuring)

| Asset | Issue | Path forward |
|-------|-------|-------------|
| CX-L2: `annotate_descent_evidence` | Reconstruction pass — re-derives provenance from AST | Dissolves under Option B (provenance on bindings) or Option A (desugar) |
| `descent_evidence` on ExprCall | Evidence IS stored on call nodes | Good — but evidence should flow FROM bindings, not be reconstructed BY walking |
| `sub_value_to_call_pattern` | Lossy: drops ShrinkFactor | CallPattern should carry ShrinkFactor, or SubValueRelation should lower directly to LoweringTarget |

### Unsound (replace)

| Asset | Issue | What replaces it |
|-------|-------|-----------------|
| `classify_argument` (13 heuristics) | Reconstructs provenance from AST pattern-matching | Binding provenance |
| `is_child_accessor_in_model` table | Hardcoded function names | Type-derived facts |
| `function_size_effects` table | Hardcoded function name -> effect mapping | Function signature provenance |
| `construct_termination_proof` (old system) | Parallel classification authority to CX-L2 | Single authority: binding provenance |
| `classify_scc_call_progress` (SCC heuristics) | Re-derives evidence that CX-L2 already annotated | Read annotated evidence |
| `iteration_element_name` heuristic | Hardcoded "last param is element" | Algebra template structural lookup |

### Ratchet

424 honest violations (PR #370). These represent genuine gaps in
the compositional evidence model — not implementation bugs. The
violation count traces directly to the construct-discard-reconstruct
pattern: each violation is a case where the reconstruction heuristics
fail to re-derive information that was available at binding time.

---

## Pipeline: current vs target

### Current pipeline (construct-discard-reconstruct)

```
Parse    ──> AST nodes (no cost info)
               │
Infer    ──> TypeBinding { name, resolved }
               │                    ↑ DISCARD: provenance, lambda bodies,
               │                      match context, collection refs
               │
CX-L2    ──> annotate_descent_evidence()
               │  Re-walks entire body. Reconstructs provenance via:
               │  - 13 string name matches
               │  - 8 hardcoded lookup tables
               │  - 4 thread-through maps
               │  - 5 type introspection queries
               │  - 3 heuristic assumptions
               │
CX-L2→L3 ──> sub_value_to_call_pattern()
               │                    ↑ DISCARD: ShrinkFactor
               │
CX-L3    ──> lower_call_pattern() → bound
               (works, but input already degraded)
```

### Target pipeline (Option B: provenance preserved)

```
Parse    ──> AST nodes (no cost info)
               │
Infer    ──> TypeBinding { name, resolved, provenance }
               │  At each binding site:
               │  - let x = node.left    → provenance: SubValueOf(node, left, UnitShrink)
               │  - match arm binding    → provenance: SubValueOf(param, variant.field, UnitShrink)
               │  - lambda param c       → provenance: IteratedElementOf(collection, children)
               │  - let y = n - 1        → provenance: ArithmeticOf(n, ConstantShrink(1))
               │  - let z = new_thing()  → provenance: ConstructedValue
               │
CX       ──> For each self-call, read argument provenance:
               │  self(child)  → child.provenance = SubValueOf(node, left)
               │                → this is a descend on TreeSize
               │                → bound = O(|tree|)
               │
             Done. No reconstruction. No heuristic tables.
             The bound emerges from the provenance chain.
```

### Target pipeline (Option A: desugar)

```
Parse    ──> AST with recursive syntax
               │
Desugar  ──> Rewrite to primitive form:
               │  fn f(n) { ... f(n.left) ... }
               │  → descend(n, fn(node, children) { ... })
               │
Infer    ──> TypeBinding { name, resolved }
               │  (provenance not needed — primitive IS the cost)
               │
CX       ──> Read primitive: descend → O(|tree|). Done.
```

---

## Open design questions

1. **Provenance composition through control flow.** When `let x =
   if cond { a.left } else { b }`, what is x's provenance?
   Worst-case: `SubValueOf(a)` meet `Unknown` = `Unknown`.
   Is this too conservative? Should we track both branches?

2. **Lambda parameter provenance: contracts vs inline.** Callee
   contracts are more modular. Inlining is simpler. The decidability
   invariant guarantees all lambdas are statically known, so inlining
   is always possible. But contracts scale better to separate
   compilation.

3. **ShrinkFactor preservation.** Should CallPattern carry
   ShrinkFactor? Or should SubValueRelation lower directly to
   LoweringTarget (bypassing CallPattern)? The latter is simpler
   and avoids the lossy bridge.

4. **Provenance on function signatures.** For cross-call provenance
   (function f calls function g), does g's signature need to declare
   its return provenance? E.g., `fn accessor(n: Node) -> Node
   @returns SubValueOf(n, field: left)`.

5. **Migration path.** Can provenance be added to TypeBinding
   incrementally (start with `ConstructedValue` everywhere, then
   refine each binding site), or does it require a single coordinated
   change?

---

## Relationship to other docs

- **INVARIANTS.md** — bounded kernel, fail-closed, decidability,
  strict forward progress. The construct-discard-reconstruct pattern
  violates the spirit of fail-closed (discard = lose knowledge).
- **MODELING.md** — shared facts, concept DAG, M9 (DFS the ontology).
  Provenance is a concept that should attach to the ontology.
- **ROADMAP.md section CX** — work items, sequencing, acceptance criteria.
  The CX-B/CX-A/CX-D/CX-C/CX-E sequence describes incremental
  improvement of the reconstruction pass. The new direction replaces
  reconstruction with preservation.
- **cx-computation-model.md** — migration state, compositional deficit
  diagnosis. The "compositional deficit" IS the construct-discard-
  reconstruct pattern named differently.
- **cx-violation-triage.md** — violation breakdown. Stale (based on
  526, current is 424). All violations trace to reconstruction failures.
