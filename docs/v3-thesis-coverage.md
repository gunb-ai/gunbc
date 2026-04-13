# v3 Kernel vs. Thesis: Coverage, Gaps, and Emergent Tests

> Part of: [THESIS.md](../THESIS.md) > [src/v3/spec.dag](../src/v3/spec.dag)

Every thesis claim, mapped against the causal kernel. For each:
what the kernel already covers, what's missing, and what test
emerges structurally from the kernel's design.

The pattern for how tests emerge: in a closed system with finite
types, the compiler can generate witness values for any type. For
any subgraph, it can evaluate the kernel directly and compare
against emitted code. Tests are not hand-written — they are
structural consequences of the kernel being closed and bounded.

---

## Coverage matrix

### Legend

- **Covered**: the kernel's structure makes this a direct reading
- **Partial**: the kernel has the machinery but needs explicit wiring
- **Gap**: the kernel doesn't address this yet
- **v3.1**: deliberately deferred

---

### Core abstraction

| Claim | Kernel coverage | Gap | Emergent test |
|-------|----------------|-----|---------------|
| Program IS dependency graph | **Covered.** The kernel IS a DAG. Edges are dependencies. Nodes are causal steps. | None | Any valid .dag source produces a DAG with no cycles. |
| Parallelism is the default | **Covered.** Two nodes with no shared edge are independent. | Needs: the emitter must read independence from the DAG, not from annotations. | For any pair of nodes without a shared edge path, the compiler can emit them concurrently. Concurrent and sequential execution produce the same result. |
| Sequential requires justification | **Covered.** Sequential = edge exists. No edge = independent. | None | Every sequential ordering in emitted code traces back to an edge in the kernel. |

### Tier 1 — Structural correctness

| Claim | Kernel coverage | Gap | Emergent test |
|-------|----------------|-----|---------------|
| Type safety | **Covered.** Every Port has a TypeShape (carrier). Every edge is typed. Interaction output type is determined by ReactionKind + input types. | None | Every Port has a resolved TypeShape (not ErrorType) in a valid program. Every Interaction's output type matches the reaction table's rule for its inputs. |
| Field typos impossible | **Covered.** FieldAccess reaction references a FieldRef with a declaring type. If the field doesn't exist on the type, the reaction is undefined. | None | Every FieldAccess reaction's field exists on the receiver's TypeShape. |
| Non-exhaustive match impossible | **Covered.** Fork requires branches to cover all values of the scrutinee's type. | Needs: exhaustiveness checking as a structural property of Fork (the sum of branch patterns = the scrutinee's variant space). | For every Fork, the union of branch patterns equals the scrutinee type's variant set. Adding a variant to a type without adding a branch is a kernel-level error. |
| Termination | **Covered.** Every Chain has a Bound. The DAG is acyclic. No back-edges. | **Gap: recursive Subgraph calls.** A function that calls itself is a cycle in the call graph. It must lower to a Chain with a structural descent bound. The kernel spec says Subgraphs exist but doesn't specify how recursive calls lower. | Every cycle in the call graph lowers to a Chain. The Chain's bound is a structural fact (tree size, collection size, explicit count). No unbounded cycles exist. |
| Circular dependencies | **Covered.** DAG = no cycles by definition. | None | The kernel graph passes a topological sort. No cycle in edges. |
| Cross-target drift | **Covered.** One kernel, multiple LanguageSpecs. All targets derive from the same kernel. | None | All targets emit from the same kernel. Types agree because they derive from the same TypeShapes. |
| Coercion = emission | **Partial.** Emission translates motifs via LanguageSpec. But coercion (type-to-type conversion) must also be an Interaction in the kernel, not a separate mechanism. | Needs: coercion as a ReactionKind (TypeCast) with the coercion function defined in std/, not the emitter. | Every type coercion is a TypeCast Interaction. The coercion function is a declared .dag function with cost and correctness provable from the kernel. |
| Ownership (no aliased mutation) | **Covered.** Fan-out is edge count from a Port. motif_consumption classifies each input. | **Gap: mutation aliasing proof.** The kernel tracks consumption but doesn't explicitly prove that no two Consumed edges to the same binding exist in the same scope (aliased mutation). | For every binding with fan-out > 1, all consumers except the last are Read or Projected, not Consumed. The last consumer may Consume (move). If two Consume the same binding, that's a kernel error. |

### Tier 2 — Runtime safety

| Claim | Kernel coverage | Gap | Emergent test |
|-------|----------------|-----|---------------|
| Division by zero | **Gap.** The kernel has Interactions with BinaryOp { Div }, but nothing proves the divisor is non-zero. | Needs: carrier refinement. The divisor Port's TypeShape must include a NonZero refinement, or the reaction must be total (return Optional). | Every Div reaction either has a NonZero-refined divisor carrier, or the reaction returns Optional (never panics). |
| Integer overflow | **Gap.** BinaryOp { Add/Mul/Sub } on Int can overflow. Nothing proves the result fits. | Needs: bounded arithmetic. Either prove the result fits in the target word size, or use checked ops that return Optional. | Every arithmetic reaction either has a proven bound on the result, or uses checked ops. |
| Out-of-bounds access | **Gap.** IndexAccess reaction can access beyond collection bounds. | Needs: bounds proof. Either prove the index < length, or make index total (return Optional). | Every IndexAccess either has a proven bounds relationship (index < collection.length), or returns Optional. |
| Force-unwrap | **Gap.** Optional values can be unwrapped without checking. | Needs: make unwrap impossible. Optional access requires Fork (match on Some/None). No .force() or .unwrap(). | No reaction exists that extracts from Optional without a Fork. The only way to access an Optional's value is to Fork on it. |
| Partial functions | **Gap.** Some reactions may be partial (undefined for certain inputs). | Needs: all reactions in the reaction table must be total. Partial operations return Optional. | Every ReactionKind in the reaction table is total. For any valid input, the reaction produces a valid output (possibly Optional). |

### Tier 3 — Verification from structure

| Claim | Kernel coverage | Gap | Emergent test |
|-------|----------------|-----|---------------|
| L4: Emitted code matches .dag evaluation | **Partial.** The kernel is evaluable (closed, bounded, finite). The emitter translates. But the spec doesn't define evaluation. | Needs: a kernel evaluator (interpreter). For any Subgraph, evaluate in the kernel with generated witnesses, execute emitted code, compare. | For every Subgraph, generate witness inputs from the carrier types, evaluate the kernel directly, execute the emitted target code, compare results. Disagreement = emission bug. |
| L5: Cross-language equivalence | **Partial.** Same kernel, different LanguageSpecs. But equivalence must be verified. | Needs: L4 for each target, then cross-compare. | For every Subgraph, emit to Rust/Python/Go, execute all three, compare results. All three must agree with each other and with the kernel evaluation. |
| L6: Exhaustive form coverage | **Covered structurally.** The kernel has 5 motifs and a finite set of ReactionKinds. Exhaustive = every (motif, ReactionKind) pair compiles to every target. | Needs: a coverage matrix that tracks which motif × reaction × target combinations are tested. | For every (motif, ReactionKind, LanguageSpec) triple, at least one test case exists. The test cases are generated from type inhabitants. |
| L7: Algebraic law verification | **Partial.** Reactions have algebraic properties (in the reaction table). But verification requires evaluating laws against reality. | Needs: for each algebraic law (associativity, commutativity, identity, inverse), generate witnesses and check the law. | For every reaction with declared algebraic laws, generate witness values, evaluate both sides of the law, verify equality. e.g., for associativity: `f(f(a,b),c) == f(a,f(b,c))` for generated a,b,c. |

### Correctness dimensions

| Dimension | Kernel coverage | Gap | Emergent test |
|-----------|----------------|-----|---------------|
| **Type safety** | **Covered.** Carrier on every Port. | None | See Tier 1 above. |
| **Termination** | **Partial.** Chains bounded, DAG acyclic. Recursion gap. | Recursive calls must lower to Chain. | See Tier 1 above. |
| **Complexity (cost)** | **Covered.** motif_cost is a fold over the kernel. Cost law per motif. | None — this is the dimension that drove the kernel design. | For any Subgraph, total_cost = fold of motif_cost. Result matches manual analysis. |
| **Ownership** | **Covered.** Fan-out = edge count. motif_consumption classifies inputs. | Mutation aliasing proof (see Tier 1). | For any binding, fan_out is correct. Last-use identification is correct. No double-consume. |
| **Side effects** | **Partial.** EffectShape on DimensionValues (Pure/ServiceCall/Mutation). But effect COMPOSITION is not defined. | Needs: composition rule. Chain of Pure + Pure = Pure. Chain of Pure + ServiceCall = ServiceCall. Fork with one Pure and one ServiceCall = ServiceCall. | For any Subgraph, compose effects of all motifs. The result matches: all-Pure → Pure, any-Service → ServiceCall. Effect composition is monotonic. |
| **Purity** | **Partial.** Purity = all motifs are EffectShape.Pure. But not explicitly tracked. | Purity is a derived reading: if every motif in a Subgraph has EffectShape.Pure, the Subgraph is pure. | For every Subgraph marked Pure, verify no motif inside has ServiceCall or Mutation. |
| **Idempotence** | **Gap.** The thesis says idempotence derives from effect algebra. The kernel doesn't have effect algebra composition yet. | Needs: reaction table entries with algebraic properties. Idempotence = effect is a lattice meet. | For every workflow (Chain of Interactions), compose effect algebra. If all effects are lattice meets, workflow is idempotent. If any is not, show which one. |
| **Space bounds** | **Gap.** The thesis says max allocation is computable from CX. The kernel has cost laws but no space law. | Needs: a space_bound law per motif, analogous to cost. Space = max allocation during this motif's execution. | For any Subgraph, space_bound = fold of motif_space. Result gives max heap allocation. Stack depth = max Chain nesting depth. |
| **User-defined dimensions** | **v3.1.** The kernel has fixed DimensionValues. Generic mechanism deferred. | The thesis says this is "the test of the architecture." If user-defined dimensions require compiler changes, the mechanism is incomplete. | User declares a lattice (e.g., SecurityLevel). Compiler enforces it with the same machinery as built-in dimensions. No compiler changes needed. |

### Concept unifications

| Unification | Kernel coverage | Gap | Emergent test |
|-------------|----------------|-----|---------------|
| Coercion cost = complexity | **Covered.** A type coercion is a TypeCast Interaction. Its cost is reaction_cost(TypeCast). That cost IS the complexity of the coercion. | None | The cost of every TypeCast Interaction matches the CX-proven bound of the coercion function. |
| Coercion = emission | **Covered.** Emission translates kernel to target. TypeCast is an Interaction like any other. | Needs: verify there's no separate "coercion engine" alongside emission. | No code exists outside the emitter that does type-to-type conversion. All conversions are Interactions. |
| Target spec = transport spec = interpreter | **Partial.** LanguageSpec drives emission. Transport specs drive service calls. But the spec doesn't unify these. | Needs: transports are also LanguageSpecs (or both are instances of a common "target spec" type). | Adding a new transport (gRPC, WebSocket) requires adding a spec file, not compiler code. Adding a language requires adding a spec file, not emitter code. Cost of change = 1 file. |
| Idempotency + cancellation + redundancy = algebraic simplification | **Gap.** Reactions can have algebraic properties in the reaction table. But no simplification engine reads them. | Needs: a composition simplifier that reads algebraic laws from the reaction table and simplifies symbolic compositions. | `reverse |> reverse` simplifies to identity (involution law). `serialize |> deserialize` simplifies to identity (inverse pair). `map(f) |> map(g)` simplifies to `map(f . g)` (fusion law). |

### Free consequences

| Consequence | Kernel coverage | Gap | What's needed | Emergent test |
|-------------|----------------|-----|---------------|---------------|
| Automatic parallelism | **Partial.** Independence = no shared edges. But emitter must read this. | Emitter reads DAG structure, emits independent nodes concurrently per target's concurrency model. | Two independent Subgraphs produce the same result whether executed sequentially or concurrently. |
| Automatic memoization | **Gap.** Requires: Purity (all Pure) + cost > threshold + hashable args. | Emitter detects expensive pure Subgraphs, inserts memoization. | A pure Subgraph called twice with same args returns same result (from memo). Cost is amortized. |
| Space bound proofs | **Gap.** Requires space law per motif. | Define motif_space analogous to motif_cost. | For any Subgraph, predicted space bound ≥ actual measured allocation. |
| Cross-language optimization | **Partial.** LanguageSpec per target. Cost known. | Emitter reads target-specific cost model and chooses strategies (inline small folds, parallelize large ones). | Same .dag source, different targets, optimal strategy per target. All produce same result. |

### Error handling

| Claim | Kernel coverage | Gap | Emergent test |
|-------|----------------|-----|---------------|
| Show the correct code | **Gap.** The kernel is closed and finite — the compiler knows all possible fixes. But the spec doesn't address diagnostics. | Needs: for each diagnostic, the compiler generates the corrected .dag source. This is "emission targeted at the developer." | For every diagnostic type, the compiler suggests concrete corrected code. For every NonExhaustiveMatch, the missing arms are generated. For every TypeMismatch, the concrete fix options are listed. |

### Meta-process modeling

| Claim | Kernel coverage | Gap | Emergent test |
|-------|----------------|-----|---------------|
| Bootstrap as .dag | **Gap.** The spec has a bootstrap clause but doesn't integrate it with the kernel. | Bootstrap is a .dag program that operates on the compiler's own kernel (meta-circular). | `dag run bootstrap.dag` produces a converged stage0. CI gate enforces regen → diff → empty. |
| CI as .dag | **Gap.** Not addressed in kernel spec. | CI gates are Subgraphs with dependencies. The compiler reads the DAG and emits CI config (YAML, Actions, etc.). | Adding a CI gate = adding a node to the .dag program. The dependency structure determines execution order. |
| dag run as primary | **Partial.** Interpreter evaluates the kernel directly. | Needs: interpreter reads the kernel, not a separate IR. | `dag run foo.dag` produces the same result as compiling + executing the emitted code. |

---

## The neglected dimensions

The thesis claims 8 correctness dimensions. Here's where each stands
relative to the kernel:

```
                          Kernel   Kernel    Not in
                          covers   partial   kernel
                          ──────   ───────   ──────
Type safety               ██████
Termination                        ██████
Complexity (cost)         ██████
Ownership                          ██████
Side effects                       ██████
Purity                             ██████
Idempotence                                  ██████
Space bounds                                 ██████
```

**Fully covered:** Type safety, complexity. These are the dimensions
you focused on — they drove the kernel design.

**Partially covered:** Termination (recursive calls need to lower to
Chain), ownership (mutation aliasing proof missing), side effects
(composition rule missing), purity (derived from effects but not
explicit).

**Not in kernel:** Idempotence (needs algebraic simplification engine),
space bounds (needs space law per motif).

**Deferred:** User-defined dimensions (v3.1).

---

## How tests emerge from the kernel

In a closed system with finite types, tests are not hand-written.
They are structural consequences:

### Test generation from motifs

For each motif type, the compiler generates test inputs from the
carrier types (all types have known inhabitants in a closed system):

| Motif | What to test | How inputs are generated |
|-------|-------------|------------------------|
| **Constant** | Value is well-typed | Trivial — the constant IS the test. |
| **Interaction** | Reaction produces correct output | Generate all combinations of input carriers. For small types (Bool, small enums): exhaustive. For large types (Int, String): witness sampling. |
| **Fork** | Correct branch activates; exhaustive | Generate all variants of scrutinee type. Each variant activates exactly one branch. |
| **Chain** | Accumulation produces correct result | Generate collections of varying sizes (0, 1, small, boundary). Verify fold/descend/repeat. |
| **Subgraph** | Body evaluates correctly | Generate all combinations of parameter carriers. Compare kernel eval vs emitted execution. |

### Test generation from laws

For each conserved quantity, the law itself generates tests:

| Law | Generated test |
|-----|---------------|
| **Cost** | For a Subgraph, predicted cost ≥ actual measured operations. |
| **Size** | For an Interaction, output size matches size_effect(kind) applied to input size. |
| **Consumption** | For a binding, actual runtime references match predicted fan_out. |
| **Effect** | For a Subgraph, actual side effects match predicted EffectShape. |
| **Algebraic** | For a reaction with declared laws, evaluate both sides with witnesses. |

### Test generation from boundaries

Each thesis tier generates a class of boundary tests:

| Tier | Boundary test |
|------|--------------|
| **Tier 1** | Every structural error (type mismatch, non-exhaustive match, non-termination) is caught at compile time. Generate programs with each error class; verify rejection. |
| **Tier 2** | Every runtime-unsafe operation (div zero, overflow, OOB, force-unwrap) either has a compile-time proof or is total. Generate edge-case inputs; verify no panics. |
| **Tier 3** | For every Subgraph, kernel eval == emitted execution. Generate witnesses; compare across all targets. |

### The meta-test

The ultimate test of the kernel: **can a new correctness dimension
be added without compiler changes?**

Procedure:
1. User declares a lattice in std/ (e.g., SecurityLevel = Public | Secret)
2. User attaches it to their types
3. The compiler enforces it universally

If this works with zero compiler changes, the kernel is general.
If it requires new compiler code, there's a structural gap.

This is the thesis's own acceptance criterion ("This is the test of
the architecture"). It should be a real test in the test suite, not
prose.

---

## Summary: what to work on next

**Don't fixate on one dimension.** The kernel design was driven by
complexity (cost). That's covered. Here's what's not:

1. **Recursive call lowering** — how self-calls become Chains with
   structural descent bounds. This completes termination.

2. **Effect composition** — how effects compose across Chains and
   Forks. This completes side effects, purity, AND idempotence.

3. **Space law** — motif_space analogous to motif_cost. This
   completes space bounds.

4. **Tier 2 totality** — refinement types or total operations for
   div, index, unwrap. This is a separate design from the kernel
   but uses carrier refinement on Ports.

5. **Algebraic simplification** — reading algebraic laws from the
   reaction table and simplifying compositions. This completes
   idempotence, cancellation, and redundancy detection.

6. **Diagnostics as emission** — when the compiler finds a broken
   causal link, emit the fix as corrected .dag code.

Each of these is a FOLD over the same kernel. No new structural
patterns needed. No new motifs. Just new laws and new readings.
That's the test: if adding these requires heuristics, the kernel
is missing something. If they fall out as folds, the kernel is right.
