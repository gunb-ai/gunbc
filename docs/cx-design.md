# CX: Complexity Analysis Design

Stable principles for the complexity analyzer. If a change violates
one of these, stop and update this doc first — then assess the impact
on the codebase to determine if the direction is viable.

See also: `cx-computation-model.md` (migration state),
`cx-violation-triage.md` (current violation snapshot).

---

## P1: Descent is a type-level fact

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

## P2: The analyzer reads, never derives

The complexity analyzer is a **consumer** of structural facts, not a
producer. Every fact it needs — which parameters carry recursive
structure, which expressions descend, which calls are bounded — should
be annotated on the IR by upstream stages (resolve/infer).

Today the analyzer re-derives descent from AST pattern-matching. This
is the root cause of every heuristic extension: each new code pattern
requires a new recognizer. The fix is not better recognizers — it's
making the pipeline produce the facts so the analyzer just reads them.

**Implication:** `collect_descent_vars`, `collect_evidence_incremental`,
`classify_self_call_evidence`, `expr_contains_descent` — the entire
pattern-matching apparatus — should dissolve. The analyzer reads
descent witnesses from the IR the same way it reads `method_semantics`.

## P3: No rejected patterns — but Forever factors out

Every call pattern has a finite bound. `self(same_arg)` →
`repeat(Forever)`. The analyzer computes bounds; it never rejects.
CostUnknown means "missing fact," not "invalid program."

**However:** Forever is not "just a large O(1)." It conflates two
orthogonal questions:

1. **Scalability** — how does cost grow with input size? (`O(n)`, `O(n²)`)
2. **Termination character** — is the bound input-dependent or degenerate?

`O(Forever)` is technically constant w.r.t. input — but calling an
infinite loop O(1) is useless analysis. The purpose of complexity
analysis is to tell the user whether their algorithm is *sustainable
as input grows*. Forever is independent of input, but it's not
sustainable.

**The resolution:** Forever is a **separate dimension**, factored out
of the scalability report. A bound like `Forever × n` does not
"simplify to Forever" — it decomposes into:

- **Termination character:** runs forever (intentional: server loop,
  event loop, retry loop)
- **Per-iteration scalability:** O(n) in tree size

The user cares about the second part. A server that processes requests
in O(n) per request is fundamentally different from one that processes
in O(n²), even though both run "forever." The Forever dimension is
factored out because the input-dependent portion always dominates
actual computation time within each iteration.

**Implication:** The bound algebra must support factoring. A bound is
a product of dimensions, some input-dependent, some degenerate. The
scalability report extracts the input-dependent portion. Forever is
honest (not hidden or rejected) but separated from the analysis the
user actually needs.

## P4: Global descent, not strict monotonicity

The current analyzer requires every self-call to strictly decrease
some measure. This is unnecessarily strict. Real programs take
detours — a function may temporarily preserve or even increase a
measure before eventually decreasing it.

**What matters:** the computation *eventually* terminates — global
descent. Not that every individual step goes strictly down — local
monotonicity.

Examples of valid global descent that strict monotonicity rejects:
- `render_node_type(n: with_required_cardinality(n: n))` — tree size
  stays flat, but cardinality changes so the guard prevents re-entry.
  One "detour," bounded by 1.
- Worklist algorithms: adding items (ascent) is fine as long as the
  worklist eventually drains (global descent).
- Amortized patterns: occasional expensive operations that are paid
  for by many cheap ones.

**The requirement:** over every complete cycle of the computation,
some well-founded measure decreases. Individual steps may not
decrease, as long as no infinite non-decreasing sequence exists.

**Representation options:**
1. **Lexicographic tuples** — `(cardinality_depth, tree_size)` decreases
   lexicographically even when tree_size stays flat. Already partially
   in the analyzer but not wired to condition changes.
2. **Size-change termination** — track size relationships across ALL
   call paths automatically. If every infinite call sequence would
   violate some well-founded ordering, done. No user annotation needed.
3. **Explicit progress witness** — user declares `progress: cardinality`
   on the function, compiler verifies that dimension decreases over
   every cycle.

**Open question:** how to make this expressible without a math PhD.
The user needs to say "this globally descends" and the compiler needs
to verify it — no cheating allowed, but no unnecessary strictness
either.

**Phasing:** Start with strict descent — it handles the vast majority
of real patterns (tree walks, list consumption, parser advancement,
arithmetic decrease). Design the abstraction so it can be relaxed
later. The interface between pipeline and analyzer should be an opaque
proof object: "does this descend? here's the witness." Today the
witness = single-dimension strict descent. Tomorrow it could be
lexicographic, size-change, or amortized. The analyzer asks "is there
a valid proof?" and doesn't care how it was constructed.

`TerminationProof { dimensions: List<RankingDimension> }` already has
roughly this shape. The structure accommodates growth — it's the
*production* of proofs that's currently heuristic (pattern-matching
in the analyzer instead of facts from the pipeline).

## P5: Complexity classes are emergent

Linear, quadratic, Forever are emergent properties of the bound
algebra. The analyzer doesn't classify — it computes a bound, and
the complexity class falls out.

**Implication:** No `classify_complexity` function. The bound IS the
classification. `O(n)` emerges from `bound = TreeSize(param)`.
`O(n*m)` emerges from nested iteration bounds.

## P6: Recursion is emergent

Recursion is not a first-class concept to model or analyze. It falls
out of functions calling each other. Only iteration primitives
(fold/descend/repeat) are modeled explicitly because they are
intentional developer constructs.

The analyzer never tries to "prove recursion terminates" — it computes
the tightest bound it can from structural facts about data and
operations.

---

## Design direction: descent witnesses as pipeline facts

The pipeline already produces structural facts at each stage:

| Stage | Fact | Example |
|-------|------|---------|
| Parse | `expr_data` discriminant | ExprLet, ExprCall |
| Resolve | `inferred` type | Resolved { node } |
| Infer | `method_semantics` | AlgebraMethodSemantics |
| Infer | `is_self_recursive` | true/false |

Descent evidence should follow the same pattern: produced by
resolve/infer, consumed by the complexity analyzer.

**What the analyzer needs per recursive function:**
1. Which parameters carry recursive structure (derivable from type)
2. Which expressions are strict sub-values of those parameters
3. Whether all self-call arguments descend

(1) is a type-level query. (2) is an expression annotation produced
during inference. (3) is the verification the analyzer performs by
reading (1) and (2).

**Open questions:**
- Where exactly in the pipeline should descent annotations be produced?
- What IR representation carries them? (New field on Node? On ExprData?)
- How do annotations compose through let-bindings, match arms, method chains?
- How does this interact with the existing `method_semantics` annotation?

---

## Current state (reference)

524 violations. 46 direct self-recursive, 185 composed (callers of
direct unknowns). Full triage in `cx-violation-triage.md`.

The existing analyzer infrastructure partially works:
- CX-C (line 2683-2706 in complexity.dag) enters iteration lambdas
  when the receiver is structural children — handles simple cases
  like `node.children |> map(ch => self(ch))`
- Breaks down when: accessors aren't registered, field access chains
  are complex, condition-dependent termination (with_required_cardinality)

Migration plan in `cx-computation-model.md` describes 4 phases of
incremental cleanup. That plan is operational (how to get there), not
architectural (what "there" looks like). This doc is the architecture.
