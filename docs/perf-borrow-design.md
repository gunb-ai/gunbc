# PERF: Clone Elimination — Current State and Remaining Work

Part of: [ROADMAP.md §PERF](../ROADMAP.md#perf-eliminate-unnecessary-work) |
[THESIS.md §Concept unification](../THESIS.md#concept-unification)

**Revision note (2026-04-12):** An earlier version of this doc had
stale numbers and an incorrect cost model. This rewrite corrects both
and credits the selective borrowing work already landed in the emitter.

## Current state (verified 2026-04-12)

**Stage0:**
- 59 `.rs` files (including main.rs and lib.rs)
- 54,568 total lines
- 21,211 `.clone()` sites

**Per-file hotspots:**

| File | `.clone()` lines |
|------|-----------------|
| v2_compiler_parse.rs | 3,587 |
| v2_compiler_infer.rs | 2,285 |
| v2_compiler_emit_rust.rs | 1,831 |
| v2_compiler_complexity.rs | 1,680 |
| v2_compiler_emit.rs | 868 |
| v2_compiler_infer_resolve.rs | 656 |

## What's already landed

The emitter already implements selective borrowing. The machinery
exists and is wired through the pipeline:

### Per-function read-only param analysis

`05_emit_rust.dag:471` (`build_ownership_results`):
1. Runs ownership analysis per function
2. Computes `read_only_params_index: Map<String, Map<String, Bool>>`
3. Stores per function-qualified name → param name → is-read-only

`05_emit_rust.dag:1364` (parameter emission):
- Consults `read_only_params_index` per function
- Read-only params emit as `&Rc<T>`
- Owned params emit as `Rc<T>`

`05_emit_rust.dag:2458` (call site emission):
- Consults the callee's `read_only_params_index`
- Read-only args emit as `&arg`
- Owned args emit as `arg` or `arg.clone()`

### Last-use move via `movable`

`ownership.dag:402` + `05_emit_rust.dag:1781`:
- Ownership analysis computes which uses are terminal (last use)
- Movable bindings emit as owned at the last use site
- Earlier uses of the same binding clone

### Verified on a real function

`render_node_type` (stage0 `v2_compiler_emit.rs:885`):
```rust
pub fn render_node_type(
    n: &Rc<Node>,
    target: &RenderTarget,
    shared_types: &Rc<HashMap<String, bool>>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>
) -> String
```
All four params are borrowed. This function is NOT a target — it's
already emitted correctly.

## What's NOT borrowed (the remaining clones)

The selective mechanism EXCLUDES certain functions from borrowing.
These exclusions are where the remaining clones live:

### Exclusion 1: Higher-order callable references

`05_emit_rust.dag:472-493`:
```
let callable_set = modules |> fold(..., collect_callable_refs(...))
...
let read_only = if map_contains_key(callable_set, entry.proof.func_name) {
  empty_map()  // excluded — no borrowing
} else {
  build_read_only_params(...)
}
```

Functions passed as first-class callables (to HOFs like `map`,
`filter`, `fold`) cannot have their parameters borrowed — that would
change the callable's signature. So the entire function is emitted
with owned parameters. Every call site clones.

**Question for the session:** How many functions are in the callable
set? If a function is in `callable_set` AND also called directly
(not just passed to HOFs), its direct call sites miss borrow benefits.
Could callables emit two versions — borrowed for direct calls, owned
wrapper for HOF passing?

### Exclusion 2: TCO functions

`v2_compiler_emit_rust.rs:698` disables read-only borrowing for
TCO-eligible functions. TCO rewriting reassigns parameters in a
loop, which requires owned parameters. Every param in a TCO
function is owned, every call site clones.

**Question for the session:** Is there a way to emit TCO with
borrowed params? Or: emit a thin owned-param wrapper around a
borrowed-param implementation so call sites benefit?

### Exclusion 3: Field access clones

`v2_compiler_emit_rust.rs:1900, 1915` and similar: field access
sometimes emits as `value.field.clone()` instead of `&value.field`.
Pattern-specific rather than systematic.

**Question for the session:** Which field access patterns still
clone? Is there a systematic rewrite?

### Exclusion 4: Parser state threading

The parser threads `state: Rc<ParserState>` through every function,
generating 1,085 `state.clone()` calls and 889 `.state.clone()`
field accesses.

Stream D (PR #419, dark-ant) is rewriting the parser to consume
token lists directly, eliminating `ParserState` entirely. This is
the architectural fix.

## Clone cost model (corrected)

The previous draft treated all clones as expensive heap allocations.
That's wrong. Most clones are `Rc::clone` — an atomic refcount
increment, not a heap copy:

| Type | Cost of `.clone()` | Example |
|------|-------------------|---------|
| `Rc<T>` | Atomic refcount++ | `n.clone()`, `children.clone()`, `source_indices.clone()` |
| `Rc<SourceSpan>` | Atomic refcount++ | `span.clone()` |
| `String` | **Heap allocation** (malloc + memcpy) | `name.clone()` when `name` is a `String` field |
| Copy types (`i64`, `bool`, fieldless enum variants) | Free (compiled to move) | `depth.clone()`, `target.clone()`, `ident.clone()` |

**Node field types (verified from `v2_std_core.rs:513`):**
```rust
pub struct Node {
    pub name: String,              // heap clone — M2 deletion target
    pub ident: i64,                // Copy, free
    pub span: Rc<SourceSpan>,      // refcount bump
    pub ident_span: Option<Rc<SourceSpan>>,  // refcount bump
    pub children: Rc<Vec<Rc<Node>>>,  // refcount bump
    pub connective: Connective,    // Copy if simple variants
    pub params: Rc<Vec<Rc<Node>>>, // refcount bump
    pub inferred: Option<Rc<InferredNode>>,  // refcount bump
    // ... all other fields are Rc<T> or Option<Rc<T>>
}
```

**`SourceSpan` is NOT Copy** (`std_types.rs:218`) — it's a struct
intentionally wrapped in `Rc`. So `span.clone()` is a refcount
bump, not a copy.

**The real heap-allocation sources:**
1. **String clones** — `name.clone()`, `module_name.clone()`,
   `text.clone()`, etc.
2. **Construction clones** — building new `Rc<Node>` where field
   values are cloned into the new node (genuine work, not waste).

**Refcount clones aren't free, but they're not the wall-clock
killer.** They add cache pressure and contention in multithreaded
code. On single-threaded self-compile they're fast. Eliminating
them is cleanup, not the main perf win.

## Honest priority list

### High impact (heap allocation elimination)

1. **Resume M2 Node.name deletion** — `name.clone()` is 757+ direct
   occurrences plus 431 field-access occurrences. Each is a
   `String::clone` (malloc + memcpy). Replacing with `ident: i64`
   makes them Copy (free). This is the #1 unresolved heap
   allocation source. M2 Track 3 was paused; resuming is the
   highest-impact perf win.

2. **String clones in emitter** — `module_name`, `text`, `label`
   throughout `emit_rust.rs`. Some are construction-necessary;
   others could borrow. Audit needed.

### Medium impact (refcount cleanup + cache)

3. **Reduce callable-set exclusions** — if a function is in the
   callable set AND called directly, its direct call sites miss
   borrow benefits. Dual-emission (borrowed for direct, owned
   wrapper for HOF) could recover these.

4. **TCO function borrowing** — investigate whether TCO can emit
   with borrowed params via a wrapper pattern.

5. **Stream D parser rewrite (PR #419)** — eliminates `ParserState`
   entirely. ~2,000 state-related clones dissolve. Architectural fix.

### Low impact (cleanup only)

6. **Copy-type clones** (`depth`, `target`, `connective`, etc.) —
   compile to moves. Removing them is code cleanup, no perf gain.

7. **Systematic field-access borrowing** — pattern-specific audit
   of sites that clone where borrow would work.

## What this doc gets right vs wrong

**Right (general principle):**
- .dag is pure; all params are read-only; borrow is the correct
  translation where possible
- Clone count is a structural metric, not a time target
- Construction clones are genuine work, not waste

**Previous draft got wrong (corrected here):**
- Numbers were stale (13,694 → actual 21,211; 51,798 lines → 54,568)
- Cost model conflated `Rc::clone` (refcount++) with heap copy
- Claimed borrow-by-default was missing — it's already landed for
  most functions via `read_only_params_index`
- Used `render_node_type` as a "needs fixing" example when it's
  already correctly emitted with borrows
- Over-promised clone reduction by treating all clones as equivalent

## How to test perf hypotheses

The clone count alone doesn't tell us where TIME goes. To measure
real wall-clock impact:

1. **Profile self-compile** with `cargo flamegraph` or `perf record`
2. **Count heap allocations per stage** (not just clones)
3. **Measure `String::clone` hot spots** specifically — that's
   where malloc+memcpy happens
4. **Before/after comparison** for any targeted change

The smoke test for a specific hypothesis:
1. Pick ONE category (e.g., String clones in emitter)
2. Manually apply the fix to ~10 sites
3. Regen stage0
4. Measure self-compile time
5. If time moves, automate the pattern in the emitter

Without profile data, targeting specific clones is guessing.
Profile first, then fix the hot spots.

## Recommendation

**For the perf session:**

1. **Profile first.** Run `cargo flamegraph` on self-compile. Find
   what actually dominates wall-clock time. The answer may or may
   not be clones.

2. **If clones dominate, target String heap allocations first.**
   Resuming M2 Node.name deletion is the highest-leverage fix.

3. **If something else dominates (parsing, type inference, emission),
   investigate that instead.** The clone count is a hypothesis, not
   a conclusion.

4. **Don't chase refcount clones until profile data proves they
   matter.** They might, but they might not. Guessing wrong wastes
   the session's time.
