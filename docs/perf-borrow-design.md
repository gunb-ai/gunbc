# PERF Root Cause: Borrow-by-Default Emission

Part of: [ROADMAP.md §PERF](../ROADMAP.md#perf-eliminate-unnecessary-work) |
[THESIS.md §Concept unification](../THESIS.md#concept-unification)

## Hypothesis

All 13,694 `.clone()` calls in stage0 trace to 3 emitter decisions.
Fixing these 3 decisions at the emission layer eliminates ~13,000
clones. The surviving ~500-1,000 are genuine construction clones
(building new values that share data). No per-function analysis.
No ownership annotations. The purity of .dag IS the proof.

## Verified data (2026-04-12)

**13,694 total `.clone()` calls** across 57 stage0 .rs files
(51,798 lines). Breakdown by actual cost:

| Category | Count | Cost per clone | Fix |
|----------|-------|---------------|-----|
| Vec/Map heap alloc (`children`, `params`, `diagnostics`, Rc<HashMap>) | ~2,400 | **Heap alloc proportional to size** | Borrow |
| String heap alloc (`name`, `.name`, `module_name`, `text`) | ~2,250 | **Heap alloc** | Borrow or ident:Int |
| Parser state (`state`, `tokens`, `err`, `s`) | ~2,700 | Rc increment | Stream D + borrow |
| Read-only env threading (`source_indices`, `type_env`, `registry`, `si`) | ~2,200 | Rc increment on large maps | Borrow |
| Rc<Node> refcount (`n`, `texpr`, `expr`, `body`) | ~1,400 | Atomic increment | Borrow |
| Copy types (`depth`, `target`, `connective`, `span`) | ~1,200 | **Free** (compiler optimizes) | Remove (cleanup) |
| Other (accumulators, loop vars) | ~1,500 | Mixed | Case by case |

**The expensive clones are Vec/Map + String = ~4,650.** Each is a
`malloc` + `memcpy`. On self-compile (113 files), this is millions
of heap operations. These are the wall-clock killers.

Top 10 clone sites (variable × count):

| Variable | Count | File | Type |
|----------|-------|------|------|
| `state.clone()` | 1,085 | parse | Rc<ParserState> |
| `name.clone()` | 751 | all | String |
| `source_indices.clone()` | 732 | emit + infer | Rc<HashMap> |
| `err.clone()` | 677 | parse | Rc<Node> |
| `tokens.clone()` | 634 | parse | Rc<Vec<Token>> |
| `children.clone()` | 502 | all | Rc<Vec<Node>> |
| `span.clone()` | 409 | all | SourceSpan (Copy!) |
| `expr.clone()` | 348 | all | Rc<Node> |
| `type_env.clone()` | 342 | infer + emit | Rc<HashMap> |
| `acc.clone()` | 335 | all | Various accumulator |

## Root cause

.dag is pure. All values are immutable. A function that receives a
value NEVER mutates it. But the Rust emitter translates every
parameter as **owned** `Rc<T>`, forcing every call site to clone.

**One .dag function call:**
```dag
fn render_node_type(n: Node, source_indices: Map<String, NewlineIndex>) -> String {
  let tn = authored_name_at(source_indices: source_indices, node: n)
```

**Current Rust emission:**
```rust
fn render_node_type(n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>) -> String {
    let tn = authored_name_at(source_indices.clone(), &n);
    //                        ^^^^^^^^^^^^^^^^^^^^^^^^
    //                        CLONES entire Rc<HashMap> to satisfy owned param
```

**Correct Rust emission:**
```rust
fn render_node_type(n: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>) -> String {
    let tn = authored_name_at(source_indices, n);
    //                        no clone — already a borrow
```

This is not an optimization. It is the **correct translation** of
.dag's pure semantics into Rust. The current emission is wrong — it
treats reads as if they might mutate.

## Why .dag purity makes this trivial

Traditional Rust needs complex borrow analysis because mutation
makes borrows unsafe. .dag has NO mutation. Therefore:

- Every parameter is read-only → emit as `&Rc<T>`
- Every field access is a read → emit as `&n.field`
- Every binding use is a read → emit as `&binding`
- The ONLY clone needed is at **construction sites** — building a
  new `Rc<Node>` that outlives the current scope and shares data

No lifetime annotations. No borrow checker fights. No per-function
analysis. The purity of .dag IS the ownership proof.

## The 3 decisions

### Decision 1: Function parameters (owned → borrowed)

**Rule:** All `fn` parameters emit as `&Rc<T>` (or `&T` for Copy
types). All call sites pass `&arg` instead of `arg.clone()`.

**Exception:** Parameters moved into a return value or stored in a
new data structure need owned `Rc<T>`. The emitter detects this:
if the parameter appears in a Node construction or Map insertion,
it needs ownership. All others are borrows.

**Estimated impact:** ~8,000 clones eliminated.

**Emitter changes:**
- `05_emit_rust.dag` — function signature emission
- `05_emit_rust.dag` — call argument emission

**Validation:** Regen stage0. Count `.clone()`. Expect ~13,694 →
~5,500. Run performance test. All tests pass.

### Decision 2: Last-use move (clone every use → move last)

**Rule:** For bindings with fan-out > 1, last use moves, prior
uses borrow. For fan-out = 1, the single use moves (zero clones).

**Note:** With Decision 1 (borrow params), most multi-use bindings
pass borrows anyway. Decision 2 matters only for params that DO
need ownership (the exception from Decision 1).

**Estimated impact:** ~3,000 clones eliminated (overlaps with D1).

**Emitter changes:**
- `05_emit_rust.dag` — variable reference emission
- Reads from `ownership.dag` last-use data (already computed)

### Decision 3: Field access borrow (clone → borrow)

**Rule:** Field access on a borrowed value produces a borrow.
`&n.name` instead of `n.name.clone()`. Exception: fields moved
into new constructions keep `.clone()`.

**Estimated impact:** ~2,000 clones eliminated.

**Emitter changes:**
- `05_emit_rust.dag` — field access emission

## Implementation plan

### Phase A: Borrow parameters (highest leverage, do first)

1. In `05_emit_rust.dag`, change parameter emission from `Rc<T>`
   to `&Rc<T>` for all non-Copy compound types.
2. Change call site emission from `arg.clone()` to `&arg`.
3. Handle exceptions: params in Node construction or Map insertion
   keep owned `Rc<T>`. Detect by scanning function body for
   construction sites that reference the param.
4. Regen stage0. Fix compile errors (lifetime issues indicate
   genuine ownership needs, not bugs).
5. Count `.clone()`, run perf test.

**Expected outcome:** 13,694 → ~5,500 clones. Perf test improves.

### Phase B: Field access borrow

1. Change field access emission from `n.field.clone()` to `&n.field`.
2. Handle exceptions: fields moved into constructions keep clone.
3. Regen, fix errors, count clones.

**Expected outcome:** ~5,500 → ~3,500 clones.

### Phase C: Last-use move

1. Wire ownership last-use data into variable reference emission.
2. Last use moves, prior uses borrow or clone.
3. Regen, count clones.

**Expected outcome:** ~3,500 → ~800 clones (construction-only).

## How to test the hypothesis

Before implementing, a quick smoke test:

1. Take ONE function (e.g., `render_node_type`) in stage0 .rs
2. Manually change its params from `Rc<T>` to `&Rc<T>`
3. Manually change its callers to pass `&arg` instead of `arg.clone()`
4. Count clones eliminated in that one function's call graph
5. Measure compile time before/after

This validates the hypothesis on a single function before the
full emitter change. If clones drop and time improves for one
function, the emitter-wide change will scale linearly.

## Relationship to other work

| Active work | Relationship |
|-------------|-------------|
| **Node.name deletion (M2)** | Subset. Eliminates ~396 String clones via Int (Copy). Still valuable for identity modeling. No longer perf-critical once borrows land. |
| **Stream D parser** | Eliminates ParserState (~1,085 clones). Architectural improvement for CX. Still valuable. |
| **Stream B clone elision** | Layer 3 (borrow propagation) IS Decision 1. Same work. Don't maintain separately. |
| **LS-4 borrow model** | Trivially solved: .dag is pure → borrow everything → clone at construction only. This doc IS the LS-4 design. |

## Success criteria

Not time targets. Clone elimination targets:

| Metric | Before | After A | After A+B+C |
|--------|--------|---------|-------------|
| `.clone()` in stage0 | 13,694 | ~5,500 | ~800 |
| Heap-alloc clones (String+Vec+Map) | ~4,650 | ~1,500 | ~300 |
| Rc refcount-only clones | ~6,300 | ~2,500 | ~400 |
| Copy-type clones (dead code) | ~1,200 | ~1,200 | 0 |
