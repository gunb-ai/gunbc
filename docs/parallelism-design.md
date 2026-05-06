# Parallelism + Concurrency — Design

> **Parent docs:** `THESIS.md` (§"parallelism is not a feature"),
> [`INVARIANTS.md#p4-decidability`](../INVARIANTS.md#p4-decidability) (bounded iteration),
> `src/v3/SELF_HOSTING.md` §14.7 (ownership track),
> `docs/ownership-rendering-design.md` (rendering model).
>
> **Purpose:** design how the compiler detects parallelism
> opportunities and produces correct concurrent code with
> appropriate threading and synchronization for each target
> platform.
>
> **Key thesis claim:** "parallelism is not a feature, it is the
> default." In a pure, immutable, decidable language, independent
> operations are ALWAYS safe to run in parallel. The compiler
> reports this as a structural fact. MapReduce, fork-join,
> pipeline parallelism — these are not features to add. They
> are consequences of the dependency structure that the compiler
> already knows.
>
> **Status:** design phase.

---

## §1. The primitive: dependency analysis

The DAG already encodes every data dependency via `produced_by`
edges. The compiler doesn't need a "parallelism analysis." It
needs ONE thing: **for each port, what is its transitive
dependency set?**

Two ports with non-overlapping dependency sets are independent.
Independent operations are safe to run in parallel because:
1. All values are immutable (no data races)
2. All functions are pure (no side effects in .dag code)
3. The DAG explicitly encodes all dependencies

Everything else — parallelism detection, fold decomposition,
MapReduce, critical path analysis — falls out as a READING
of this single fact.

### §1.1 The dependency lens

A lens that reads the DAG and computes per-port dependency sets:

```dag
type DependencySet {
  port: PortId
  depends_on: List<PortId>  // transitive closure of produced_by
}
```

The computation is the same walk `unused_parameters.dag` does
in `referenced_ports` — follow `produced_by` edges backward
transitively. The dependency lens generalizes this into a
reusable first-class fact.

### §1.2 Independence as a derived fact

```dag
fn are_independent(dag: Dag, a: PortId, b: PortId) -> Bool =
  not(any_overlap(depends_on(dag, a), depends_on(dag, b)))
```

Two operations are independent if their dependency sets don't
overlap. This is the complete parallelism safety proof for pure
code. No additional analysis needed.

### §1.3 What falls out of independence

**Independent let bindings:**
```dag
let a: Int = expensive_computation_1(x)
let b: Int = expensive_computation_2(y)
// a and b share no dependency → parallelizable
```

**Map elements:**
```dag
map(items, |x| f(x))
// each f(x) depends only on x, not on other elements → parallelizable
```

**Branch path setup:**
```dag
if condition then expensive_a else expensive_b
// at runtime only one path executes, but if the target supports
// speculative execution, both paths are independent
```

**Pipeline stages:**
```dag
let parsed = parse(source)
let lowered = lower(parsed)
// sequential — lowered depends on parsed
// but parse of FILE 2 is independent of lower of FILE 1
// → pipeline parallelism between files
```

---

## §2. Fold decomposition

A fold's body lambda has two parameters: the accumulator and
the current element. The dependency lens reveals which parts
of the body depend on the accumulator:

```dag
fold(items, init, |acc, x| body(acc, x))
```

The body's sub-DAG partitions into:
- **acc-independent nodes:** don't transitively depend on `acc`
- **acc-dependent nodes:** do transitively depend on `acc`

```dag
type FoldStructure {
  fold_node: NodeId
  acc_param: PortId
  element_param: PortId
  acc_independent: List<NodeId>  // the "map" part
  acc_dependent: List<NodeId>    // the "reduce" part
}
```

### §2.1 Pure map (acc unused)

```dag
fold(items, [], |acc, x| cons(f(x), acc))
// f(x) is acc-independent → map
// cons(result, acc) is the only acc-dependent node → collect
// Equivalent to: map(items, |x| f(x))
```

The fold IS a map. The compiler reports this as a structural
fact. The emitter can render parallel execution (e.g.,
`rayon::par_iter().map(f).collect()` in Rust).

### §2.2 Map + reduce (partial independence)

```dag
fold(items, 0, |acc, x| acc + (x * x))
// x * x is acc-independent → map
// acc + result is acc-dependent → reduce
// Equivalent to: map(items, |x| x * x) then fold(mapped, 0, +)
```

The per-element work (`x * x`) is independent. The
accumulation (`acc + result`) is sequential. The compiler
decomposes this into a parallel map followed by a sequential
reduce. If the reduce operation is also associative
(CommutativeMonoid), the reduce itself can be parallelized
via tree reduction (L2 M1 algebra awareness).

### §2.3 True sequential (acc used throughout)

```dag
fold(items, initial_state, |acc, x| update(acc, x))
// update reads acc everywhere → genuinely sequential
// no map decomposition possible
```

The compiler reports: this fold is sequentially dependent.
If `update` inhabits an associative algebra, tree reduction
is still possible (L2 algebra work). Otherwise, it's
inherently sequential.

---

## §3. Effects boundary

Pure .dag code is always safe to parallelize. But
`ExternalRealization` operations may have side effects. The
effects lens (L2 M3) classifies operations:

- **Pure:** no effects → always parallelizable
- **Read-only:** reads external state → parallelizable with
  each other (no conflicts)
- **Write:** modifies external state → needs synchronization
  with other writes to the same resource

The parallelism strategy composes dependency independence
WITH effect safety:

```
independent + both pure          → parallel, no sync needed
independent + both read-only     → parallel, no sync needed  
independent + one writes         → needs sync (lock, channel, ordering)
dependent                        → sequential regardless of effects
```

For L1.5/L2, we only handle the pure case (which covers all
.dag-native code). Effect-aware parallelism is L2 M3+.

---

## §4. Target rendering strategy

How the target language expresses parallelism is declared in
the realization spec, not hardcoded in the emitter:

```dag
type ParallelismStrategy {
  parallel_map: String        // e.g. "rayon::par_iter().map"
  parallel_reduce: String     // e.g. "rayon::par_iter().reduce"
  spawn: String               // e.g. "std::thread::spawn"
  join: String                // e.g. ".join().unwrap()"
  sync_primitive: String      // e.g. "Arc<Mutex<{T}>>"
}
```

Target declarations:

```dag
// rust.dag
data rust_parallelism: ParallelismStrategy = {
  parallel_map: "par_iter().map({F}).collect()",
  parallel_reduce: "par_iter().reduce(|| {INIT}, {F})",
  spawn: "std::thread::spawn(move || {BODY})",
  join: ".join().unwrap()",
  sync_primitive: "Arc<Mutex<{T}>>"
}

// go.dag
data go_parallelism: ParallelismStrategy = {
  parallel_map: "// goroutine pool over {LIST}",
  ...
}
```

### §4.1 Platform capability

```dag
// extdeps/platform/
type PlatformCapability {
  core_count: Int           // 0 = unknown (conservative)
  thread_spawn_cost_ns: Int // cost of spawning a thread
  min_parallel_work_ns: Int // don't parallelize below this threshold
}
```

The emitter reads platform facts to decide whether
parallelism is worth the overhead. If the per-element work
is cheaper than the spawn cost, sequential is faster.

---

## §5. Relationship to ownership rendering

The parallelism design and ownership design share the same
upstream fact: **purity guarantees safe sharing.** The
ownership doc (`docs/ownership-rendering-design.md`) asks
"how does the target share values between consumers?" The
parallelism doc asks "how does the target share values between
threads?"

The answer is the same for both: in a pure language, sharing
is always safe. The rendering difference is:

- Ownership: borrow (`&T`) for reads, clone for construction
- Parallelism: shared reference (`Arc<T>` or `&T` with
  scoped threads) for cross-thread reads

Both read the same dependency lens. Both read the same purity
guarantee. The emitter composes both: a parallel map needs
the input shared across threads (parallelism strategy) AND
each thread needs the element by reference (ownership
strategy).

**The ownership design doc's SourceMutability type is the
same upstream fact both systems need.** If `Immutable`,
both sharing (ownership) and parallelism are safe. If
`Mutable`, neither is safe without additional analysis.

---

## §6. Phasing

| Phase | When | What |
|---|---|---|
| Phase 1 | L2 | Dependency lens + independence detection. `DependencySet` and `FoldStructure` types in `std/`. Tests: independent let bindings, map elements, fold decomposition. |
| Phase 2 | L2 M3+ | Effects-aware parallelism. Independence + effect classification → sync requirements. |
| Phase 3 | L2.5 | ParallelismStrategy + PlatformCapability types in `std/` and `extdeps/`. Realization specs declare target-specific rendering. |
| Phase 4 | L3 | Emitter produces concurrent code. Reads dependency + effects + strategy + platform. |

---

## §7. Testing approach

**Structural tests (testable NOW):**

1. Independent let bindings have no transitive dependency
2. Sequential let bindings DO have a transitive dependency
3. Map elements are independent (no cross-element dependency)
4. Fold accumulator chains iterations (sequential)
5. Fold with acc-independent body decomposes into map + reduce
6. Fold with acc-dependent body does NOT decompose

**Behavioral tests (testable at L3):**

7. Parallel map emits `rayon::par_iter().map(...)` in Rust
8. Sequential fold emits `iter().fold(...)` in Rust
9. Decomposed fold emits parallel map then sequential reduce
10. Effect-bearing operations emit synchronization primitives

**Property tests:**

11. For any two independent ports, swapping their evaluation
    order produces the same result (commutativity of independent
    operations — testable via the interpreter at Path C)

---

## §8. When this doc updates

- Dependency lens lands → §1 graduates
- Fold decomposition lands → §2 graduates
- Effects lens integrates → §3 graduates
- First parallel emission target → §4 graduates
- All phases complete → this doc archives
