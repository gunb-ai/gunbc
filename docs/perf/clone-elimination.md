# PERF: Eliminating Unnecessary Work (Clones are the Symptom, Not the Disease)

Part of: [ROADMAP.md §PERF](../../ROADMAP.md#perf-eliminate-unnecessary-work) |
[THESIS.md §Concept unification](../../THESIS.md#concept-unification) |
[INVARIANTS.md](../../INVARIANTS.md)

**Revision 3 (2026-04-12, post-dark-emu):** The previous revisions
focused on `.clone()` elimination as the perf target. A real
profiling session proved that framing was wrong. Clones are NOT
the perf crisis. **Fact re-derivation is.** A 6-line fix to
`merge_envs` produced more speedup than any clone elimination
work. This revision captures what actually happened and what
it teaches us.

## The canonical example: merge_envs

### What the bug was

The compiler threads a single `InternTable` through every
`TypeEnv` that flows through inference. `merge_envs` takes
multiple envs and produces a merged one. Its old code:

```
let merged_intern_table = merge_intern_tables(
  envs |> map(e => e.intern_table)
)
```

This iterated every string in every env's intern table and
re-interned them into a fresh table. Per call: O(envs × strings).
Called twice per module (`build_type_env` + `build_type_env_unresolved`).

**For 143 .dag files, this was ~20 seconds of pure waste.**

### Why it was waste

After PR2 (wiring InternTable to TypeEnv), every env in the
pipeline shared the **same** intern table — they all pointed
to one upstream authority. Merging N copies of the same table
is "take the first one." But `merge_envs` didn't know about
the upstream change, so it kept doing the expensive merge.

### The fix

6 lines:
```
let merged_intern_table = match envs |> first {
  Some { value: first_env } => first_env.intern_table
  None => empty_intern_table()
}
```

### The measurement

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| Reconcile stage | 9.54s | 140ms | **68×** |
| Per-module reconcile | ~1.1s | ~5ms | **200×** |
| Total pipeline (gist test) | 11.72s | 2.34s | **5×** |
| Self-compile (all 143 files) | ~60-75s | 37.6s | **~2×** |
| Reconcile % of total | 81% | 6% | — |

**Note what's NOT in this table:** `.clone()` count is unchanged.
This speedup had nothing to do with clones.

## What this taught us

### Lesson 1: Clones are the visible symptom, not the disease

We counted clones because they're grep-able. 21,211 of them!
Huge number! Must be the problem!

But most clones are `Rc::clone` — an atomic refcount increment.
Individually fast. The 21,211 number was a red herring. The
ACTUAL cost was in one function doing O(n²) work on data it
could have read in O(1).

**The rule:** if you can count it with grep, it's probably not
the bottleneck. Real bottlenecks hide in algorithmic behavior,
not in syntactic patterns.

### Lesson 2: Profile before you plan

The first two versions of this doc were **detailed investigation
plans for the wrong target.** They proposed 3-phase clone
elimination with specific emitter changes. None of that would
have mattered, because clones weren't the problem.

The dark-emu session succeeded because it followed "profile
first" even when the plan pointed elsewhere. The existing
profile infrastructure (`profile_full_pipeline` in
`compiler_tests_rust.dag`) showed `reconcile: 81%` immediately.
That pointed at `merge_envs`. The 6-line fix followed.

**The rule:** before writing any perf design doc, run the
profiler. Not to gather data for the plan. To INVALIDATE
hypotheses before committing to them.

### Lesson 3: "Construct-discard-reconstruct" is the root pattern

The thesis and INVARIANTS.md:104 already named this pattern:

> When the compiler compensates conservatively, the real fix is
> usually to stop losing a fact, not to optimize the compensation.

`merge_envs` is this pattern exactly:
- **Upstream:** single intern table (fact)
- **Threaded through TypeEnv:** preserved
- **`merge_envs`:** discarded the fact, reconstructed from parts

Before PR2, the compensation was necessary — envs had their
own intern tables. After PR2, the compensation became pure
waste. **Nobody deleted it** because the compensation still
compiled and produced correct output.

**The rule:** when you thread a single authoritative value
through the pipeline, immediately delete any downstream code
that used to reconstruct it locally. If you don't delete it,
it becomes the next perf bug.

### Lesson 4: This is KF-2 we're committing against ourselves

KF-2 in the roadmap is "reject suboptimal algorithms" — the
compiler refuses to compile code when a provably cheaper
equivalent exists. Three right turns equals one left turn.
Redundant work is a compile error.

**`merge_envs` is a KF-2 violation.** It computes `merge(a, a, a)`
when the algebra says `merge(a, a, a) = a` (idempotency of
merge on identical inputs). The compiler should reject this
code. The only reason it doesn't is we haven't built KF-2 yet.

**We keep committing this same bug** because we're writing a
compiler that should catch it, but the compiler isn't complete.
Every perf regression that turns out to be "we did the same
work twice" is evidence that KF-2 would have saved us. The
perf crisis is partially a thesis crisis — we're living in a
half-built causal engine that doesn't yet enforce the rules
the rest of the codebase documents.

## The meta-principle: no redundant/duplicate work

This connects to the algebraic simplification concept
unification in [THESIS.md](../../THESIS.md#algebraic-simplification-idempotency-cancellation-redundancy):

> Idempotency (`f ∘ f = f`), cancellation (`f ∘ f⁻¹ = id`), and
> redundant work (`f₁ ∘ ... ∘ fₙ = g` where `cost(g) < cost(...)`)
> are all instances of **algebraic simplification**.

`merge_envs` was algebraic simplification failing:
- `merge(x, x, x) = x` (idempotency of merge on identical inputs)
- The compiler had the algebra (meet is idempotent)
- But no mechanism enforced the simplification
- So the runtime did the wasted work

**The deeper fix is not just "don't write redundant code."
It's building KF-2 so the compiler catches redundant work
structurally.** Every perf bug we hit that's "we recomputed X"
is advance payment on KF-2's value. If KF-2 existed, those
bugs would be compile errors, not latent hot spots.

## Revised investigation methodology

### Rule 1: Profile first

Before any perf work:
1. Run `profile_full_pipeline` or equivalent
2. Identify the dominant stage (>30% of time)
3. Identify the dominant function within that stage
4. Investigate THAT, not what you guessed was the problem

If you find yourself writing a "plan" before you have profile
data, STOP. Get the data first. The plan is downstream.

### Rule 2: Audit for re-derivation, not just clones

Look at every function with a name like:
- `merge_*`
- `combine_*`
- `collect_*`
- `unify_*`
- `reconcile_*`
- `build_*_from_*`

For each: **ask whether the thing being built is already
computed upstream and passed in.** If yes, the function is
construct-discard-reconstruct waste.

Secondary audit: any code that iterates over inputs and
rebuilds an aggregate ("collect all types from modules,"
"gather all names from env"). Check if the aggregate already
exists at a single authority point.

### Rule 3: When you thread a fact forward, DELETE the old reconstruction

PR2 threaded a single InternTable through TypeEnv. It did
NOT delete `merge_envs`'s reconstruction code. That was the
bug.

**Rule: every PR that adds "thread X through the pipeline"
must include "delete downstream reconstruction of X" in its
scope.** The threading is pointless if downstream still
rebuilds.

The check: after threading X, grep for "recompute X,"
"rebuild X," "merge X from parts." Delete every match.

### Rule 4: Watch for fact-flow violations in reviews

When reviewing a PR that adds a function taking N structured
inputs and producing one output, ask: **"could the output
be read from any single input instead of computed from all
of them?"**

If yes, the function is a re-derivation hazard. Either delete
it or document why the inputs genuinely differ.

### Rule 5: This is KF-2 territory — don't just fix, document

Every merge_envs-class bug we find is evidence for KF-2. Log
it as a case study. When KF-2 eventually lands (catalog of
cheaper equivalents in `std/optimization.dag`), these logged
cases become the test suite. The compiler must reject code
the test suite contains.

## Remaining clone work

Everything the previous draft said about clone elimination is
still TRUE — it's just not urgent anymore. The session got
the urgent perf win from fact-flow, not clones. Clone
elimination remains valuable for:

- **Node.name deletion (M2)** — String clones are real heap
  allocations. Still worth eliminating. But no longer on the
  critical path for perf.
- **Stable binding identity** (unblocks last-use clone elision,
  Stream B Layer 1) — worth doing for modeling correctness.
- **The exclusion categories** (callables, TCO, match-bound,
  owned-after-unwrap) — each is a downstream consumer of an
  upstream fact. The fix is NOT a new umbrella concept; it's
  threading the existing authorities through the call-site
  boundary. See "Cross-pass composition" below.

But all of this is **planned work with known structure.** It's
not the "urgent perf crisis" the previous revisions framed it
as. The urgent crisis was one bug.

## The uncomfortable question

**How many more merge_envs-class bugs are hiding?**

We don't know. The dark-emu session found this one because
profiling pointed straight at reconcile. Other fact-flow
violations may be hidden in stages that are currently under
30% of time — quietly inefficient, not dominant enough to
trigger investigation.

The sustainable answer is KF-2 + better fact-flow discipline
during reviews. The tactical answer is:

1. Profile every release
2. Audit every `merge_*` / `combine_*` / `build_*_from_*` function
3. Grep for rebuilds of threaded facts
4. When you find one, check if it's a KF-2 case study

## Current state (post-dark-emu)

| Metric | Value | Note |
|--------|-------|------|
| Self-compile | 37.6s | Was 60-75s. -50% |
| Perf ratchet | 55s | Was 120s, was 75s |
| CX violations | 353 | Unchanged |
| `.clone()` sites | 13,724 | Unchanged — clones weren't the problem |
| merge_envs bug | Fixed | 68× speedup on reconcile |
| Skip data-only analysis | Landed | 82 of 143 files skip CX/ownership |
| authored_name_at fallback | Mostly eliminated | ~8 parser sites remain |
| InternTable threading | Landed | Unblocks registry migration |

## What's next

**Not:** another clone elimination plan. That framing was wrong.

**Is:**

1. **Audit for other re-derivation hotspots.** Apply Rules 2
   and 3 above. Find the next merge_envs.
2. **M2 Node.name deletion continues** — for modeling, not
   perf. String clones are worth eliminating but no longer
   critical.
3. **M1 Step 3 — thread existing authorities, don't invent
   new ones.** `SubValueRelation` (std/induction.dag),
   `TerminationProof` (std/termination.dag), and
   `read_only_params_index` (05_emit_rust.dag:471) already
   exist. The gap is that helper function outputs don't carry
   `SubValueRelation` across the call-site boundary. Threading
   that is the path. See "Cross-pass composition" below.
4. **KF-2 planning** — we're committing this bug against
   ourselves repeatedly. Building KF-2 means the compiler
   catches the NEXT merge_envs before it ships.

## Cross-pass composition: the existing authorities chain

A recurring pattern: multiple emitter decisions (TCO owned
params, callable-set exclusion, match-bound cloning,
owned-after-unwrap) look like independent side-tables that
need unifying. A naive reading says "invent a transition
relation type that replaces all of them."

**That's wrong.** The existing authorities already compose.
The gap is not an umbrella concept. The gap is that
`SubValueRelation` evidence doesn't flow across the call-site
boundary.

The existing chain:

| Authority | Lives in | What it models |
|-----------|----------|----------------|
| `SubValueRelation` | `std/induction.dag` | Is this argument a structural sub-value of the input? |
| `TerminationProof` | `std/termination.dag` | Lexicographic composition of per-param descent evidence |
| `DescentEvidence` | complexity.dag consumers | Per-call-site descent facts |
| `read_only_params_index` | 05_emit_rust.dag:471 | Per-function read-only param set from ownership analysis |
| `movable` set | ownership.dag:402 | Single-use owned locals (fan_out == 1) |
| `callable_set` | 05_emit_rust.dag:472 | Functions used as first-class values |

Each authority lives at its correct level (std/, complexity,
ownership, emission). Each is the single source of truth for
its fact. They're NOT duplicating each other — they're
different facets that need to compose.

**The real gap:** helper function OUTPUTS don't carry
`SubValueRelation` across the boundary. When
`parse_expr(tokens)` returns a `ParseResult` containing a
sub-list of tokens, the caller's downstream use of
`result.tokens` has no way to see that it's still a
`SubValueRelation::StrictSubValue(tokens)`. CX has to
reclassify per-argument in isolation.

**M1 Step 3 is threading this fact through the call-site
boundary.** The plan already exists: output provenance on
function signatures + per-field provenance consumption.
When it lands, the downstream exclusions compose
mechanically because each consumer reads the relevant
facet from the flowing evidence:

- TCO owned param requirement ← reads per-param "param is
  reassigned across the recursive edge" from the proof
- Last-use move ← reads per-binding "this use is terminal"
  from ownership proof
- Callable exclusion ← reads "this function is referenced
  at value position" from expression analysis
- Match-bound cloning ← reads "this binding projects a
  shared owner" from pattern analysis

Each consumer reads a different fact. Each fact lives at a
single authority. The **composition** across passes is what
dissolves the exclusions — not a replacement concept. This
is the M1/M8/M9 principle: model once, read many, compose
at the usage site.

The prior draft of this unification invented a "transition
relation" type to replace the existing chain. That was the
wrong framing per Codex review and INVARIANTS.md §Single
authority. The right framing is: **the authorities already
exist; thread their outputs through the boundary so they can
be composed downstream.**

## The bottom line

The perf crisis was not what we thought it was. The fix was
6 lines. The lesson is: **we're running a compiler that should
catch these bugs, and it doesn't catch them yet, so we keep
committing them.** Every such bug is both a perf fix AND
evidence for KF-2's priority.

Profile first. Audit for re-derivation. Delete reconstruction
code when you thread facts forward. Log every case as a KF-2
target.
