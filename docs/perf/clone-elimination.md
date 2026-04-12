# PERF: Clone Elimination — Current State and Remaining Work

Part of: [ROADMAP.md §PERF](../ROADMAP.md#perf-eliminate-unnecessary-work) |
[THESIS.md §Concept unification](../THESIS.md#concept-unification)

**Revision note (2026-04-12):** An earlier version of this doc had
stale numbers and an incorrect cost model. This rewrite corrects both
and credits the selective borrowing work already landed in the emitter.

**Scope:** This doc covers **Rust-target clone elimination** in the
self-compile pipeline. It is NOT the full LS-4 borrow model design —
LS-4 is language-spec work (`SharingStrategy` in `languages.dag:174`)
that spans Rust/Go/Python borrow semantics per target. Python/Go use
GC, so Rc-style clone elimination doesn't apply to them. This doc is
about making the Rust target's emitted code faster, not about
defining LS-4 semantics.

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

### Single-use move via `movable` (NOT general last-use)

`ownership.dag:402` + `05_emit_rust.dag:1876` (`emit_var_ref`):
- Ownership proof computes `movable` for bindings where
  **fan_out == 1** (single-use owned locals)
- Movable bindings emit as owned at that single use site
- This is NOT general last-use analysis for fan_out > 1

**Important correction:** the current `movable` set is narrower
than "last use moves." For fan_out > 1 bindings, every use still
clones — there's no "last use moves, earlier uses clone" analysis
yet. And even single-use has correctness exceptions: match-bound
names must still clone (`pipeline.rs:1323`), TCO functions
disable borrowing entirely.

So "wire existing data" for last-use understates the gap. Building
general fan_out > 1 last-use analysis is genuine work, not a
cleanup task.

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

### Exclusion 3: Field access clones (not a simple rewrite)

Field access already has nontrivial special cases in the emitter.
`05_emit_rust.dag:1954` and `1988` only suppress the outer clone
on the BASE expression — the field VALUE is still cloned or moved
based on access style (tuple/option/enum) and ownership state.
`05_emit_rust.dag:2022` and `2728` handle anonymous-record
flattening and `owned_bindings` after `Rc::try_unwrap`.

**A blanket `&n.field` rewrite would change expression types
across the backend, not just delete clones.** This is NOT a
systematic find/replace. Each special case needs its own analysis.

**Question for the session:** Can we enumerate which field-access
patterns are legitimately clone-vs-borrow choices vs which are
unnecessary clones? The investigation must be pattern-specific.

### Exclusion 4: Parser state threading (landed via Stream D)

~~The parser threads `state: Rc<ParserState>` through every function.~~
**RESOLVED:** Stream D (PR #419) merged 2026-04-12 and rewrote
02_parse.dag to consume token lists directly. `ParserState` is
eliminated from the parser path.

**Caveat:** Stream D's CX ratchet impact was -3 (353 → 350),
far short of the expected -137. The parser restructured
mechanically, but per-field provenance consumption may not be
firing. This is a SEPARATE investigation — see "Known gap:
Stream D vs CX expectations" below.

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

## Investigation plan

This section is the concrete scope for the perf session. Each
step produces a decision, not just data. Reviewers: please
challenge the methodology, the hypotheses, and the decision
points.

### Step 1: Measure the baseline (no changes yet)

**Command:**
```bash
cargo build -p v2-compiler --release
time ./target/release/v2-compiler compile \
  --source-root src/v2 --source-root dsl \
  --output-dir /tmp/perf-baseline
```

**Capture:**
- Wall-clock time (three runs, median)
- Stage-by-stage time breakdown (requires instrumentation —
  see Step 2 if missing)
- `cargo flamegraph` output (one run, release build)

**Decision:** What's the distribution of wall-clock time across
stages (tokenize → parse → resolve → infer → CX → ownership →
emit)? If one stage is >50% of time, that's the bottleneck
regardless of clone count.

### Step 2: Add per-stage instrumentation if missing

The `performance_ratchet` test already runs the full pipeline
but doesn't break down by stage. Add `std::time::Instant` at
each stage boundary in `compile.dag`'s `compile_sources` and
print elapsed per stage. Commit this as a separate PR — it's
useful infrastructure beyond this investigation.

**File:** `src/v2/compile.dag` around the pipeline sequence in
`compile_sources`.

**Emits:** stderr lines like `[perf] tokenize: 245ms` per stage.

### Step 3: Profile the dominant stage

Use `cargo flamegraph` to find the hot function(s) inside the
dominant stage. Expected suspects:

| Hot function candidate | Why it might be hot | Fix direction |
|-----------------------|---------------------|---------------|
| `classify_descent_evidence` in complexity.dag | Heuristic tables scanned per-function | Wire proof construction (M1 Step 3) |
| `resolve_node_bounded` in 04_resolve.dag | Type resolution walks each Node deeply | Caching / single-pass |
| String allocation in parser | Every token creates strings | M2 Node.name deletion (ident:Int) |
| `map_insert` / `map_get` on String keys | Hash + clone per lookup | M2 ident:Int registry keys |
| Nested `.clone()` in emission | Field access clones (exclusion 3) | Systematic field borrow |

**Decision:** The ONE function or pattern that's >30% of stage
time is the target. If no single hot function dominates, the
time is diffuse and the fix is architectural.

### Step 4: Validate hypothesis on a single function

**Smoke test pattern:**
1. Pick the hottest function from Step 3
2. Identify its dominant cost (String clone? Registry lookup?
   Recursive pattern?)
3. Apply a targeted fix to THAT function only (manual, not via
   emitter change)
4. Regen stage0, measure again
5. Compare: did the hot function's time drop? Did total time drop?

**Decision:**
- If the targeted fix drops both function time AND total time
  proportionally → scale up the fix to the emitter
- If function time dropped but total didn't → Amdahl's law says
  the hot spot was only 20% of total, move to the next suspect
- If nothing moved → the fix didn't do what we thought, profile
  again

### Step 5: Scale the fix

Only after Step 4 validates a hypothesis, apply the fix
systematically. This is where M2 Node.name deletion fits if
the profile confirms String allocation is the bottleneck.

## Known unknowns

These are questions the investigation must answer, not
assumptions it can make:

1. **Is the pipeline single-threaded end-to-end?** If yes, Rc
   refcount clones are near-free (no atomic contention). If
   there's any parallelism, refcount clones matter more.

2. **Is the performance ratchet hitting 120s because of CI
   runner variance or real regression?** Compare local
   self-compile time to CI time. If local is <30s and CI is
   120s, the issue is CI runner, not code.

3. **Does Stream D's parser rewrite make things FASTER or
   SLOWER?** The parser grew from ~3000 clones to similar
   count but different distribution. Measure before/after
   Stream D explicitly.

4. **Are the generated tests the hot path?** `full_dsl_compiles`
   runs 113 files through the full pipeline. If the test
   harness itself is slow (test discovery, compilation of test
   crate), fixing the compiler won't help CI time.

## Non-goals for this investigation

To keep scope tight:

- **No emitter changes without profile evidence.** If someone
  proposes "let's borrow X," they must show the profile data
  first.
- **No hypothetical optimizations.** Only fix what measurement
  shows is slow.
- **No ratchet bumps.** The ratchet at 120s is already a
  symptom-treatment red flag. Fixing should LOWER it, not
  raise it further.
- **No parallelism work yet.** Parallelizing a pipeline with
  quadratic inner loops just makes the quadratic parallel.
  Fix the inner loops first.

## Known gap: Stream D vs CX expectations

Stream D (PR #419) merged 2026-04-12 with the expectation
of -137 CX violations. Actual impact: -3 (353 → 350).

**This is a separate investigation, not part of the perf work,
but it's related:** understanding why Stream D didn't deliver
expected CX wins may reveal whether per-field provenance
consumption is wired correctly. If it's not, the clone
elimination hypothesis for parser struct returns may also be
broken — the parser passes shrinking token lists but CX
doesn't see the sub-value relationship, AND the emitter
doesn't apply borrows because the analysis is wrong.

Recommend a parallel investigation into Stream D CX wiring
separate from this perf investigation.

## Review questions

Reviewers, please specifically address:

1. **Is the cost model accurate?** I claim `Rc<T>.clone()` is
   cheap (atomic refcount++) and `String::clone()` is expensive
   (malloc + memcpy). Is there a case I'm missing?

2. **Are the exclusions (callables, TCO) correctly identified?**
   I found them by reading `05_emit_rust.dag:471-493`. Have
   I missed other exclusion categories?

3. **Is "profile first" the right methodology?** Would you
   recommend a different approach given the state of the
   codebase?

4. **What's the right metric?** Clone count? Heap allocations?
   Wall-clock per stage? Something else?

5. **Should we parallelize the pipeline before optimizing
   single-threaded?** 113 files × 8 stages is embarrassingly
   parallel, but doing parallelism on top of bad single-thread
   code multiplies the badness.
