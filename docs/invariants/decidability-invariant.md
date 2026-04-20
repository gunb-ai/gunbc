## Decidability Invariant

All `.dag` programs are decidable. Undecidable programs are structurally
unrepresentable — the language has no primitive for unbounded computation.

This follows directly from strict forward progress. If every execution
step moves forward through a bounded structure, the computation must
terminate — there are only finitely many steps to take.

This is the highest-leverage invariant in the system. If every function
terminates, then: complexity analysis is total (every function gets a
time and space bound), space analysis is total (peak memory is
computable), the compiler itself is provably terminating on all inputs,
and composition is closed — piping two `.dag` programs together is still
decidable. Without decidability, one unbounded function poisons the
entire pipeline.

### Structural proof from primitives

Decidability is a consequence of the language's modeling primitives,
not a per-function check. The proof has three parts:

**Part 1: All values are finitely constructed.**

Base values are finite: `Bit` has cardinality 2, `Word64` has 2^64.
Every constructor preserves finiteness:

| Constructor | Cardinality | Preserves finiteness |
|---|---|---|
| Product (Conj) | \|A × B\| = \|A\| · \|B\| | Product of finite = finite |
| Coproduct (Disj) | \|A + B\| = \|A\| + \|B\| | Sum of finite = finite |
| Collection append | \|list ++ [x]\| = \|list\| + 1 | Increment of finite = finite |
| Node construction | children is a finite List | Finite list of finite = finite |

There is no constructor for infinite values. A collection of 10^200000
elements is finite — it has a cardinality. The compiler does not care
about the *value* of the cardinality, only that one *exists*.

**Part 2: All iteration is bounded by finite structure.**

The language provides exactly three iteration primitives (see
`std/iteration.dag`):

| Primitive | Bound | What it processes |
|---|---|---|
| `fold` | \|collection\| | Each element of a finite collection |
| `descend` | \|tree\| | Each node of a finite tree (catamorphism) |
| `repeat(N)` | N | Explicit count, N can be up to 2^63 - 1 |

There is no `while(true)`, no unbounded `loop`, no general recursion.
These primitives are the ONLY way to iterate. Each takes a finite
structure or explicit bound and processes it in bounded steps.

**Part 3: Composition preserves boundedness.**

Bounded operations compose to bounded operations:
- Sequential: cost(a; b) = cost(a) + cost(b) — bounded + bounded = bounded
- Nested: cost(fold(list, f)) = |list| × cost(f) — bounded × bounded = bounded
- Conditional: cost(if c then a else b) = cost(c) + max(cost(a), cost(b)) — bounded

No composition of bounded primitives produces unbounded computation.
The primitives are closed under composition. QED.

### Recursive syntax is sugar

Developers write recursive functions for readability. The compiler
lowers every call pattern to a bounded primitive:

| Call pattern in recursive function | Lowers to | Why it's bounded |
|---|---|---|
| Self-call on child of input | `descend` (catamorphism) | Bounded by \|tree\| |
| Self-call inside `fold` body | Already bounded by fold | Fold bounds the iteration |
| Self-call with `n - 1` | `repeat(n, ...)` | Bounded by n |
| Mutual recursion (SCC) on children | `descend` over SCC | Bounded by \|SCC\| |
| Self-call with unchanged argument | `repeat(Forever, ...)` | Bounded by 2^63 - 1 |

No call pattern is rejected. The last row uses the bounded truth
principle: in a Bit/Word64 system, "forever" is a finite bound
(2^63 - 1 iterations). `repeat(Forever)` is not an approximation of
infinity — it is the correct answer for the largest representable
iteration count. See `std/computation.dag` (CallPattern →
LoweringTarget) and `std/iteration.dag` for the full model.

### Fail-closed compilation

Decidability is enforced at two levels:

1. **Structural (construction):** The language has no unbounded iteration
   primitive. Every call pattern maps to exactly one bounded primitive
   via the exhaustive lowering table in `std/computation.dag`.

2. **Fail-closed (compilation):** If the compiler encounters a call
   pattern it cannot classify (a gap in the classifier, not in the
   model), compilation fails with a hard error. This is a safety net —
   it catches analyzer incompleteness. In a correct implementation,
   this error is unreachable because the lowering table is exhaustive.

The complexity analyzer does not enforce decidability — it derives cost
formulas from the bounded structure that the language guarantees. If the
analyzer produces `?O(?)`, the bug is in the analyzer (it cannot see the
bound that structurally exists), not in the program.

### Tight upper bounds — no exceptions

Every function and expression in the language must have a **provably
tight** upper bound. `Conservative` certainty is a modeling deficit,
not acceptable steady state. `Unknown` certainty is a hard error.

### Cost algebra is upstream of language primitives

The cost algebra (`CostExpr`) is the **upstream authority** that
determines what the language can express, not a downstream attempt to
describe what the language already does.

```
Cost algebra defines expressible cost classes
    ↓
Language primitives must declare a cost from the algebra
    ↓
Complexity analyzer reads the declaration — trivially correct
```

A language primitive cannot be added until its cost class exists in
the algebra. This is the same structural guarantee as decidability:
just as bounded primitives make undecidability unrepresentable, the
cost algebra makes unanalyzable primitives unrepresentable.

**The current `sort_by` gap is this deficit in action.** `sort_by` was
added without `CostLog` in the algebra. The analyzer falls back to
Conservative O(n) — valid but not tight. The fix is not "add CostLog
later." The fix is: the algebra must have `CostLog` before `sort_by`
can exist. Adding the primitive without its cost class violates the
modeling order.

**The contract:** for any `.dag` program P, the complexity analyzer
produces bound B such that B is the exact tight bound for P. No
`Conservative`. No `Unknown`. Every function is `Proven`. This is
guaranteed by construction because every primitive declares its cost
in the algebra, and the algebra can express that cost exactly.

**The principle:** the cost algebra and language primitives are
co-designed, with the algebra leading. When someone proposes a new
primitive, the first question is: "what is its cost class, and can
the algebra express it?" If not, the algebra grows first. The
primitive follows.

### Practical ergonomics

Decidable does not mean small. The bound can be astronomically large:

```
// Server that handles requests for 292 million years (at 1 req/ms)
fn serve(handler: fn(Request) -> Response) {
  repeat(bound: max_int, f: (_) => handler(accept_request()))
}

// Process with generous safety margin
fn process_batch(items: List<Item>, safety_factor: Int) {
  repeat(bound: items |> count * safety_factor, f: process_next)
}
```

Developers think "serve forever" or "process with margin." The compiler
sees bounded iteration. Same program, different semantics. The developer
gets smooth ergonomics. The compiler gets total analysis.

### Closure property

If someone builds a DSL on top of `.dag`, that DSL is also decidable.
The DSL is composed from `.dag` primitives, which are all bounded.
There is no escape hatch. To express unbounded computation, someone
would need to invent a new modeling language from scratch — they cannot
reach it by composing `.dag` primitives.

### Lowering table

Every recursive pattern has a bounded iterative equivalent:

| Recursive pattern | Structural bound | Bounded lowering |
|---|---|---|
| Tree walk (visit children) | \|nodes\| (strict child descent) | `descend` over tree structure |
| Tokenizer loop (advance pos) | \|source\| (monotonic advance) | `fold` over characters with position |
| Accumulator recursion | decreasing counter or list length | `repeat(n, ...)` or `fold` with init + step |
| Mutual recursion (A↔B on children) | \|SCC\| with shared measure | `descend` over SCC-ordered nodes |
| Long-running process | explicit bound | `repeat(bound: N)` with N up to 2^63 - 1 |

Graph-like properties (cycles, unbounded iteration, general recursion)
are not expressible in the core language. Recursive syntax is surface
sugar that the compiler lowers to these bounded forms.

