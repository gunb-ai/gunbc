# Dependency Graph, Ownership, and Parallelism — Unified Design

> **Parent docs:** `THESIS.md`, `INVARIANTS.md` §"Facts Flow
> Forward", `src/v3/SELF_HOSTING.md` §14.7.
>
> **Purpose:** the DAG's `produced_by` edges + behavior input
> lists define a complete dependency graph. Ownership,
> parallelism, complexity, provenance, and dead-code detection
> are all projections of this one structure. This doc defines
> the shared foundation and each projection.
>
> **Status:** design in progress. Core model (read vs construct)
> validated by implementation (72 → 6 clones). Open edges
> remain around transitive ownership transfer and fold
> accumulator linearity (§3.5, §3.6). Needs review.

---

## §1. The dependency graph is already in the DAG

The dependency information is never lost. It's created at
lowering and persists unchanged through inference to emission:

```
Source:   let b = a + 3       // "a" is a name reference
Parse:    SurfaceExpr::Var("a")   // name string in AST
Lower:    Transform(+, inputs=[a_port, 3_port])
          output.produced_by = this Transform
          // name reference is now a PORT EDGE
Infer:    walks same edges, adds types. No edges added/removed.
Emit:     walks same edges, renders code.
```

The raw dependency graph is the DAG itself:
- **Forward edges:** behavior.inputs = which ports this node reads
- **Backward edges:** port.produced_by = which node produced this value
- Both are structural, explicit, and complete for pure code

No analysis creates this graph. Lowering creates it. It persists.

---

## §2. What needs to be INDEXED (not computed)

The raw graph is complete but not indexed for efficient access.
The emitter needs derived views that require walking the raw
edges. These are INDEXES over existing structure:

| Derived view | What it answers | How to build |
|---|---|---|
| **Consumer list** | Who reads this port? | Scan all behaviors' input lists |
| **Consumer count** | How many readers? | Length of consumer list |
| **Use kind** | Does this consumer read or construct? | Classify from behavior type (§3) |
| **Last use** | Is this the final consumer? | Evaluation order + consumer count |
| **Transitive deps** | Does C depend on A? | Walk produced_by backward |
| **Independence** | Are A and B unrelated? | No transitive path between them |
| **Copy classification** | Is cloning free for this type? | Read is_copy from realization spec |

Built during the emitter's existing index pass (alongside
`RealizationIndex`). Read at every render site.

---

## §3. Projection 1: Ownership (read vs construct)

### §3.1 The primary classification

The first-order model: every consumer edge is one of two kinds:

| Consumer behavior | Use kind | Why |
|---|---|---|
| Transform input (function argument) | Read | Pure function reads its arguments |
| Branch scrutinee | Read | Inspects value to choose a path |
| Match pattern destructure | Read | Reads tag + fields |
| Field access (FieldProject) | Read | Projects one field |
| Comparison operand | Read | Inspects value |
| Record literal field | **Construct** | Value becomes part of a new record |
| List cons element | **Construct** | Value becomes part of a new list node |
| Function return value | **Construct** | Value becomes the function's output |

In a pure language, ALL function parameters are reads at the
IMMEDIATE level — the caller passes the value, the callee
inspects it.

### §3.2 Target rendering for read vs construct

```dag
type RenderingModel {
  read: ReadStrategy
  construct: ConstructStrategy
}

type ReadStrategy = Borrow | PassByValue
type ConstructStrategy = CopyOrClone | PassByValue
```

```dag
// rust.dag
data rust_rendering: RenderingModel = {
  read: Borrow           // &T
  construct: CopyOrClone  // copy if Copy, else move/clone
}

// go.dag
data go_rendering: RenderingModel = {
  read: PassByValue
  construct: PassByValue
}
```

### §3.3 Construction rendering for Rust

At a Construct site, the decision depends on:

| is_copy(T) | Last use? | Rendering | Cost |
|------------|-----------|-----------|------|
| true | any | `*value` (deref) | 0 (bits copied) |
| false | yes | `value` (move) | 0 (ownership transfer) |
| false | no | `value.clone()` | O(size) |

`is_copy` is declared per-type in the realization spec.
`last_use` is derived from the dependency index.

### §3.4 Validated result (Phase 1)

Implementation (PR #475) applied this model to the emitter.
Generated lens clone count: **72 → 6**. The 6 remaining:

| Line | Clone | Category | Correct? |
|------|-------|----------|----------|
| 31 | `fold_acc.clone()` | Fold accumulator | **Unnecessary** — see §3.5 |
| 79 | `span.clone()` | SourceSpan (non-Copy) at record construction | **Correct** |
| 131 | `fold_acc.clone()` | Fold accumulator | **Unnecessary** — see §3.5 |
| 134 | `fold_acc.clone()` | Fold accumulator | **Unnecessary** — see §3.5 |
| 203 | `list_head.clone()` | PortId (Copy type) | **Unnecessary** — see §3.6 |
| 217 | `list_head.clone()` | PortId (Copy type) | **Unnecessary** — see §3.6 |

Expected after fixing §3.5 and §3.6: **1 clone** (the
SourceSpan construction, which is genuinely necessary).

### §3.5 OPEN EDGE: Transitive ownership transfer

**Problem.** The read-vs-construct model classifies IMMEDIATE
consumers. But some function calls are effectively constructs
from the caller's perspective — the callee embeds the argument
in its return value.

Example:
```dag
let result = cons(transform(x), acc)
```

`acc` is passed to `cons` — a function call, classified as
Read. But `cons` internally constructs a new list node
containing `acc` as the tail. The emitter sees "function
argument = read = borrow" but the function STORES the value.

**Why this matters.** If the emitter borrows `acc` (`&acc`)
and passes it to `cons`, but `cons` needs to own the tail
to store it in the new Cons node, the emitted Rust won't
compile — you can't store a `&T` where a `T` is needed
without cloning.

**The general pattern: ownership transfer through calls.**
A function call is a "transitive construct" if:
1. The callee's return value transitively contains the
   parameter (the parameter port has a path to the return
   port through a construct site inside the callee's body)
2. The caller uses the return value (it's not dead)

In these cases, calling the function effectively TRANSFERS
OWNERSHIP of the argument into the return value. The caller
should pass by value (move), not by reference (borrow).

**Known instances:**
- `cons(head, tail)` — both arguments end up in the return
  Cons node
- `fold` accumulator — the closure's return value IS the
  next accumulator, which may contain the input acc
- Any function that wraps its argument in a record and
  returns it

**What makes this tricky.** For .dag functions, the DAG
shows whether a parameter flows to the return value through
a construct site. But for ExternalRealization functions
(Rust builtins), the DAG doesn't show internals. The
realization spec would need to declare "this parameter is
consumed" vs "this parameter is borrowed."

**Possible approaches:**

*Option A: Conservative — borrow everything, clone at
construct sites inside callees.* This is what the emitter
does now. It produces unnecessary clones inside functions
like `cons` (clone the borrowed tail to store it). Safe but
suboptimal.

*Option B: Callee-signature analysis — mark parameters as
"owned" or "borrowed" in the function's Arrow declaration.*
If a parameter flows to the return value through a construct,
the parameter is "owned" and callers must pass by value. This
is similar to Rust's own fn signature model where parameters
are either `T` (owned) or `&T` (borrowed).

*Option C: Realization-declared ownership — each
ExternalRealization declares which parameters are consumed.*
Like Rust's type system but declared in the spec, not inferred.

**Open question:** which approach? Option B is the most
principled (derive from the DAG for .dag functions, declare
for ExternalRealization). Option A is the safest starting
point (no risk of missing a necessary clone).

### §3.6 OPEN EDGE: Copy types and deref vs clone

**Problem.** The emitter generates `.clone()` on `&PortId`
and `&NodeId` when these types implement Rust's `Copy` trait.
For Copy types, `*value` (dereference) is equivalent to
`.clone()` but more idiomatic and makes the intent clear
(this is a trivial bit-copy, not a deep clone).

**Fix (straightforward).** The realization spec declares
`is_copy: true` per type. The emitter checks this and emits
`*value` instead of `value.clone()` for Copy types. This is
not an open modeling question — it's a missing fact in the
realization spec.

### §3.7 OPEN EDGE: Fold accumulator linearity

**Problem.** Rust's `iter().fold(init, |acc, x| body)` takes
`acc` by value. The closure OWNS the accumulator, mutates it,
and returns it. This is a Rust-specific rendering fact — the
fold's semantics say "consume old acc, produce new acc."

The emitter currently generates:
```rust
|acc, item| {
    let mut left = acc.clone();  // UNNECESSARY — acc is owned
    left.extend(...);
    left
}
```

Correct rendering:
```rust
|mut acc, item| {
    acc.extend(...);
    acc
}
```

**Is this fold-specific or general?** The user asked whether
this generalizes beyond fold. The answer: the fold accumulator
is an instance of **Rust's closure ownership model.** When
Rust's `fold` takes `FnMut(B, Item) -> B`, the closure owns
`acc`. This is a TARGET-LANGUAGE rendering fact.

The general question is: **are there other call patterns where
the target language's calling convention gives the callee
ownership of an argument?** In Rust:
- `fold` accumulator: owned by closure
- `map` element in `into_iter().map(f)`: owned by `f`
- Any `FnOnce(T)` parameter: owned
- Move closures capturing outer values: owned

For .dag, these are rendering decisions — the source language
doesn't distinguish owned vs borrowed parameters. The emitter
needs to know which Rust calling conventions give ownership.
This is part of the rendering strategy in `rust.dag`.

**Possible approach:** extend `CallableRealization` in
`rust.dag` with parameter ownership annotations:

```dag
data rust_fold: CallableRealization = {
  strategy: ListFold
  acc_ownership: Owned    // closure takes acc by value
  element_ownership: Borrowed  // element is &T
}
```

---

## §4. Projection 2: Parallelism

### §4.1 Independence from the graph

Two ports with no transitive `produced_by` path between them
are independent. In a pure language, independent operations
are ALWAYS safe to run in parallel.

This is NOT an analysis. It's a structural property of the
DAG. The dependency index makes it efficient to check.

### §4.2 Fold decomposition

A fold's body lambda has `acc` and `x` parameters. The
dependency index shows which body nodes transitively depend
on `acc`:

- **acc-independent nodes** → "map" part (parallelizable)
- **acc-dependent nodes** → "reduce" part (sequential)

If all per-element work is acc-independent, the fold IS a
map. The compiler reports this as a structural fact.

### §4.3 Target rendering

```dag
type ParallelismStrategy {
  parallel_map: String
  parallel_reduce: String
  sequential: String
}
```

### §4.4 Effects boundary

For ExternalRealization operations (side effects), the effects
lens classifies operations. Independent + pure → parallel.
Independent + effectful → needs sync. Future work (L2 M3).

---

## §5. Projection 3: Complexity

The dependency graph's longest chain = the program's inherent
sequential cost (critical path). Total work minus critical
path = parallelizable work. `lens_cost` already walks
`produced_by` edges. With the dependency index, it can also
report critical path length and parallelizable fraction.

Ownership feeds into complexity: clone = O(n) cost, move =
O(1), borrow = O(1). The dependency index is computed first,
ownership reads it, complexity reads both.

---

## §6. Projection 4: Provenance + dead code

**Provenance:** one-hop backward — `port.produced_by`.
Already `lens_provenance`. Trivial with the shared index.

**Dead code:** `consumer_count == 0` and not a function
return → dead. Emitter skips it.

---

## §7. Safety: negative testing for missing clones

**THIS IS CRITICAL.** If the ownership model incorrectly
classifies a construct as a read, the emitter will borrow
where it should clone. The emitted Rust may:
- Not compile (best case — Rust's borrow checker catches it)
- Compile but produce wrong results (worst case — data shared
  when it should be independent)

In a pure language, the second case SHOULD be impossible
(immutable values can't produce wrong results from sharing).
But Rust's `Vec` is mutable — if the emitter borrows a Vec
and the borrower modifies it through interior mutability
(unlikely but possible in generated code), the result is
wrong.

**Required negative tests:**

1. **Every generated artifact compiles with rustc.** If the
   ownership model is wrong, Rust's borrow checker catches
   most errors. This is the primary safety net.

2. **Roundtrip execution matches.** The generated lens must
   produce identical output to the handwritten oracle. If
   a missing clone changes behavior, the parity test catches
   it.

3. **Intentional over-clone test.** Generate the same program
   with "clone everything" and "use ownership model." Both
   must produce identical output. If the ownership model
   changes behavior, this test catches it.

4. **Boundary tests for each ownership decision:**
   - Borrow a value used by two readers → both see same data
   - Move a value at last-use construct → callee has ownership
   - Clone a non-Copy value at non-last construct → both
     copies are independent
   - Deref a Copy value → value preserved correctly

5. **Stress test: nested constructs.** A record containing a
   record containing a list — verify the ownership model
   handles depth correctly and doesn't miss an inner clone.

---

## §8. What v2 reconstructed vs what falls out

| Fact | v2 (719 lines) | v3 (indexed from DAG) |
|---|---|---|
| Who produces? | Walk ExprData backward | `port.produced_by` |
| Who reads? | Walk tree, count names | `DependencyIndex.consumers` |
| Read or construct? | Not modeled | Classify from behavior type |
| Last use? | Not modeled | Evaluation order + consumer count |
| Independent? | Not modeled | No path in graph |
| Fold = map? | Not modeled | Body doesn't reach acc |
| Copy type? | Hardcoded heuristics | `is_copy` in realization spec |
| Critical path? | Not modeled | Longest chain in graph |
| Ownership transfer through calls? | Not modeled | **OPEN — §3.5** |

---

## §9. Phasing

| Phase | When | What |
|---|---|---|
| **Phase 1** | L1.5 | Build `DependencyIndex` during emitter index pass. Classify use kind (read/construct). Add `is_copy` per type in rust.dag. Emitter renders `&T` for reads, `*value` for Copy constructs, `.clone()` for non-Copy non-last constructs. Conservative on §3.5 (clone at transitive construct sites). |
| **Phase 2** | L2 | Resolve §3.5 (transitive ownership transfer). Fold accumulator ownership annotation (§3.7). Last-use tracking (move at final construct). Expected: clone count → 1 on generated lens. |
| **Phase 3** | L2 M1+ | Parallelism detection tests. Complexity reads dependency index. Effects-aware parallelism (L2 M3). Dead-code skipping. |
| **Phase 4** | L3 | Self-analysis. Clone count at zero on generated compiler code. Parallel emission for independent stages. |

---

## §10. Testing approach

**Dependency structure tests (testable NOW):**

1. Direct dependency: `let a = 1; let b = a + 1` → b
   depends on a
2. Transitive: `let a = 1; let b = a+1; let c = b+1` → c
   depends on a
3. Independence: `let a = 1+2; let b = 3+4` → independent
4. Diamond: `let b = a+1; let c = a+2; let d = b+c` → b,c
   independent; d depends on both
5. Fold acc-independence: body's x*x doesn't reach acc
6. Fold acc-dependence: body's acc+x reaches acc
7. Map elements independent
8. Cross-function independence

**Ownership rendering tests (Phase 1):**

9. Read-only function → zero clones, all params `&T`
10. Record with Copy fields → zero clones
11. Record with non-Copy field → clone at construct site
12. Multiple reads → zero clones regardless of fan-out
13. Generated lens clone count pinned (~6 at Phase 1, ~1
    at Phase 2)

**Safety / negative tests (CRITICAL):**

14. Every generated artifact compiles with rustc
15. Generated lens matches handwritten oracle (roundtrip)
16. Clone-everything vs ownership-model produces same output
17. Nested construct depth handled correctly
18. Intentionally wrong ownership → rustc rejects or parity
    fails

---

## §11. When this doc updates

- Phase 1 lands → clone count pinned, §3.5 approach chosen
- Phase 2 lands → transitive ownership resolved, fold
  accumulator fixed
- All phases → doc archives
