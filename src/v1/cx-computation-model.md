> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > **Tier 1: Structural** (CX gate)
> See also: [cx-design.md](cx-design.md), [cx-violation-triage.md](cx-violation-triage.md)

# CX Computation Model

Design doc for the complexity analyzer. Single source of truth for
modeling decisions, design principles, architectural gaps, and
design directions.

## Core Model (approved 2026-04-04)

All computation desugars to three bounded iteration primitives:
**fold**, **descend**, **repeat**. No call pattern is rejected. The
compiler always succeeds in lowering; the question is not "does this
terminate?" but "what is its bound?"

Authoritative files:
- `dsl/std/iteration.dag` — declares the three primitives
- `dsl/std/computation.dag` — maps syntax to primitives (lowering table)
- `dsl/std/termination.dag` — proves termination via ranking functions
- `dsl/std/induction.dag` — type-level recursive structure (InductiveField)

### The 7 system concepts

Everything the system understands:

1. **Node/DAG** — structural carrier (the ONLY recursive type)
2. **Product/Coproduct** — structural connectives
3. **Bit** — truth value, finite (2 inhabitants)
4. **Ring/Field/BooleanAlgebra/FreeMonoid** — value algebra
5. **fold/descend/repeat** — iteration (THE bottleneck)

Everything else is sugar. A developer writes `while`/`for`/`self`/`+`/`-`/`list.map`
and the compiler lowers it all to the above.

### CallPattern → LoweringTarget (exhaustive)

| CallPattern              | Lowers to      | Bound                 |
|--------------------------|----------------|-----------------------|
| ChildAccessorCall        | descend        | TreeSize              |
| CollectionShrinkCall     | fold           | CollectionSize        |
| ArithmeticDescentCall    | repeat         | ArithmeticParam       |
| ParserAdvanceCall        | fold           | CollectionSize(tokens)|
| WorklistDrainCall        | fold           | CollectionSize(set)   |
| FoldBodyCall             | (already fold) | (inherited)           |
| SameArgumentCall         | repeat         | Forever               |

No pattern is rejected. `SameArgumentCall → repeat(Forever)` is the
bounded truth principle: in a Bit/Word64 system, "always" = 2^63-1
iterations. True is a Bit, not infinity.

### Cost = product of bounds

Nested computations multiply bounds:
```
fold(list, fn(acc, x) { descend(tree, ...) })
→ O(|list| × |tree| × per_node_cost)
```

---

## Four-tier proof architecture

| Tier | What happens | Authority | Implementation |
|------|-------------|-----------|----------------|
| **0: Types** | Types declare recursive structure | std/ type declarations | InductiveField, RecursionShape in std/induction.dag |
| **1: Emergence** | Call graph reveals cycles | SCC detection | complexity.dag SCC analysis |
| **2: Evidence** | Observe what happens to arguments at cycle edges | CX-L2 annotation | annotate_descent in 04_infer.dag → SubValueRelation |
| **3: Classification** | Map evidence to proven primitive | Lowering table | CallPattern → LoweringTarget in std/computation.dag |

### Design principles

**Recursion is emergent.** Users write functions; the compiler discovers
cycles (emergence), observes that arguments shrink at cycle edges
(evidence), and classifies each pattern as fold/descend/repeat (proof).
The primitives are proof vocabulary, not user-facing API.

**All programs are bounded.** All data is ultimately quantifiable
(Bit/Word64). The analyzer reports HOW bounded, not WHETHER bounded.
Forever < infinity — it is a concrete finite quantity.

**No rejected patterns.** Every call pattern has a finite bound. CX
violations mean "the analyzer couldn't derive a bound from available
facts" — the fix is providing the missing fact.

**Complexity classes are emergent.** Linear, quadratic, Forever emerge
from numbers and arithmetic. As number modeling gets richer, classes
emerge with greater precision without analyzer changes.

**CX heuristics are CM gaps.** Every name-matching classifier is a
symptom of a missing concept in std/. Model the fact, let the property
emerge, the heuristic dissolves.

**Descent is a type-level fact.** The type definition `children: List<Node>`
IS the descent fact. The analyzer reads from InductiveField declarations,
not from accessor function names.

**Fail-closed on unknown descent.** DescentUnknown (bottom of lattice)
is a hard error. The analyzer never approximates.

---

## Evidence system: current model and compositional deficit

### Current model (point-wise)

The evidence system classifies ONE argument at ONE call site:

```
classify_argument(arg_expr, param_name, ctx) → SubValueRelation
```

SubValueRelation variants:
- **StrictSubValue** — field access produces smaller tree
- **IteratedSubValue** — fold/map element is sub-value of collection
- **ArithmeticDescent** — param - k or param / k
- **PreservedValue** — same value, no descent
- **SubValueUnknown** — cannot determine (fail-closed)

The termination proof is: for every self-call, at least one parameter
has Strict/Iterated/Arithmetic evidence. For SCCs, every cycle through
the member graph has at least one Strict edge.

### Compositional deficit (identified 2026-04-10)

The remaining 250 violations (down from 472) cluster into categories
that each represent a DIFFERENT structural fact the evidence system
can't see. The number of categories is a smell — if the model were
right, violations would dissolve from one or two concepts, not seven.

The deficit: **the evidence system is point-wise (one parameter, one
call site) when the proofs require relational reasoning (across
parameters, across function boundaries, across state).**

| Category | Violations | What's missing |
|----------|-----------|----------------|
| Lambda transparency | ~105 | Evidence doesn't flow through function boundaries |
| Condition-guarded | ~40 | Evidence about property changes, not data size |
| Multi-dimension | ~21 | Termination from different parameters cooperating |
| Bounded ascent | ~20 | Relationship between two parameters (pos < length) |
| Cycle detection | ~7 | Accumulated state (seen map grows toward finite domain) |
| WorklistDrainCall | ~10 | State contraction (set removal) |
| Scattered gaps | ~47 | Individual missing patterns in classify_argument |

All of these are instances of the same gap: the evidence system
classifies the relationship between ONE argument and ONE parameter
at ONE call site. The proofs that fail require:

- **Cross-boundary evidence:** lambda parameters inherit descent from
  the caller's context (the callee extracts sub-values and passes to
  the callback)
- **Cross-parameter evidence:** termination comes from a PAIR of
  parameters (keys shrinks while template varies; pos increases but
  pos < length)
- **State transition evidence:** the combined (param1, param2, ...)
  state decreases on some well-founded measure, even if individual
  parameters don't

### Design direction: state-transition evidence

A function call is a state transition. Input = (param1, param2, ...),
output = (arg1, arg2, ...) of the recursive call. Termination = some
component of the combined state strictly decreases, none increases
unboundedly.

In this framing:
- **Lambda transparency dissolves.** A lambda `x => self(n: x)` passed
  to `f(texpr: texpr, callback: ...)` is a composed transition:
  f extracts sub-values of texpr → passes to callback → callback
  passes to self. Evidence composes through the lambda boundary.
- **Multi-dimension dissolves.** (keys shrinks, template varies) is
  a single transition that's well-founded because the keys component
  is bounded.
- **Bounded ascent dissolves.** (pos increases, length stays) is
  well-founded when the pair is considered: pos < length.

The call graph already composes through function boundaries (SCC
detection recurses into lambda bodies). The evidence system doesn't.

### Open question: how to implement compositional evidence

Two approaches under consideration:

**Callee contract declarations.** Higher-order functions declare
contracts about how they invoke callback parameters:
```
type CallbackContract {
  function_name: String
  callback_param: String
  source_param: String
  relation: SubValueRelation
}
```
This extends the existing AlgebraMethodSemantics pattern. Contracts
are declared in std/, checked once, consumed by callers. Localized
change to annotate_descent.

**Full state-transition model.** Instead of per-parameter evidence,
track combined state transitions across function boundaries. More
compositional but requires rethinking the evidence lattice from
SubValueRelation (per-param) to TransitionEvidence (all params).

The callee contract approach is a stepping stone toward the full
model — it addresses lambda transparency with minimal disruption.
The full model may be needed for multi-dimension and bounded-ascent
categories.

---

## Concrete modeling decisions (ledger)

### Decided

| Decision | Rationale | Where |
|----------|-----------|-------|
| ProgressKind → DescentEvidence | Parallel lattice with bridge functions was redundant | PR #370. Deleted ProgressKind, bridge functions, merged into std/termination.dag |
| SCC-wide advancement assumption | Standard SCC termination: assume property, check consistency | PR #370. All SCC members treated as always-advancing when computing parser edges |
| PropertyContraction on FunctionSizeEffect | Finite property domain (cardinality) bounds condition-guarded recursion | PR #370. with_required_cardinality: PropertyContraction { domain_size: 2 } |
| resolve_collection_field for skip/take/filter/reverse | Collection-preserving methods preserve InductiveField through chains | PR #370. render_node_type dissolved (n.children |> skip(1) |> first) |
| Prefer tree descent over arithmetic in evidence selection | Prevents false branching guard trigger when mixed evidence | PR #370. classify_recursion_pattern picks StrictSubValue before ArithmeticDescent |
| promote_to_strict in std/termination.dag | Fail-closed helper: preserves `Strict` / `NonIncreasing`, maps other positions to `DescentUnknown` (no unary upgrade to `Strict`) | PR #370; `std.termination` authority. Parser progress in `complexity.dag` uses `strict_after_parser_witness` where a parser witness justifies `Strict` (PR #682+); other call sites may still route through `promote_to_strict` until witness threading dissolves that hop |

### Open (need design)

| Question | Options | Blocking |
|----------|---------|----------|
| How should lambda parameters carry evidence? | Callee contracts vs. full state-transition model | ~105 SCC violations |
| How to model bounded ascent (pos + 1 < length)? | Cross-parameter evidence, external bound parameter | ~20 tokenizer violations |
| How to model cycle detection (seen map)? | SetCardinality dimension, or finite-domain contraction | ~7 resolve_scrutinee violations |
| Should CostExpr flatten to product of SizeBounds? | Phase 4 of original migration plan | Deferred (requires stable evidence system) |

### Deferred (to types-in-std)

| Item | Dissolves when |
|------|---------------|
| compiler_recursive_types parallel fact table | All types move to std/; recursive_type_set derives from inductive_fields |
| Hardcoded collection-preserving method names | Method semantics come from std/ type declarations |
| classify_let_value takes first ListRecursion by order | Types declare projection facts (which field a function accesses) |

---

## Current state (2026-04-10)

**Ratchet: 472 → 424 (10% dissolved, honest). PR #370.**

Note: ratchet was at 168 before reverting 5 unsound evidence shortcuts
(review feedback). The 424 count is honest — all remaining violations
represent genuine gaps in the compositional evidence model.

| Phase | From→To | Key insight |
|-------|---------|-------------|
| Phase 1: CX-L2 evidence | 472→435 | Literal skip/take, negation, nested patterns, ExprCall scrutinees |
| Phase 2: size_effects | 435→425 | wrapper_inner_arg, extractor_inner_arg as TreeSizeReducing |
| Phase 3: Parser SCC | 425→315 | SCC-wide advancement assumption breaks circular dependency |
| Phase 4a: PropertyContraction | 315→308 | Finite-domain property changes bound condition-guarded recursion |
| Phase 4b: Collection methods | 308→250 | resolve_collection_field for skip/take/filter/reverse |
| Phase 5a: Lattice-max bug | 250→208 | derive_edge_evidence fold took last value, not best |
| Phase 5b: Map lambda evidence | 208→168 | Preserve outer sub_value_vars through map/filter (not fold) |

### Remaining 168 by category

| Category | Count | Root cause | Design direction |
|----------|-------|------------|-----------------|
| TCO emitter SCCs | 33 | TcoFrame wraps texpr, breaking direct param relationship | Compositional evidence through constructors |
| CX self-analysis SCCs | 25 | walk_expr, resolve_expr_types, parser_state_expr_progress | Same — cross-call names don't match |
| Tokenizer | 20 | Bounded ascent (pos + 1 < source_len) | BoundedAscent evidence or fold-over-stream |
| Parser worklist | 18 | parse_exit_entries_acc, parse_mock_response_entries_acc | State advancement not modeled |
| resolve_node_bounded | 16 | depth + 1 bounded by hard limit (100) | BoundedAscent or tree descent on n param |
| Inference SCC | 11 | infer_block_stmts — heterogeneous SCC (list + tree) | Compositional evidence |
| dfs/graph | 11 | dfs_finish_order — visited set bounds recursion | WorklistDrainCall (SetCardinality) |
| resolve_scrutinee_type_node_seen | 7 | Cycle detection via seen map growing toward finite domain | Finite-domain contraction |
| infer_parser worklist | 7 | Worklist pattern — infer_parser_always_advancing_members | State advancement |
| node_to_name_str | 5 | String/tree recursion on Node structure | Likely needs tree descent evidence |
| Scattered | 15 | type_needs_rc_seen(2), process_escapes_loop(2), ceil_log_iter(2), etc. | Various |

### Why the remaining 168 won't dissolve from targeted patches

The remaining violations cluster into patterns that all share the same
root cause: **the evidence system is point-wise** (one parameter, one
call site, matched by name) when the proofs require compositional
reasoning.

**Specific structural gaps:**

1. **Cross-call name mismatch.** `build_call_evidence` aligns evidence
   to the caller's `param_order` by matching arg names. Cross-calls use
   different names (e.g., caller has `texpr`, callee expects `expr`), so
   the evidence is SubValueUnknown even when the VALUE is a sub-value.
   A scan-all-args fallback was attempted and reverted — it's a
   heuristic that fabricates evidence. The right fix is compositional:
   evidence should compose through the call boundary, not scan for
   structural coincidences.

2. **Constructor wrapping.** TCO functions wrap `texpr` in `TcoFrame`
   before passing to callbacks. The evidence system can't see through
   the constructor to the wrapped parameter.

3. **Bounded ascent.** Tokenizer/parser functions advance `pos + 1`
   bounded by `source_len`. The evidence system models descent
   (param - k) but not ascent bounded by an external parameter.

4. **Heterogeneous SCCs.** Some SCC members descend on tree (texpr),
   others on list (remaining). The lexicographic proof requires the
   leading dimension to be known on ALL edges. A cycle through both
   dimensions has NonIncreasing on both at the cross-edges.

**Do NOT attempt to fix these with more patterns in classify_argument
or more entries in function_size_effects.** Each remaining violation
represents a genuine gap in the compositional model. Targeted patches
will either violate invariants (fail-closed, no fallbacks) or create
unsound evidence.

### What to do instead

The gate-first strategy: get to 0 by modeling the right concepts, then
let gate violations guide refinement. The remaining 168 need one or
more of:

- **Callee contracts** — higher-order functions declare what they pass
  to callbacks (dissolves lambda transparency + constructor wrapping)
- **State-transition evidence** — combined (param1, param2, ...)
  transitions, not per-param (dissolves heterogeneous SCCs)
- **BoundedAscent** — ascending position bounded by external param
  (dissolves tokenizer + parser worklist)
- **Lambda desugaring** — long-term: dissolve ExprLambda into regular
  functions (deferred — 35 handlers, 927+ usages, large scope)

See "Compositional deficit" and "Design direction" sections above for
the full analysis.

### Exhaustive lowering pipeline status

| Stage | Coverage | Status |
|-------|----------|--------|
| SubValueRelation → CallPattern | 5/5 variants | sub_value_to_call_pattern — complete |
| CallPattern → LoweringTarget | 7/7 variants | lower_call_pattern — complete |
| annotate_descent ExprData | 11/21 variants handled + catch-all | Gap 1 partially addressed |
| SCC proof dimensions | 2 lexicographic combos + 5 single | Gap 2 partially addressed |
| SetCardinality evidence | 0 paths produce it | Gap 3 open |
| ArithmeticValue in SCC proofs | Single-function only | Gap 4 open |
| Lambda transparency | Iteration + callback lambdas | Gap 5 — partially addressed, needs callee contracts |

---

## Relationship to other docs

- **ROADMAP.md §CX** — work items, sequencing, acceptance criteria
- **INVARIANTS.md** — bounded kernel, fail-closed, decidability
- **MODELING.md** — DSL modeling philosophy (shared facts, concept DAG)
- **cx-violation-triage.md** — violation breakdown by callee (stale: based on 526, current is 168)
- **src/v1/DESIGN.md** — compiler design principles
