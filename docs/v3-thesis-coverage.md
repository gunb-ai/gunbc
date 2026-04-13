# v3 Kernel vs. Thesis: Gap Analysis

> Part of: [THESIS.md](../THESIS.md) > [src/v3/spec.dag](../src/v3/spec.dag)

## Premise

A closed system with finite types and bounded iteration should
provide every thesis property as a structural reading of the
kernel — the same way MapReduce provides parallelism by
desugaring computation to map + reduce. MapReduce didn't add a
"parallelism pass." It showed that parallelism IS the structure
when computation is expressed in the right primitives.

The v3 kernel has 5 causal motifs: Constant, Interaction, Fork,
Chain, Subgraph. Every .dag expression desugars to these. The
thesis claims 8 correctness dimensions, 4 concept unifications,
4 free consequences, 3 verification tiers, and several structural
guarantees.

This document shows, for each gap, exactly how it resolves to a
law on the existing motifs — no new motifs, no separate passes,
no heuristics.

---

## 1. Termination — recursive calls as Chains

### The gap

The kernel says Chains have bounds and the DAG has no cycles. But
a recursive function (`fn f(x) { ... f(smaller_x) ... }`) is a
cycle in the call graph. How does it lower to the acyclic kernel?

### How it resolves

A recursive call is a compressed Chain where the bound comes from
structural descent. The compiler already knows (from the kernel):

- The call site is an Interaction (FunctionCall)
- The target is the same Subgraph (self-reference)
- The argument has Provenance.SubValue (it's structurally smaller)

The lowering is:

```
fn count_nodes(tree: Node) -> Int {
  1 + tree.children |> fold(init: 0, fn: (acc, child) => acc + count_nodes(child))
}
```

Desugars to:

```
Chain {
  source: tree (the structure being walked)
  bound: TreeSize { tree }           // |tree| — structural descent
  body: Interaction { kind: BinaryOp { Add }, inputs: [acc, 1] }
}
```

The bound is `|tree|` because each recursive call processes a
strict sub-value (child ⊂ tree). The Provenance on the child Port
says `SubValue { parent: tree, relation: StrictSubValue }`. That
IS the bound — it's already in the kernel.

**No separate termination analysis.** Termination = every self-
referencing Subgraph lowers to a Chain. The Chain has a bound.
The bound comes from Provenance on the argument Ports. If no
argument has SubValue provenance, the call can't lower to a Chain,
and the compiler rejects it — not as "complexity unknown" but as
"this expression doesn't desugar to the kernel."

### What this means for v2

v2's 420 CX violations exist because the compiler computes
Provenance, discards it in TypeBinding, then reconstructs it in
complexity.dag with 33 heuristics. In the kernel, Provenance
lives on the Port. The Chain lowering reads it. Zero heuristics.

### Emergent test

Every recursive Subgraph produces a Chain with a valid Bound.
Generate a recursive function with known structure, verify the
Chain's bound matches the structural depth.

---

## 2. Side effects — composition along the topology

### The gap

The kernel has EffectShape (Pure / ServiceCall / Mutation) in
DimensionValues, but doesn't define how effects compose across
motifs.

### How it resolves

Effects compose along the DAG topology. The composition is a
lattice join — effects only escalate, never decrease:

```
Pure < ServiceCall < Mutation

compose(a, b) = max(a, b)  (lattice join)
```

Per motif:

```
Constant:    effect = Pure (known bits, no action)
Interaction: effect = reaction_effect(kind)  // from reaction table
Fork:        effect = join(branch_effects)   // either branch might fire
Chain:       effect = body_effect            // repeated N times, same effect class
Binding:     effect = Pure (naming, no action)
Subgraph:    effect = body_effect            // computed recursively
```

This is a fold:

```
fn subgraph_effect(sg: Subgraph) -> EffectShape {
  fold_motifs(sg.body, init: Pure, fn: (acc, motif) => join(acc, motif_effect(motif)))
}
```

**No separate effect pass.** Effect = fold of motif_effect over
the kernel. The reaction table declares each reaction's effect.
The fold composes. Pure functions are those whose fold returns Pure.

### What falls out for free

**Purity** — a Subgraph is pure iff its effect fold returns Pure.
No separate purity analysis. Purity IS `subgraph_effect(sg) == Pure`.

**Effect ordering** — if two adjacent motifs both have ServiceCall
effects, and neither has a data dependency on the other, the effects
are independent. But ServiceCalls may not commute (calling A then B
might differ from B then A). The compiler reads the effect kind
(the specific service) and checks commutativity from the reaction
table. If the services are independent, the calls can be parallelized.
If not, the data edge constrains ordering.

### Emergent test

For every Subgraph, the predicted effect matches the actual runtime
behavior. A Subgraph predicted Pure produces no observable side
effects when executed. A Subgraph predicted ServiceCall produces
exactly the declared service interactions.

---

## 3. Idempotence — reading algebraic properties from the reaction table

### The gap

The thesis says idempotence derives from effect algebra: an
operation whose effect is a lattice meet is idempotent. The kernel
doesn't have algebraic simplification.

### How it resolves

The reaction table already has a slot for algebraic properties.
Idempotence is one such property:

```
Reaction table:
| Kind | Cost | Size | Effect | Algebra |
|------|------|------|--------|---------|
| MapUpsert | O(log n) | preserving | Mutation | lattice_meet |
| MapDelete | O(log n) | reducing | Mutation | lattice_meet |
| ListAppend | O(1) | expanding | Mutation | monoid (not idempotent) |
```

A Chain is idempotent iff every Interaction in its body has a
`lattice_meet` algebra. The check is a fold:

```
fn is_idempotent(chain: Chain) -> Bool {
  fold_motifs(chain.body, init: true, fn: (acc, motif) =>
    acc && motif_algebra(motif) == LatticeMeet
  )
}
```

**Cancellation** (`f ∘ f⁻¹ = id`) is the same mechanism: the
reaction table declares inverse pairs. A composition of a reaction
with its inverse simplifies to identity.

**Redundancy** (`f₁ ∘ ... ∘ fₙ = g where cost(g) < cost`) is the
same mechanism: the compiler reads algebraic laws from the table,
symbolically composes, and checks if a cheaper equivalent exists.

**All three** — idempotence, cancellation, redundancy — are
readings of the reaction table's algebraic properties. One
mechanism, not three.

### The merge_envs case

v2's `merge_envs(a, a, a)` bug: merge is a lattice meet. Merging
a value with itself = the value (idempotency). The compiler would
read: `merge` has algebra `lattice_meet`. Three inputs are the
same Port (fan-out = 3 from one source). `lattice_meet(x, x, x) = x`.
Compile-time simplification: the merge is redundant. The 68x speedup
becomes a compile error instead of a latent hot spot.

### Emergent test

For every reaction with declared algebraic laws, generate witnesses
and verify the laws hold. For idempotence: `f(f(x)) == f(x)`. For
inverse pairs: `f(f_inv(x)) == x`. For fusion: `map(f, map(g, x)) == map(f.g, x)`.

---

## 4. Space bounds — a space law per motif

### The gap

The thesis says max heap allocation is computable from CX. The
kernel has cost laws but no space law.

### How it resolves

Space is another conserved quantity, analogous to cost. Each motif
has a local space fact:

```
fn motif_space(node: KernelNode) -> SpaceBound {
  match node {
    Constant { value, carrier } =>
      sizeof(carrier)                         // the constant's storage

    Interaction { kind, inputs, output } =>
      reaction_space(kind)                    // from reaction table
      + sizeof(output.carrier)                // the output allocation

    Fork { branches, .. } =>
      max(branches |> map(b => space(b.body)))  // worst-case branch

    Chain { body, bound, .. } =>
      if chain_reuses_accumulator(body) {
        space(body)                           // O(1) extra per iteration (reuse)
      } else {
        bound_value(bound) * space(body)      // O(N) if allocating per iteration
      }

    Binding { .. } => 0                       // naming, no allocation
    Subgraph { body, .. } => space(body)      // body's space
  }
}
```

Total space = fold of motif_space. Stack depth = max Chain nesting
depth (each Chain is one stack frame in recursive lowering).

**This completes space bound proofs.** Stack overflow prevention =
max Chain nesting depth < stack limit. Memory budget = fold of
motif_space < budget. Embedded deployment = prove program fits in
N bytes. All structural readings.

### Emergent test

For any Subgraph, predicted space ≥ actual measured allocation at
runtime. Run the Subgraph with witness inputs, measure peak
allocation, compare against predicted bound.

---

## 5. Runtime safety (Tier 2) — carrier refinement

### The gap

Division by zero, integer overflow, out-of-bounds, force-unwrap,
partial functions. None addressed.

### How it resolves

Every Port has a carrier (TypeShape). Carrier refinement narrows
the type to exclude dangerous values. The reaction table declares
preconditions:

```
Reaction table:
| Kind | Precondition on inputs |
|------|----------------------|
| BinaryOp { Div } | input[1].carrier includes NonZero |
| IndexAccess | input[1].carrier includes InBounds(input[0].length) |
| OptionalUnwrap | FORBIDDEN — must use Fork instead |
```

If a Port's carrier doesn't satisfy the precondition, the
Interaction is undefined. The compiler rejects it — not as a
runtime error, but as "this expression doesn't lower to the kernel."

**Division:** `a / b` requires `b: NonZero<Int>`. If b comes from
user input, you must Fork on `b == 0` first. The non-zero branch
carries `NonZero<Int>`. The division lives inside that branch.

**Index:** `xs[i]` requires `i: InBounds<xs.length>`. If i comes
from computation, you must prove `i < xs.length` (which may be
a Fork or a prior bounds check that narrows the carrier).

**Force-unwrap:** doesn't exist. Optional values go through Fork
(match Some/None). The Some branch carries the inner value. No
unwrap reaction in the table.

**Totality:** every reaction in the table is total. For any valid
input carriers, the reaction produces a valid output. If the table
has no entry for a given input combination, that expression doesn't
lower to the kernel.

### The closed system advantage

In an open system, refinement types are hard because you can't
know all possible values at compile time. In a closed system:
- All base types are finite (Bit/Word64)
- All containers are bounded
- All iterations are bounded

So the compiler can actually prove refinement predicates
statically. `b != 0` after a Fork on `b == 0` is a structural
fact about the Fork — the non-zero branch carries the proof.

### Emergent test

For every reaction with a precondition, generate inputs that
violate the precondition. Verify the compiler rejects them.
Generate inputs that satisfy the precondition. Verify execution
produces no panics.

---

## 6. Parallelism — reading independence from the DAG

### The gap

The thesis says parallelism is structural. The kernel has
independence (no shared edges) but the emitter must read it.

### How it resolves

Two nodes are independent iff there is no directed path between
them in the DAG. Independence is a topological property — a
reading of the graph, not an analysis.

For emission, the compiler partitions the DAG into waves:
- Wave 0: all nodes with no predecessors
- Wave 1: all nodes whose predecessors are all in wave 0
- Wave N: all nodes whose predecessors are all in waves < N

Nodes in the same wave are independent. The emitter reads the
target's concurrency model (from LanguageSpec) and emits:
- Rust: `rayon::join` or `tokio::spawn` for independent nodes
- Go: `go func()` for independent nodes
- Python: `multiprocessing.Pool` for CPU-bound, `asyncio.gather`
  for IO-bound
- CI: parallel jobs in the same workflow step

**MapReduce falls out here.** A Chain whose body is pure and whose
combining function is associative+commutative (declared in the
reaction table's algebra slot) can be partitioned:

```
fold(items, init, combine)
  where combine.algebra = CommutativeMonoid

→ partition items into chunks
→ fold each chunk independently (parallel)
→ combine chunk results (parallel reduce)
```

The compiler reads:
1. It's a Chain (iteration primitive)
2. The body is Pure (effect fold = Pure)
3. The combine is associative + commutative (reaction table)

MapReduce emission follows. No "parallelism pass." No annotations.
The algebraic properties in the reaction table + purity from the
effect fold + the Chain structure = automatic MapReduce.

### Emergent test

For every pair of independent nodes, executing them in either
order (or concurrently) produces the same result. For every
Chain with CommutativeMonoid combine, sequential fold and parallel
map-reduce produce the same result.

---

## 7. Memoization — reading purity + cost

### The gap

Pure functions with known cost can be memoized. The kernel has
purity and cost but doesn't connect them to memoization.

### How it resolves

Memoization is an emission optimization, not a kernel concept. The
emitter reads:
1. Subgraph is pure (`subgraph_effect == Pure`)
2. Cost exceeds threshold (`subgraph_cost > memoization_threshold`)
3. Parameter carriers are hashable (in a closed system with finite
   types, ALL types are hashable)

If all three: emit a memoization wrapper. The wrapper is a
LanguageSpec-specific translation (Rust: `HashMap<Args, Result>`,
Python: `functools.lru_cache`, Go: `sync.Map`).

No kernel change. No annotation. The emitter reads existing facts
(purity, cost, carrier types) and makes a rendering decision.

### Emergent test

A memoized pure Subgraph called twice with same args returns the
same result. The second call is O(1) (cache hit). Total cost =
first_call_cost + O(1), not 2 × first_call_cost.

---

## 8. Algebraic simplification — symbolic composition from the reaction table

### The gap

The thesis claims `reverse |> reverse = id`, `serialize |> deserialize = id`,
`map(f) |> map(g) = map(f.g)`. The kernel doesn't have a simplifier.

### How it resolves

The reaction table declares algebraic properties per reaction:

```
| Reaction | Algebra |
|----------|---------|
| ListReverse | involution (f ∘ f = id) |
| Serialize | has_inverse: Deserialize |
| Deserialize | has_inverse: Serialize |
| ListMap | functor (map(f) ∘ map(g) = map(f ∘ g)) |
| FoldWithMonoid | monoid_homomorphism |
```

The simplifier reads the DAG, identifies adjacent Interactions
whose algebraic properties allow simplification, and rewrites:

1. **Involution:** two adjacent identical involutions → identity
   (delete both nodes, connect predecessor to successor)

2. **Inverse pair:** reaction followed by its inverse → identity

3. **Functor fusion:** two adjacent maps → one map with composed
   body

4. **Monoid homomorphism:** fold of monoid → parallelizable

The simplifier is a graph rewrite pass that reads the reaction
table. It's generic — it doesn't know about `reverse` or
`serialize`. It knows about involutions and inverses. Adding a
new simplification = adding an algebraic property to a reaction's
table entry.

### Emergent test

For every declared algebraic property, generate a composition that
should simplify. Verify the simplified version produces the same
result as the original but with lower cost.

---

## 9. Verification (Tier 3) — the closed system makes tests free

### The gap

L4-L7 are not implemented. Tests are hand-written.

### How it resolves

In a closed system with finite types, the compiler knows every
possible value of every type. For any Subgraph, it can:

1. **Enumerate inputs.** For small types (Bool: 2 values, small
   enums: N variants), enumerate exhaustively. For large types
   (Int: 2^64 values), sample witnesses at boundaries (0, 1, -1,
   max, min) and random points.

2. **Evaluate the kernel directly.** The kernel is executable
   (it's a bounded DAG — walk it). The interpreter does this today.

3. **Emit to each target.** The emitter produces target code.

4. **Execute and compare.** Run the emitted code with the same
   inputs. Compare outputs.

This is not "testing." It's **emission verification** — proving
the translation is faithful. The kernel IS the oracle.

| Test level | What it proves | How it generates |
|------------|---------------|-----------------|
| L4 | Emitted code matches kernel eval | For each Subgraph: generate inputs → eval kernel → exec emitted → compare |
| L5 | Cross-language equivalence | Same as L4, but exec Rust + Python + Go, all must agree |
| L6 | Form coverage | For each (motif × ReactionKind × target): at least one generated test exists |
| L7 | Algebraic laws hold | For each declared law: generate witnesses → evaluate both sides → compare |

**Test generation is a fold over the kernel.** For each Subgraph,
the fold generates: input witnesses from carrier types, expected
output from kernel evaluation, and verification assertions. The
test suite is a CONSEQUENCE of the kernel, not a separate artifact.

### Emergent test

The test generator itself can be tested: for every generated test,
the kernel evaluation and the emitted execution agree. If they
disagree, either the emitter has a bug or the kernel evaluator has
a bug. Both are testable because the system is closed.

---

## 10. Diagnostics — emission targeted at the developer

### The gap

The thesis says the compiler should show the fix, not just the error.

### How it resolves

A diagnostic is a broken causal link in the kernel. In a closed
system with finite types, the compiler knows the COMPLETE set of
valid repairs:

| Broken link | Repair options | How the kernel finds them |
|-------------|---------------|--------------------------|
| Type mismatch at Port | Change the source type, change the consumer type, insert a TypeCast Interaction | Enumerate TypeCast reactions from the reaction table that bridge the gap |
| Non-exhaustive Fork | Add the missing branches | Compute: scrutinee type's variant set minus existing branch patterns |
| Non-terminating recursion | Identify which argument should descend | Check each argument Port's carrier for SubValue provenance opportunities |
| Division by zero | Insert a Fork on `== 0` before the division | Generate the Fork structure with NonZero carrier on the non-zero branch |
| Missing field in RecordConstruct | Show the missing fields with their types | Compute: target type's field set minus provided fields |

The fix IS emission: the compiler emits corrected .dag code to the
terminal. This uses the same emission machinery as target code
generation — just targeted at .dag syntax instead of Rust/Python/Go.

**Corrected .dag emission is a LanguageSpec.** The .dag language
is itself a target. The compiler reads the .dag LanguageSpec and
emits corrected source. The diagnostic IS an artifact.

### Emergent test

For every diagnostic type, generate a program with that error.
Verify the compiler suggests a fix. Apply the fix. Verify the
fixed program compiles.

---

## 11. User-defined dimensions (v3.1 — but the design must accommodate it)

### The gap

The thesis says this is "the test of the architecture." If user-
defined dimensions require compiler changes, the mechanism is
incomplete.

### How it resolves (design, not implementation)

A user-defined dimension is a new law on the kernel. The existing
mechanism:

1. **Carrier** — the type/state space on each Port
2. **Law** — a local fact per motif, foldable over the kernel

A user-defined dimension adds a law. Example: SecurityLevel.

```dag
// User declares in their std/:
type SecurityLevel = Public | Internal | Secret

// User declares the law:
fn security_law(motif: KernelMotif) -> SecurityLevel {
  match motif {
    Constant { value, .. } => classify_constant(value)  // data classification
    Interaction { kind, inputs, .. } =>
      max(inputs |> map(i => i.security))               // output ≥ max input
    Fork { branches, .. } =>
      max(branches |> map(b => security(b.body)))
    Chain { body, .. } => security(body)
    Binding { value, .. } => value.security
    Subgraph { body, .. } => security(body)
  }
}

// User declares the constraint:
// Secret data cannot flow to a Port whose consumer is a ServiceCall
// with transport = PublicAPI
fn security_check(port: Port, consumer: KernelMotif) -> Diagnostic? {
  if port.security == Secret && consumer.effect == ServiceCall { transport: PublicAPI } {
    SecurityViolation { source: port, drain: consumer }
  }
}
```

The compiler reads the law and folds it over the kernel — the same
way it folds cost, effect, and space. No compiler change. The user
just declared a new conserved quantity.

### The test

User defines SecurityLevel. Writes a program that passes Secret
data to a public API. The compiler catches it with zero compiler
modifications. The enforcement uses the same fold machinery as
termination, cost, and effects.

---

## Summary: thesis claim completeness

Every thesis claim maps to one of:

1. **A motif** — the structural shape (5 total)
2. **A law** — a local fact per motif, foldable (cost, size, effect,
   space, algebra, consumption)
3. **A reading** — a fold of laws over the kernel (complexity,
   ownership, purity, idempotence, parallelism, memoization)
4. **An emission strategy** — translating the kernel to target syntax
   using LanguageSpec data (including diagnostics as .dag emission)
5. **A carrier refinement** — narrowing Port types to exclude
   dangerous values (Tier 2 safety)

No thesis claim requires a new motif. No thesis claim requires a
separate analysis pass. No thesis claim requires heuristics.

The v3 kernel's 5 motifs + local laws + fold mechanism is
sufficient to express every claim the thesis makes. The gaps are
not structural — they are unwritten laws and unimplemented folds
on a kernel that already has the right shape.

```
Thesis claim                      Resolves to
──────────────────────────────────────────────────────
Type safety                       carrier on Ports
Termination                       recursive call → Chain lowering
Complexity                        motif_cost fold
Ownership                         edge fan-out + consumption
Side effects                      motif_effect fold (lattice join)
Purity                            effect fold == Pure
Idempotence                       reaction table algebra
Space bounds                      motif_space fold
Tier 2 safety                     carrier refinement on Ports
L4-L7 verification                kernel eval vs emitted exec
Coercion = emission               TypeCast is an Interaction
Algebraic simplification          reaction table rewrite rules
Parallelism                       DAG independence + algebra
MapReduce                         Chain + Pure + CommutativeMonoid
Memoization                       Pure + cost threshold
Diagnostics                       emission to .dag LanguageSpec
User-defined dimensions           user-declared laws + fold
```
