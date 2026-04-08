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

## P2: The analyzer reads .dag structure, never derives from AST

The complexity analyzer is a **consumer** of structural facts, not a
producer. Every fact it needs — which parameters carry recursive
structure, which expressions descend, which calls are bounded — should
be represented as .dag structure (types, edges, functions) produced by
upstream stages (resolve/infer).

**Not annotations.** The Modeling Faithfulness Invariant (INVARIANTS.md)
requires facts to be .dag structure, not metadata. Descent evidence is
modeled as .dag values (`DescentEvidence`, `TerminationProof` from
`std/termination.dag`), not as IR annotations on nodes.

Today the analyzer re-derives descent from AST pattern-matching. This
is the root cause of every heuristic extension: each new code pattern
requires a new recognizer. The fix is not better recognizers — it's
making the pipeline produce .dag-typed descent facts that the analyzer
reads.

**Implication:** `collect_descent_vars`, `collect_evidence_incremental`,
`classify_self_call_evidence`, `expr_contains_descent` — the entire
pattern-matching apparatus — should dissolve. The analyzer reads
descent witnesses the same way it reads `method_semantics`.

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

## Roadmap to launch gate

### Existing foundation

| Asset | Lines | Role |
|-------|-------|------|
| `std/termination.dag` | 275 | DescentEvidence, RankingDimension, TerminationProof, DescentSource — well-designed, keep |
| `std/iteration.dag` | 163 | fold/descend/repeat conceptual model — keep |
| `std/computation.dag` | 369 | CallPattern → LoweringTarget — keep |
| Pipeline facts | — | `is_self_recursive`, `AlgebraMethodSemantics`, `recursive_type_set`, `RecursiveVariantFieldWitness` already produced by inference |
| `complexity.dag` | 4965 | Heuristic analyzer — replace, not extend |
| `00_core.dag` tables | — | `node_field_roles`, `expr_child_roles`, `function_size_effects` — dissolve into type-derived facts |

### CX-L1: Type-level recursive field detection

**Where:** Inference already does this.

**What already exists:** The pipeline detects recursive types and
tracks which fields carry self-references:

- `TypeEnv.recursive_type_set: Map<String, Bool>` — which types
  are self-referential (detected via dependency cycle analysis)
- `TypeEnv.recursive_variant_fields: Map<String, List<RecursiveVariantFieldWitness>>`
  — which variant fields reference the parent type
- `RecursiveVariantFieldWitness { variant_name, field_name }` — e.g.,
  for `type MyList<T> = Nil | Cons { head: T, tail: MyList<T> }`,
  witness is `{ variant_name: "Cons", field_name: "tail" }`
- `is_recursive_type(env, name)` — queries the set

**Recursive types in .dag:** The bounded kernel invariant says Node
is the only recursive type in the **compiler IR**. But users CAN
define recursive types (`MyList<T>`, binary trees, etc.) — these are
represented as Node trees in the IR, and the compiler tracks their
recursion via `recursive_type_set`. Test: `generic_recursive_type`
in pipeline.rs.

**What CX-L1 adds:** Map the existing `RecursiveVariantFieldWitness`
to ranking dimensions for complexity analysis.

| Witness pattern | Dimension |
|-----------------|-----------|
| Field type == parent type | TreeSize |
| Field type == `List<parent>` | TreeSize (children-style) |
| Field type == `parent?` | TreeSize |

**Dissolves:** `node_field_roles`, `expr_child_roles` (for descent
purposes), `function_size_effects` — these hand-maintained tables
are replaced by the type-derived `RecursiveVariantFieldWitness`.

### CX-L2: Descent witness production in inference

**Where:** Inference (after method semantics are assigned).

**What:** For each self-recursive function, track sub-value
relationships from parameters through expressions. When a self-call
argument is a provable sub-value of a parameter on a recursive
dimension, annotate it with `Strict` evidence.

Standard patterns to recognize (strict descent):
```
// Direct iteration over recursive field
param.children |> fold(init: ..., f: (child, acc) => self(child))

// Child accessor extraction
let child = accessor(param)    // accessor projects from children
self(child)

// Match on recursive field
match param.body { Some { value: inner } => self(inner) }

// Arithmetic decrease
self(n: n - 1)
```

**Key property:** the witness is produced by the pipeline, not
discovered by the analyzer. Adding a new accessor or field requires
zero changes to the analysis — the type declaration IS the authority.

**Output:** Each self-call site annotated with descent evidence per
parameter. Stored on the IR (design TBD: new field on Node, on
ExprData, or in a side table).

### CX-L3: Bound computation and reporting

**Where:** New, small analysis pass after inference.

**What:** For each self-recursive function:
1. Read descent witnesses from CX-L2
2. If all self-calls have `Strict` evidence on some dimension →
   bound = that dimension's measure
3. If not → bound = Forever (honest "couldn't prove with strict descent")
4. Report as user-facing diagnostic: `info: f is O(n) in tree_size`

**Forever factoring (P3):** For functions with Forever bounds
containing input-dependent sub-computation, factor out the Forever
and report the per-iteration scalability separately.

**Size:** This pass should be small — hundreds of lines. It reads
annotations, it doesn't pattern-match AST. The complexity is in
CX-L2 (producing witnesses), not here (consuming them).

### CX-L4: Cost composition (aspirational)

For functions calling other functions, compose their bounds:
- `f` calls `g` where `g` is `O(n)` → f's cost includes g's bound
- Nested: `fold` body calls recursive function → multiply bounds
- Report: `f is O(n × m)` for nested iteration

### CX-L5: Suboptimality detection (aspirational)

Hand-curated equivalence catalog: pattern → cheaper alternative.
Even 3-5 rules would be a compelling demo.

| Pattern | Suggestion | Why |
|---------|------------|-----|
| `xs \|> filter(p) \|> count > 0` | `xs \|> any(p)` | O(n) → O(k), short-circuits |
| `xs \|> map(f) \|> filter(p)` | `xs \|> filter_map(f, p)` | One pass instead of two |
| `xs \|> sort \|> first` | `xs \|> min` | O(n log n) → O(n) |

This does not need to be architecturally complete — a small catalog
is valuable. The catalog lives in `std/` as structural data, not as
analyzer heuristics.

### Launch gate = CX-L1 + CX-L2 + CX-L3

The hard gate is phases 1-3. User-written functions using standard
recursion patterns get proven complexity bounds via type-derived
strict descent. The existing `complexity.dag` (4965 lines) continues
to run non-blocking for internal compiler metrics — it is not on the
launch path.

---

## Current state (reference)

524 violations in the compiler's own code (internal debt, not
user-facing). 46 direct self-recursive, 185 composed. Full triage
in `cx-violation-triage.md`.

The existing `complexity.dag` analyzer partially works but is built
on heuristic pattern-matching that grows with each new code pattern.
It is not the launch architecture — CX-L1/L2/L3 above replace it
for user-facing analysis.

Migration plan in `cx-computation-model.md` describes incremental
cleanup of the existing analyzer. That plan is operational (how to
get there), not architectural (what "there" looks like). This doc
is the architecture.
