> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > **Tier 1: Structural** (CX gate)
> See also: [cx-computation-model.md](cx-computation-model.md), [cx-violation-triage.md](cx-violation-triage.md), [ownership-design.md](ownership-design.md)

# CX: Complexity Analysis Design

Stable principles for complexity analysis. If a change violates
one of these, stop and update this doc first — then assess the impact
on the codebase to determine if the direction is viable.

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

### Option B: Thread existing SubValueRelation through bindings (preferred)

The sound foundation already exists: `SubValueRelation` in
std/induction.dag models exactly the right vocabulary
(StrictSubValue, IteratedSubValue, ArithmeticDescent, PreservedValue,
SubValueUnknown) with InductiveField and ShrinkFactor. The lowering
path exists (SubValueRelation → CallPattern → LoweringTarget). The
lattice operations exist (`merge_argument_relations`,
`sub_value_to_evidence`).

**The problem is not missing types — it's that these facts are
computed too late (CX-L2 reconstruction) instead of at binding time.**

The fix: put SubValueRelation on TypeBinding so the existing
authority is preserved from computation to consumption:

```
type TypeBinding {
  name: String
  resolved: Node
  provenance: SubValueRelation   // NEW — reuses existing authority
}
```

No new type algebra. No parallel vocabulary. The same SubValueRelation
that CX-L2 currently reconstructs is instead computed once at binding
time and carried through the IR.

**Why "at binding time" works:** Inference processes bindings
top-to-bottom (sequential). When `let child = node.left` is bound,
`node` is already in scope with its provenance (`PreservedValue` for
a parameter). The compiler looks up `node`'s binding, finds its
SubValueRelation, and composes with the field access to get `child`'s
provenance. Each binding reads prior bindings' provenance from scope.
The chain is sequential, not circular.

This is the same computation that `annotate_descent` does today via
`sub_value_vars` — but as a separate reconstruction pass. Under
Option B, the computation happens at binding sites because the scope
already carries provenance. The reconstruction pass dissolves; the
classification logic moves to binding sites (~200 lines).

**Known gap: SubValueRelation needs extension.** The current type
covers structural descent, iterated elements, and arithmetic descent.
It does NOT cover property contractions (e.g.,
`with_required_cardinality` changes a semantic property without
shrinking the value). A `PropertyContraction { domain_size: Int }`
variant is needed. This is a small extension (~5 lines in
std/induction.dag) but should not be deferred.

**Known limitation: lambda parameters.** Lambda parameter provenance
depends on the call site, not the definition site. When
`f(texpr, callback: child => ...)` is inferred, the lambda param
`child` has no provenance at binding time — its provenance comes from
what `f` does with `callback`, which requires a callee contract.
Collection methods (fold/map/filter) can get contracts via
AlgebraMethodSemantics. User-defined higher-order functions start at
`SubValueUnknown`. **~105 lambda transparency violations will NOT
dissolve in the first iteration.** This is deferred, not solved.

**What dissolves:**

| Current reconstruction | Why it dissolves |
|------------------------|------------------|
| `classify_argument` (13 heuristics) | Read `provenance` from binding |
| `classify_let_value` (5 heuristics) | Provenance set at let-bind time |
| `classify_collection_shrink` | Method semantics carry shrink info |
| `build_call_evidence` name matching | Provenance flows with value, not name |
| Lambda transparency guessing | Lambda params inherit provenance from caller |
| `is_child_accessor_in_model` table | Field access sets provenance from type |
| `function_size_effects` table | Function return facts from structural contracts |
| `sub_value_vars` map threading | Provenance IS the binding, not a side map |
| `size_aliases` map threading | Arithmetic provenance tracks factors |
| SCC evidence re-derivation | Read annotated provenance from call args |

**Pros:**
- Single authority. Reuses SubValueRelation — no dual representation.
- No reconstruction. Facts preserved from computation to consumption.
- Aligns with how the compiler already handles types (types flow
  through bindings; SubValueRelation would too).
- Incremental: can be added to TypeBinding without rewriting the
  rest of the pipeline.
- Generalizes: the same provenance field serves CX, space analysis,
  effect tracking, tracing.
- Preserves recursive syntax as user writes it (no desugaring).

**Cons:**
- Every binding-creation site must compute SubValueRelation (there
  are ~10 sites in 04_infer.dag).
- SubValueRelation must compose through control flow (if/match/block)
  — requires worst-case merging (lattice already exists).
- Lambda parameters need special handling: their provenance depends
  on the call site, not the definition site.
- Cross-call provenance requires knowing the callee's structural
  contract (which parameter the return value derives from).

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
- Option B: in the binding's SubValueRelation (existing authority)

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

421 violations measured locally (2026-04-11); ratchet in bootstrap.rs
is 424. Root-cause audit identifies 8 categories:

| Cat | Root cause | Count | Fix |
|-----|-----------|------:|-----|
| A | Node tree descent (children are sub-values) | 159 | Body-inferred return contracts |
| B | Parser SCC (integer position advancement) | 132 | Stream D: structural parser ([parser-design.md](parser-design.md)) |
| C | Emission TCO mutual recursion | 22 | Body-inferred return contracts |
| D | Inference mutual recursion (list shrinkage) | 15 | Body-inferred return contracts |
| E | Complexity self-analysis (parser progress) | 17 | Follow-on work |
| F | Tokenizer (integer position into bounded string) | 22 | Separate PR (span design needed) |
| G | Arithmetic descent (`(n-d)/10`) | 44 | S3 classification refinement |
| H | Graph DFS (visited-set termination) | 10 | Deferred (needs worklist primitive) |

**Path to 0:** Stream D (-132) + tokenizer (-22, separate PR) +
body inference (-196) + arithmetic (-44) + complexity self-analysis
(-17) = -411. Remaining ~10 = graph DFS (needs language primitive).

Not all violations trace to construct-discard-reconstruct. Categories
B+F (154) trace to **integer opacity** — the parser uses `pos: Int` where
structural list consumption would be provable. Category G (44) traces to
**arithmetic opacity** — composite expressions like `(n-d)/10` aren't
recognized as ProportionalShrink. Category H (10) traces to **set opacity**
— visited-set growth is not expressible as a bounded primitive.

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
               │  - let z = new_thing()  → provenance: SubValueUnknown (no descent relation)
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

## Option B implementation plan

### Key insight: SubValueRelation IS provenance

`SubValueRelation` (std/induction.dag) already models the right
vocabulary: StrictSubValue, IteratedSubValue, ArithmeticDescent,
PreservedValue, SubValueUnknown. It carries InductiveField and
ShrinkFactor. No new type needed — the authority already exists.

The only thing missing is **placement**: SubValueRelation is
currently computed as a separate reconstruction pass (CX-L2) instead
of at binding time. Put it on TypeBinding:

```
type TypeBinding {
  name: String
  resolved: Node
  provenance: SubValueRelation   // NEW — reuses existing authority
}
```

Source tracking (which parameter this binding derives from) follows
from the scope chain: when `let child = node.left` is bound, the
compiler looks up `node` in scope, finds its SubValueRelation
(PreservedValue for a parameter), and computes `child`'s relation
relative to the same root. No separate `source_param` field needed —
the provenance chain is implicit in the binding structure, just as
type chains are implicit today.

This reuses the existing type, its lattice operations
(`merge_argument_relations`, `sub_value_to_evidence`), and its
lowering path (direct SubValueRelation → LoweringTarget, bypassing
the lossy CallPattern bridge). No new parallel type system.

### Binding-site audit: what each site produces

There are 7 binding-creation sites in `04_infer.dag` that matter
for function-body provenance. Type environment setup (~15 more sites
at lines 3684-3923) creates bindings for type names — these are
always `SubValueUnknown` (type names have no descent relation to
function inputs) and can be ignored.

**Site 1: Function parameters** (`build_params_scope`, line 699)

```
TypeBinding { name: p, resolved: type_expr, provenance: PreservedValue, }
```

Parameters are themselves. `PreservedValue` is correct — the
parameter IS the input. This is the root of the provenance chain.

Available: param name, param type, param index.
Change: add `provenance: PreservedValue`.

**Site 2: Let-bindings** (`extend_scope`, line 1676)

The full typed value expression `val_typed` is available. Provenance
is determined by the value expression's structure:

| Value expression | Provenance |
|------------------|-----------|
| `ExprVar` matching a param | `PreservedValue` (inherited from param binding) |
| `ExprVar` matching a local | inherited from local's binding |
| `base.field` where base has provenance | `StrictSubValue { field, factor: UnitShrink }` |
| `param - k` (arithmetic) | `ArithmeticDescent { factor: ConstantShrink(k) }` |
| `param / k` (arithmetic) | `ArithmeticDescent { factor: ProportionalShrink(k) }` |
| `list \|> skip(k)` | `StrictSubValue { field, factor: ConstantShrink(k) }` |
| `list \|> take(mid)` with DividedSize | `StrictSubValue { field, factor: ProportionalShrink(d) }` |
| Constructor / literal | `SubValueUnknown` (new value, no descent) |
| Function call (unknown contract) | `SubValueUnknown` (fail-closed) |

Available: `val_typed` (full inferred expression), scope with all
prior bindings and their provenance.
Change: compute provenance from `val_typed` before calling
`extend_scope`. This is the same classification that
`classify_let_value` (line 2413) currently does in CX-L2 — but done
once, at binding time, with the result stored.

**Site 3: Match arm bindings** (`extend_scope_match_bound`, line 788)

```
TypeBinding { 
  name: field_name, 
  resolved: field_type,
  provenance: StrictSubValue { field: inductive_field, factor: UnitShrink }
}
```

Available: variant name (`vname`, line 776), field name
(`field_binding_name_at`, line 782), field type, parent enum,
scrutinee expression. The scrutinee's binding has provenance —
propagate it.
Change: look up scrutinee's provenance from scope, compose with
field access to get variant field's provenance.

**Site 4: Lambda params with Callable expected** (line 1792-1796)

```
TypeBinding {
  name: param_name,
  resolved: callable_param_type,
  provenance: <depends on callee contract>
}
```

This is the hard case (see Lambda provenance section below).
Available: expected Callable type with param types. What's NOT
available: which input parameter this lambda param corresponds to.
Change: requires callee contracts or lambda inlining.

**Site 5: Lambda params with element expected** (line 1798-1812)

```
TypeBinding {
  name: param_name,
  resolved: element_type,
  provenance: IteratedSubValue { field: collection_inductive_field }
}
```

Available: expected element type, the collection being iterated
(it's the method receiver — available from the call context).
Change: thread the collection's provenance from the method call
context to the lambda param binding.

**Site 6: Lambda params with no expected** (line 742-755)

```
TypeBinding { name: p, resolved: type_variable, provenance: SubValueUnknown }
```

Fail-closed: if there's no type context, there's no provenance
context either. `SubValueUnknown` is honest.

**Site 7: For-each variable** (line 1925)

```
TypeBinding {
  name: variable,
  resolved: elem_type_node,
  provenance: IteratedSubValue { field: collection_inductive_field }
}
```

Available: `coll_typed` (inferred collection expression), element
type. The collection expression has a binding with provenance.
Change: look up collection's provenance from scope, create
`IteratedSubValue`.

### Provenance composition algebra

Provenance composes through control flow. The merge operation already
exists: `merge_argument_relations` (04_infer.dag:2611). The rules:

**Let-chain (sequential):**
```
let a = node.left       → a: StrictSubValue(node, left)
let b = a.children      → b: StrictSubValue(node, left.children)  // transitive
```
Implementation: when computing b's provenance, look up a's
provenance from scope. If a is `StrictSubValue` of some source,
and b accesses a field of a, b is `StrictSubValue` of the same
source (deeper descent).

**If/match (branching):**
```
let x = if cond { a.left } else { b }
→ merge(StrictSubValue(a, left), PreservedValue(b))
→ PreservedValue (conservative — weakest branch wins)
```
Implementation: compute provenance for each branch, merge via
lattice. This is what `merge_argument_relations` already does.

**Block (sequential):**
```
{ let tmp = f(x); tmp.field }
→ provenance of last expression in block
```
Implementation: provenance of a block is provenance of its last
expression.

**Lattice order (from strongest to weakest):**
```
StrictSubValue > IteratedSubValue > ArithmeticDescent > PreservedValue > SubValueUnknown
```
Merge always takes the weaker (more conservative) of two provenances.
`SubValueUnknown` is absorbing — any branch producing Unknown makes
the whole thing Unknown.

### Lambda provenance: the hard case

When `f(texpr: texpr, callback: child => self(texpr: child))` is
called, the lambda param `child` should carry provenance tracing
back to `texpr`. Two approaches:

**Approach 1: Callee contracts (recommended)**

Higher-order functions declare what they pass to callbacks:

```
// Structural contract in std/ (extends AlgebraMethodSemantics):
type CallbackContract {
  callback_param_index: Int     // which param is the callback (e.g., 2 for fold's f)
  source_param_index: Int       // which param is the collection (e.g., 0 for fold's xs)
  relation: SubValueRelation    // IteratedSubValue for fold/map/filter
}
```

Collection methods already have algebra semantics
(`AlgebraMethodSemantics`). Extend with callback contracts:
- `fold(collection, init, f)`: f's element param gets
  `IteratedSubValue` of collection
- `map(collection, f)`: f's param gets `IteratedSubValue`
- `descend(tree, f)`: f's node param gets `StrictSubValue` of tree

For user-defined higher-order functions (e.g., `emit_shared_expr`):
the compiler can INFER the contract by analyzing what the function
does with its callback parameter. In a decidable language, this
analysis terminates. The contract is computed once at definition
time and attached to the function's signature.

**Approach 2: Lambda inlining (simpler, less modular)**

Since all lambdas are statically known, inline the lambda body at
the call site and propagate provenance through the body directly.
This avoids contracts but doesn't work for separate compilation.

**Recommendation:** Start with contracts for collection methods
(these are the common case and already have algebra semantics).
Defer user-defined higher-order function contracts — use
`SubValueUnknown` for those call sites initially. The ~105 lambda
transparency violations are dominated by collection methods; the
user-defined cases are the long tail.

### ShrinkFactor preservation

The current pipeline loses ShrinkFactor at `sub_value_to_call_pattern`
(std/induction.dag:208). The fix: **bypass CallPattern entirely.**

SubValueRelation already carries ShrinkFactor. LoweringTarget needs
ShrinkFactor for the Master theorem. The bridge through CallPattern
is lossy and unnecessary.

New lowering path:
```
SubValueRelation → LoweringTarget  (direct, preserving ShrinkFactor)
```

Instead of:
```
SubValueRelation → CallPattern → LoweringTarget  (lossy bridge)
```

This means adding a new function `sub_value_to_lowering_target` in
std/induction.dag that maps directly:

| SubValueRelation | LoweringTarget |
|------------------|---------------|
| StrictSubValue { field, UnitShrink } | Descend, TreeSize, Strict |
| StrictSubValue { field, ProportionalShrink(d) } | Descend, TreeSize, Strict + factor d |
| IteratedSubValue { field } | Fold, CollectionSize, Strict |
| ArithmeticDescent { param, ConstantShrink(k) } | Repeat, ArithmeticParam, Strict |
| ArithmeticDescent { param, ProportionalShrink(d) } | Repeat, ArithmeticParam, Strict + factor d |
| PreservedValue | Repeat, Forever, NonIncreasing |
| SubValueUnknown | fail-closed: hard error |

The factor reaches LoweringTarget, so `derive_bound` /
`master_theorem` can compute O(log n) vs O(n) correctly.

### Final-state CX consumer

Under Option B, the complexity consumer becomes:

```
fn bound_for_recursive_function(entry: FuncEntry) -> CostBound {
  let self_calls = find_self_calls(body: entry.body, fn_name: entry.name)
  // Each self-call's arguments already carry SubValueRelation from
  // inference (on ExprCall.descent_evidence). The difference from today:
  // inference computes this at expression-typing time, not as a
  // separate reconstruction pass. Call arguments are expressions,
  // not bindings — the provenance is on the expression node, not
  // looked up from a scope.
  let per_call_provenance = self_calls |> map(call =>
    call.descent_evidence  // already on ExprCall, computed during inference
  )
  // All self-calls must have Strict on at least one param
  let all_have_strict = per_call_provenance |> all(call_ev =>
    call_ev |> any(p => is_strict(p))
  )
  if all_have_strict {
    let best_evidence = select_best_evidence(per_call_provenance)
    let target = sub_value_to_lowering_target(best_evidence)
    bounded_cost(target, per_iter: cost_of_body(entry.body))
  } else {
    CostUnknown { reason: "..." }
  }
}
```

**What dissolves:**

| Current code | Lines | Status |
|-------------|-------|--------|
| `annotate_descent_evidence` walk + threading | ~400 | Dissolves — walk is the reconstruction pass |
| `classify_argument` (13 heuristics) | ~200 | **Moves** to binding sites (~200 lines, same logic, different location) |
| `classify_let_value` (5 heuristics) | ~170 | **Moves** to let-binding site (subset of classify_argument) |
| `classify_collection_shrink` | ~90 | Dissolves — method semantics carry shrink |
| `build_call_evidence` | ~30 | Dissolves — read argument provenance |
| `annotate_descent` lambda heuristic | ~50 | Partially dissolves (collection methods via contracts; HOFs deferred) |
| `is_child_accessor_in_model` table | ~10 | Dissolves — type-derived |
| `function_size_effects` table | ~40 | Dissolves — structural contracts in std/ |
| `construct_termination_proof` (old system) | ~200 | Dissolves — single authority |
| `classify_scc_call_progress` | ~60 | Dissolves — read annotated provenance |
| `sub_value_to_call_pattern` (lossy bridge) | ~15 | Dissolves — direct lowering |
| Thread-through maps (param_names, sub_value_vars, size_aliases) | ~100 | Dissolves — provenance on binding |
| **Total reconstruction code** | **~1365** | |

**Honest accounting:** ~200 lines of classification logic (the core
of `classify_argument` and `classify_let_value`) moves to binding
sites — the same computation, done earlier. The reconstruction
PASS dissolves (~750 lines of walking/threading/context management),
but the classification itself is inherent work that shifts location.

**Net dissolution estimate: ~1100-1200 lines** (derived from the
cleanup catalog line counts below, minus ~200 lines of classification
that moves to binding sites). Plus the old proof system (~450 lines)
dissolves separately.

**What remains:**

| Code | Lines | Role |
|------|-------|------|
| SCC detection | ~200 | Still needed: call graph cycle detection |
| `sub_value_to_lowering_target` | ~30 | New: direct lowering preserving ShrinkFactor |
| Binding-site classification | ~200 | Moved from reconstruction to binding sites |
| `bound_for_recursive_function` | ~50 | New: read provenance, compute bound |
| `bounded_cost` / cost composition | ~100 | Keep: composition algebra is sound |
| `master_theorem` / `derive_bound` | ~80 | Keep: bound computation is sound |
| Reporting / violation collection | ~100 | Keep: output formatting |
| **Total remaining** | **~760** | |

Estimated reduction: ~5000 lines → ~760 lines (~85% dissolution).

### Migration path

Provenance can be added to TypeBinding **incrementally**:

1. **Add field with default.** Add `provenance: SubValueRelation` to
   TypeBinding. Default to `SubValueUnknown` at all creation sites.
   This is a no-op change — all existing behavior preserved, all tests
   pass.

2. **Instrument one site at a time.** Starting with the easiest
   (function parameters → `PreservedValue`), update each binding site
   to compute provenance. After each site, the downstream CX code can
   optionally READ provenance instead of reconstructing — but doesn't
   have to yet.

3. **Add direct lowering.** Write `sub_value_to_lowering_target` in
   std/induction.dag. This preserves ShrinkFactor.

4. **Switch CX to read provenance.** Replace `classify_recursion_pattern`
   to read `binding.provenance` from self-call arguments instead of
   calling `collect_self_call_evidence` → `classify_argument`. This
   is the switchover point.

5. **Delete reconstruction code.** Once CX reads provenance, the
   entire `annotate_descent_evidence` pass and its 33 heuristic
   functions can be deleted. The hardcoded tables dissolve.

Steps 1-2 can run in parallel with existing CX code (provenance is
computed but not consumed yet). Step 4 is the switchover. Step 5 is
cleanup.

### Open design questions

1. **SubValueRelation composition through control flow.** The
   conservative merge (weakest branch wins) may be too aggressive — a
   function that returns `a.left` in one branch and `a.right` in
   another should still be `StrictSubValue`, not `PreservedValue`. The
   merge should check: are both branches sub-values of the same
   source? If so, keep `StrictSubValue` with the weaker factor.
   The lattice operation `merge_argument_relations` already exists
   (04_infer.dag:2611) — the question is whether it needs refinement.

2. **Lambda parameter provenance: contracts vs inline.** Callee
   contracts are recommended for collection methods (already have
   algebra semantics via AlgebraMethodSemantics). User-defined
   higher-order functions deferred to `SubValueUnknown` initially.
   The decidability invariant guarantees inlining is always possible
   as a future fallback.

3. **Function return contracts.** For cross-call provenance (function
   f calls accessor g), g needs a structural contract declaring the
   relationship between its return value and its parameters. This is
   a .dag structural fact (like AlgebraMethodSemantics), not an
   annotation. E.g., `param_node_type_expr` would carry a structural
   witness in std/ declaring "return is SubValueOf(param 0)." This
   dissolves `is_child_accessor_in_model`. Deferred until the basic
   binding-level provenance pipeline is working.

---

## Cross-compiler audit: the anti-pattern is systemic

The construct-discard-reconstruct pattern is not CX-specific. An
audit of all compiler stages found the same structural gap in every
stage that needs facts about values beyond their types.

### Confirmed instances

**1. Complexity (CX) — 33 reconstruction heuristics**

Fully documented above. TypeBinding drops provenance. CX-L2
reconstructs via string matching, accessor tables, and AST re-walks.
424 violations. ~1100-1200 lines net dissolution (~200 lines of
classification logic moves to binding sites).

**2. Ownership — string name matching for fold accumulators**

`ownership.dag` re-derives fold structure by name-matching:
- `a.name == "init"` to find fold initializers (line 214)
- `terminal.name == acc_type_name` to identify accumulator structs
  (line 430)
- `collect_acc_field_moves` re-walks the body to find field accesses
  (lines 438-454)

Root cause: inference knows the fold structure (which arg is init,
which is body, which is collection) but stores it as generic ExprCall
children with string names. Ownership must reconstruct semantics
from names.

**3. Emission — upstream information loss (M2)**

The ROADMAP reviewer root cause analysis (2026-04-03, line 193)
already identified this: "facts that exist upstream are lost before
emit, so emission compensates with heuristics." Specific instances:
- `variant_to_enum` map uses empty string as ambiguity sentinel;
  emitter falls back to AST inspection (05_emit_rust.dag:1787)
- Emitter re-derives method dispatch, expression semantics, and
  clone decisions that inference already computed

M2 (Boundary Sufficiency) is the same anti-pattern named differently:
the resolution→emit boundary doesn't carry enough structure.

**4. Core tables — hand-maintained semantic maps**

`00_core.dag` maintains hardcoded tables that should be derived from
type declarations:
- `expr_child_roles` (line 750): maps ExprData variant names to
  their child accessor functions — via string keys
- `node_field_roles` (line 845): maps Node field names to structural
  recursion roles — via string keys
- `function_size_effects` (line 881): maps function names to size
  effects (TreeSizePreserving, TreeSizeReducing) — via string keys

Every consumer that queries these tables is doing name-based
reconstruction of facts that the type system already has.

**5. Inference — string dispatch for built-in functions**

`04_infer.dag` has `if func_name == "..."` branches for built-in
function type logic:
- `func_name == "empty_map"` (line 1228)
- `func_name == "map_get"` (line 1256)
- `func_name == "map_keys"` (line 1269)
- `func_name == "map_values"` (line 1282)

Each branch applies type-narrowing rules that should be declared on
the function's signature, not hardcoded as name-matching in the
inference engine.

**6. Algebra — ExprBinOp.algebra_field: AlgebraFieldKind?** (RESOLVED)

Resolved: `algebra_field` now uses the structural `AlgebraFieldKind`
coproduct defined in `std/syntax.dag`. All producers and consumers
updated to use structural dispatch instead of string matching.

### The pattern

Every instance follows the same structure:

```
Stage N computes a structural fact
     ↓
IR stores only the type (or a string proxy)
     ↓
Stage N+k reconstructs the fact via name matching / AST re-walking
```

The root cause is always the same: **the IR vocabulary is too narrow.**
TypeBinding stores `{ name, resolved }`. ExprCall stores generic
children. ExprBinOp stores a string. Each narrowing forces downstream
stages to reconstruct what upstream stages already knew.

### What provenance-on-bindings fixes (and what it doesn't)

Provenance on TypeBinding directly fixes instances 1 (CX) and 2
(Ownership). It indirectly helps with 3 (Emission) by making value
relationships visible to emit.

Instances 4-6 are separate but related: they're about semantic
metadata on types and operations (not on bindings). The fix for
those is to move hand-maintained tables into type declarations in
std/ — the same "declare facts at authority" principle.

All six instances share one thesis: **the IR should carry structural
facts from the point of computation to the point of consumption.**
Provenance on bindings is the first and highest-leverage fix.
Type-derived semantic tables are the second.

---

## Workboard

Active work items. Each has a TDD test and a cleanup target.

### Shared infrastructure (with ownership — see ownership-design.md)

| # | Item | Test | Cleanup target | Status |
|---|------|------|---------------|--------|
| S1 | Add `provenance: SubValueRelation` to TypeBinding (default SubValueUnknown) | All existing tests pass, no behavior change | — (no-op step) | DONE (04_env.dag:24-28) |
| S2 | Instrument function params → PreservedValue | Test: compile known .dag, assert param bindings have PreservedValue | — | DONE (04_infer.dag:703) |
| S3 | Instrument let-bindings → classify from value expr | Test: `let child = node.left` produces StrictSubValue | `classify_binding_provenance` (04_infer.dag:2438) | DONE (04_infer.dag:769, 1682) |
| S4 | Instrument match arms → StrictSubValue from variant field | Test: `match param { Cons { tail } => ... }` produces StrictSubValue on `tail` | `extend_scope_match_bound` (04_infer.dag:732) | DONE (PR #379) |
| S5 | Instrument for-each variable → IteratedSubValue | Test: `children \|> map(c => ...)` produces IteratedSubValue on `c` | For-each handling in annotate_descent | DONE (04_infer.dag:1934-1947) |
| S6 | Lambda params with element expected → IteratedSubValue | Test: fold/map callback param gets IteratedSubValue | Lambda heuristic (04_infer.dag:2870-2897) | Partial (standard fold/map path; heuristic source) |
| S7 | Lambda params with Callable expected (callee contracts) | Test: user-defined HOF callback param gets correct SVR | Full lambda transparency heuristic | Not started |
| S8 | Declare lattice inhabitants (DescentEvidence + SubValueRelation) | Test: lattice laws (idempotent, commutative, absorptive) | `merge_argument_relations` now uses `meet_sub_value` | Phase 1: DescentEvidence lifters + SVR meet wired (PR #379). Phase 2: generic lifters (blocked on generic emission). |

### CX-specific

| # | Item | Test | Cleanup target | Status |
|---|------|------|---------------|--------|
| C1 | Direct SubValueRelation → LoweringTarget (bypass CallPattern) | Test: StrictSubValue{ProportionalShrink{2}} → correct LoweringTarget with factor | `sub_value_to_call_pattern` (std/induction.dag:206, lossy bridge) | DONE |
| C2 | Switch annotate_descent to read binding provenance from scope_locals | Test: violation count matches or improves vs current | `classify_argument` fallback (04_infer.dag:2776) | DONE (PR #386) — hybrid: reads binding provenance for ExprVar/ExprFieldAccess, falls back to classify_argument for ExprCall |
| C3 | Validate: binding provenance == CX-L2 reconstruction on all functions | Test: for every function, assert new path agrees with old path | Comparison harness (temporary, deleted after C4) | Not started |
| C4 | Delete annotate_descent_evidence and all reconstruction code | Test: all existing CX tests pass without reconstruction | See cleanup catalog below | Not started |
| C5 | Delete old proof system (construct_termination_proof) | Test: all SCC analysis reads provenance, no fallback | `construct_termination_proof` (~200 lines), `classify_scc_call_progress` (~60 lines) | Not started |
| C6 | Re-enable CX gate as blocking (0 violations) | Test: `strict_compile_diagnostic_count` = 0 | CX gate disable in compile.dag + main.rs exit code filter | Not started — requires Stream D + body inference + arithmetic refinement |

### TDD strategy

**Validation harness (temporary, for C3):** During migration, compute
provenance both ways — at bind time (new) and via CX-L2 reconstruction
(old). Assert they agree on every function. When they agree on all
functions, the old path can be deleted (C4). This is the proof that
the new path is correct.

```
// Pseudocode for comparison test:
for each function in self-compile:
  let new_evidence = read descent_evidence from binding provenance
  let old_evidence = run annotate_descent_evidence (reconstruction)
  assert new_evidence == old_evidence
```

**Test progression:** Each shared step (S1-S8) has a focused test.
As sites are instrumented, the comparison harness (C3) validates
correctness incrementally. The violation count should monotonically
decrease as more sites produce provenance. If it doesn't, the design
needs re-evaluation before proceeding.

**Red flag protocol:** If any step produces provenance that disagrees
with CX-L2 reconstruction on a case where CX-L2 is correct, STOP.
Diagnose whether:
1. The binding site doesn't have enough information (design gap)
2. The provenance composition is wrong (lattice bug)
3. CX-L2 was using a heuristic that can't be expressed structurally
   (indicates the heuristic was unsound — good to discover)

### Cleanup catalog

Code that dissolves as provenance lands. Each entry lists what
deletes it and the estimated line count.

**Dissolves after S3-S6 + C2 (binding provenance replaces reconstruction):**

| Code | File | Lines | Deletes after |
|------|------|-------|--------------|
| `annotate_descent_evidence` + helpers | 04_infer.dag:2183-2930 | ~750 | C4 |
| `classify_argument` (13 heuristics) | 04_infer.dag:2633-2821 | ~190 | C4 |
| `classify_let_value` (5 heuristics) | 04_infer.dag:2413-2582 | ~170 | C4 |
| `classify_collection_shrink` | 04_infer.dag:2258-2348 | ~90 | C4 |
| `classify_size_expr` | 04_infer.dag:2212-2253 | ~40 | C4 |
| `build_call_evidence` | 04_infer.dag:2801-2829 | ~30 | C4 |
| `merge_argument_relations` | 04_infer.dag:2611-2631 | ~20 | S8 (lattice replaces) |
| `DescentContext` type + threading | 04_infer.dag:2201-2208 | ~10 | C4 |
| **Subtotal** | | **~1300** | |

**Dissolves after C5 (old proof system deleted):**

| Code | File | Lines | Deletes after |
|------|------|-------|--------------|
| `construct_termination_proof` | complexity.dag:~2721 | ~200 | C5 |
| `classify_scc_call_progress` | complexity.dag:~3103 | ~60 | C5 |
| `collect_evidence_incremental` | complexity.dag:~2455 | ~100 | C5 |
| `is_child_descent_expr` + helpers | complexity.dag:~1767 | ~80 | C5 |
| `sub_value_to_call_pattern` (lossy bridge) | std/induction.dag:206 | ~15 | C1 |
| **Subtotal** | | **~455** | |

**Dissolves when types move to std/ (Track 7):**

| Code | File | Lines | Deletes after |
|------|------|-------|--------------|
| `is_child_accessor_in_model` | stage0 (v1_std_core.rs) | ~10 | Track 7 |
| `function_size_effects` table | stage0 (v1_std_core.rs) | ~40 | Track 7 |
| `expr_child_roles` table | 00_core.dag | ~50 | Track 7 |
| `node_field_roles` table | 00_core.dag | ~40 | Track 7 |
| **Subtotal** | | **~140** | |

**Total estimated dissolution: ~1700 lines** (net, after accounting
for ~200 lines of classification logic that moves to binding sites).

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
