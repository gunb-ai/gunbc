# Inference as Data — Experiment Sequence

> **Parent docs:** `docs/substrate-reflection-design.md` (the
> reflection framework this experiment sequence builds on),
> `src/v3/SELF_HOSTING.md` §5 (Stage 3 — `infer.dag`, which is
> what this sequence feeds into), `docs/v3-validation-experiments.md`
> (the v2-era experiment log this complements).
>
> **Status:** plan of record, not yet started. I0 is runnable
> today as a paper exercise; I1 and I2 are already scoped into
> the reflection PR and its prereq slate; I3-I8 run
> sequentially after reflection lands.
>
> **Purpose:** prove empirically that v3's inference pass — the
> hardest pipeline stage in the self-hosting arc — can be
> expressed as `.dag` data operating on substrate values. Each
> experiment has a specific success criterion (byte-identical
> output to the Rust reference) and a specific failure criterion
> (named expressiveness or substrate gap). If any experiment
> fails, the sequence stops and the gap becomes the next
> substrate decision.

---

## §1. Motivation

The reflection PR validates one half of the self-hosting
claim: **lenses can be `.dag` programs reading substrate
declarations as data.** The other half — **the compiler itself
can be `.dag` programs operating on substrate values** — is
still untested, and inference is its hardest sub-problem.

Inference is the hardest pipeline stage because:

1. **It's mutation-shaped in the Rust reference.** The current
   `src/v3/compiler/src/infer.rs` walks a mutable `Dag` and
   populates `Port.state` in place. A functional `.dag` form
   has to express this without mutation primitives.
2. **It's the deepest consumer of substrate facts.** Every
   other pipeline stage (parse produces surface syntax, lower
   produces declarations, emit consumes both) is a walk over
   data. Inference computes new facts — type resolutions,
   operator dispatches, template instantiations — that depend
   on the entire substrate being available as input.
3. **It's the place v2's analysis debt accumulates.** v2's
   `complexity.dag`, `ownership.dag`, and related analyses all
   depend on type-inferred port states to do their work. If
   v3's inference can't be expressed as data, every consumer
   analysis that depends on inference has to either duplicate
   v3's Rust state reading or wait for Stage 3 of the
   self-hosting arc.

This experiment sequence exists to answer one question
empirically: **is Stage 3 (infer.dag) tractable, and if so,
which substrate commitment does it require?** The answer
directly resolves the open question in
`src/v3/SELF_HOSTING.md` §5 about how inferred state is
represented.

**Why these experiments, and not a V2-style "throw at the wall"
session.** V2 validated five experiments (lambda = function,
provenance on binding, rule-table extensibility, purity lens,
ExprData variant cost) by building against each claim and
measuring. That approach works when the substrate is mature
enough to support the experiment. For inference-as-data, the
substrate might not be mature enough — the experiments
themselves are substrate-decision forcing functions. Each I-N
experiment is designed so its failure modes are **specific
substrate gaps**, not generic "inference is hard."

---

## §2. Three frontload concerns — decide BEFORE I0 runs

Before the experiment sequence starts, three decisions gate
the work. Each has to be made explicitly, documented, and
committed to.

### §2.1 The write surface is a real substrate decision

Reader lenses work. Writer "lenses" don't exist. Inference
writes port states, so inference-as-data needs a **write
surface** — a way for a `.dag` program to produce substrate
state as output. Three options:

- **Option (a) — `.dag` gets mutation primitives.** A `.dag`
  function can say "set this port's state to X" and the
  substrate remembers the change. Big language commitment.
  Breaks the immutable-substrate invariant that's been
  load-bearing throughout v3's design. The thesis's
  physics-plus-lens claim depends on substrate values being
  stable while lenses walk them; mutation primitives break
  that. **Rejected by construction** unless the other options
  both fail.
- **Option (b) — `.dag` is pure functional, inference returns
  a new Dag per step.** Each inference pass walks the input
  Dag, constructs a new Dag with additional facts populated,
  returns it. Multiple passes fold into `iterate(initial,
  infer_step)` where `infer_step: Dag → Dag`. Copy-on-write
  at the emit layer keeps the performance cost O(n) not
  O(n²) — only the port state changes, and port state is a
  per-port field, so structural sharing dominates. **Aligns
  with the project's values.**
- **Option (c) — structural deltas.** Inference returns a
  list of `(PortId, NewState)` tuples that a merger applies
  to the input Dag. Compromise between (a) and (b) — no
  mutation primitives, no full-Dag allocation per step, but
  the merger is a new substrate concept.

**Decision: Option (b), with a named fallback to (c).**

Rationale:

1. **Thesis alignment.** Functional Dag-to-Dag matches the
   project's existing reader-lens shape. Reflection already
   built the machinery for "read a Dag as input"; (b) just
   flips the arrow — "return a Dag as output." The mental
   model is consistent.
2. **Performance commitment becomes a requirement, not a
   claim.** Choosing (b) means "the emit layer MUST exploit
   structural sharing" is a load-bearing requirement. This
   forces the copy-on-write work early instead of discovering
   it as a surprise later.
3. **(c) is the fallback.** If (b)'s copy-on-write can't be
   made to work (e.g., because the substrate's current shape
   makes structural sharing expensive), (c) is the escape
   hatch. (c) is strictly more complex than (b) — it adds a
   new substrate concept (the delta merger) that lenses then
   have to understand — so (c) is chosen only if (b) is
   demonstrably impossible, not as a preference.
4. **(a) stays rejected.** Runtime mutation primitives would
   destabilize the entire reader-lens foundation. A lens
   currently assumes the Dag it's reading is stable; mutation
   primitives invalidate that assumption. The cost of
   rescuing the reader-lens model after admitting mutation is
   higher than the cost of building copy-on-write correctly.

**Commitment:** this decision is load-bearing for the
experiment sequence. Experiment I3 (the first write-surface
test) assumes Option (b). If I3 fails for performance reasons,
re-evaluate Option (c) before continuing; do not revisit
Option (a) unless both (b) and (c) fail.

### §2.2 "Inference" is a category, not a concern

v3's `infer.rs` currently does several distinct things
bundled under the label "inference":

1. **Identifier resolution** — walk `Var` nodes, resolve to
   `Declaration` via scope lookup.
2. **Type resolution** — walk type expressions in
   declarations, resolve to declaration references.
3. **Port type filling** — populate each port's `state` with
   the type its producer yields (literal types for `Value`,
   operator output types for `Transform`, etc.).
4. **Template substitution** — walk `Instantiation` nodes,
   substitute template arguments into the template's body.
5. **Pattern-with-payload resolution** — bind variant
   payloads in match arms to locals in the path body's scope.
   (Added in Prereq 2 during warm-elk's work.)
6. **Diagnostic emission** — attach diagnostics to ports that
   fail to resolve, via the existing fail-closed diagnostic
   mechanism.
7. **Cross-file forward reference resolution** — the
   `resolve_pending_identifiers` sweep that fills in stubs
   once all files have parsed.

These have **different hardness profiles**. Identifier
resolution is a scoped walk; template substitution is
substantially harder because it interacts with the SubstStack
mechanism. Bundling them as one "inference-as-data" research
stream risks making experiments too coarse — a stall on
template substitution would block progress on identifier
resolution if they're bundled.

**Commitment:** each sub-concern gets its own experiment in
the sequence. I5 is identifier resolution; I6 is template
substitution. They run sequentially per §4, but a failure on
I6 does not invalidate I5's result — each experiment has an
independent success criterion.

### §2.3 Decidability of the inference function itself

v3's decidability invariant (`INVARIANTS.md` §"Decidability")
says every `.dag` program must terminate by construction:
bounded folds, bounded recursion via `Loop`, no unbounded
iteration. Inference rules implemented as `.dag` functions
must fit under this invariant.

Most inference algorithms are naturally decidable:

- **HM-style identifier resolution** is linear in expression
  size with memoization — a bounded fold over the expression
  tree.
- **Template substitution** is bounded by the template's size
  — a bounded recursive walk.
- **Scope lookup** is bounded by scope depth, which is
  itself bounded by expression nesting.

But the constraint is worth **verifying early**, before
committing to the experiment sequence. If v3's current
`infer.rs` has any rule that doesn't fit the decidability
model (e.g., an unbounded fixpoint loop, an accidentally
recursive rule without a descent measure), that's the
constraint to surface now, not after weeks of implementation.

**I0 is the check.** It's a paper exercise: pick one
representative inference rule from `infer.rs`, transliterate
it to pseudo-`.dag`, and verify it fits the decidability
rules. See §3.0 for details.

**I0's output is one of three signals:**

- **Pass.** The rule transliterates cleanly and is decidable.
  The whole experiment sequence is on solid ground.
- **Pass with caveats.** The rule transliterates but uses a
  pattern (e.g., unbounded `while let`) that maps to `Loop`
  with a specific descent measure. Note the caveats and
  proceed — they're not blockers, but future experiments
  should follow the same pattern.
- **Fail.** The rule has a structural shape that doesn't fit
  decidability. Stop the experiment sequence. Decide whether
  to (a) change the rule in v3's Rust form, (b) weaken the
  decidability invariant, or (c) abandon inference-as-data
  for that rule class. This is the cheapest possible place to
  hit a wall.

---

## §3. The experiment sequence

Eight experiments, I0 through I8. Each has a specific success
criterion, a specific failure mode, and a specific gap it
would surface if it fails. I0 runs today; I1 and I2 are
already scoped into the reflection PR and its prereq slate;
I3 through I8 run sequentially after reflection lands.

### §3.0 Experiment I0 — Decidability paper exercise

**Purpose:** cheapest possible check that v3's inference
algorithm is expressible under the decidability invariant.
Runs as a paper exercise — no substrate changes, no grammar
extensions, no code. One implementer + `infer.rs` + a pencil.

**The rule to transliterate:** the simplest real rule in
`infer.rs` — probably **literal type filling for `Value`
nodes**. Specifically:

> A `Value` node whose payload is an integer literal has port
> type `Int`. A `Value` node whose payload is a boolean
> literal has port type `Bool`. A `Value` node whose payload
> is a string literal has port type `String`.

In the current Rust `infer.rs`, this is the `Behavior::Value`
match arm. It reads the literal's kind and sets
`Port.state = PortState::Resolved(literal_type)`.

**Instructions:**

1. **Find the rule in `src/v3/compiler/src/infer.rs`.** Search
   for the `Behavior::Value` arm in the main inference walker
   (likely in a function named something like `infer_behavior`
   or `walk_node`).
2. **Write it in pseudo-`.dag` syntax** assuming the
   reflection prerequisites (field access, match-with-payload,
   higher-order calls) have already landed. Rough shape:

   ```
   fn infer_value(v: ValueNode, d: Dag) -> Dag =
     match v.payload {
       IntLit(_)    => d.with_port_type(v.result_port, Int)
       BoolLit(_)   => d.with_port_type(v.result_port, Bool)
       StringLit(_) => d.with_port_type(v.result_port, String)
     }
   ```

   The `d.with_port_type(port, type)` is a pseudo-operation
   representing the write surface from §2.1 Option (b) — it
   returns a new Dag with one additional port state populated.
   The real form once I3 lands may look different; the paper
   exercise doesn't care about the exact syntax, only the
   shape.

3. **Check each structural element against the decidability
   rules:**

   | Element | Decidability check | Pass/Fail |
   |---|---|---|
   | `match v.payload { ... }` | Exhaustive match over a finite Disj (the payload variants). No iteration. Bounded by number of variants. | ✅ decidable |
   | `d.with_port_type(...)` | A single function call with no implicit iteration. The write surface function itself must be decidable, but that's I3's problem, not I0's. | ✅ decidable at call site |
   | Return value | A new Dag; no mutation, no side effects. | ✅ decidable |
   | Whole function | No recursion, no iteration, no unbounded search. Strictly linear in the number of Value variants. | ✅ decidable |

4. **Document the result.** Write a short note (1 paragraph)
   at the bottom of this doc in §5 Open questions, naming the
   rule you checked, the outcome (pass / pass-with-caveats /
   fail), and any surprises.

**Success criterion:** the rule fits the decidability model
without introducing patterns that v3's current `.dag` grammar
can't express under the invariant.

**Failure criterion:** the rule requires (a) unbounded
iteration, (b) unstructured recursion without a descent
measure, (c) a dependency on state external to the Dag input,
or (d) a side-channel that can't be expressed as "take Dag,
return Dag." Any of these is a structural blocker that
predates the experiment sequence and must be addressed
before I1 runs.

**Estimated effort:** 1-4 hours of focused reading +
transliteration + checking. **This is the cheapest experiment
in the sequence and should run immediately** — before any
implementation work on reflection or prereqs, since its
result either validates the whole sequence or stops it cold.

**Why I0 before I1.** I1 is `lens_unused_parameters`
migration — substantial implementation work. If I0 fails for
some reason (a decidability wall that nobody anticipated),
I1's implementation work was wasted. I0 runs in hours and
either greenlights the sequence or surfaces the constraint
before any lines of code are written.

### §3.1 Experiment I1 — Reader lens enumerating substrate

**Purpose:** prove that a `.dag` program can walk substrate
declarations via field access + pattern matching. Already
scoped as **Prereq 4 (`list.dag`) + reflection PR** in the
reflection design.

**Implementation:** `dsl/lenses/unused_parameters.dag`. The
`.dag` form of the existing Rust `lens_unused_parameters.rs`,
using field access on `Dag`/`Behavior`/`BindNode`, pattern
matching with payload binding, and `std/list.dag`'s `fold`/
`filter`/`enumerate`/`map`.

**Success criterion:** for every test input in
`m1_3_lens_unused_parameters_test.rs`, the `.dag` form
produces byte-identical output to the Rust form.

**Failure criterion:** any expressiveness gap that the
reflection PR's compositional model can't span — e.g., a
case where field access or pattern matching isn't
sufficient. Such a gap would force a return to the query-
primitive layer or a substrate extension.

**Status:** scoped into the reflection PR's acceptance
criteria. Success here is a gate for the reflection PR to
merge.

### §3.2 Experiment I2 — Reader lens composing facts

**Purpose:** prove that a `.dag` program can FOLD over
substrate data, not just enumerate it. Fold is strictly
richer than enumeration because it requires the accumulator
to be threaded through the walk in a type-preserving way.

**Implementation:** migrate `lens_cost.rs` (80 lines) to
`dsl/lenses/cost.dag`. The cost lens walks each node's inputs
recursively and combines per-node costs via a structural
fold. Cost composition (sum for Transform inputs, max for
Branch paths, etc.) is the first non-trivial use of fold
over substrate data.

**Success criterion:** for the same test inputs as
`lens_cost.rs`'s Rust form, the `.dag` form produces the
same cost values node-by-node.

**Failure criterion:** the fold pattern can't express the
per-behavior composition rule (e.g., Branch's "max over
paths" is a fold-with-initial-zero pattern that the fold-via-
list-recursion form from Prereq 4 can't express cleanly).

**Status:** follow-up PR after reflection lands. Part of the
L2 consumer migrations (see `docs/substrate-reflection-design.md`
§12.5 M1 — though `lens_cost.rs` is listed there as the
"placeholder for complexity," so I2 is a small slice of the
larger M1 migration).

### §3.3 Experiment I3 — Pure function `Dag → Dag` that adds one declaration

**Purpose:** FIRST write-surface test. Every experiment
before I3 is reader-only; I3 is the first time a `.dag`
program has to return substrate state, not just consume it.
The decision from §2.1 (Option b) gets tested empirically
here.

**Implementation:** a trivial constructor function. Something
like:

```
fn add_hello_world(d: Dag) -> Dag =
  d.with_new_declaration(
    Declaration {
      name: "hello_world",
      connective: Atom(Literal(StringLit("Hello, world!"))),
      ...
    }
  )
```

`d.with_new_declaration(decl)` is the write-surface
primitive: returns a new Dag with the additional declaration
appended. The primitive itself is part of §2.1's Option (b)
commitment — its implementation is where copy-on-write gets
tested.

**Success criterion:** calling `add_hello_world` on an
arbitrary Dag produces a new Dag with exactly one more
declaration than the input, and the original Dag is unchanged
(referentially). The copy-on-write implementation must not
copy the entire declaration table — only the "changed" part
(the tail of the declaration list) is allocated fresh.

**Failure criterion:** (a) the write surface requires
mutation primitives that break the immutable-substrate
invariant, or (b) copy-on-write can't be made efficient
enough (i.e., every `with_new_declaration` call is O(n) in
the declaration count, not O(1)), or (c) the new Dag's
downstream consumers can't tell it's a "same-shape"
continuation of the input.

**Status:** requires reflection PR + write-surface substrate
extension. First post-reflection experiment.

**Substrate question this resolves:** SELF_HOSTING.md §5's
open question about how inferred state is represented. If I3
passes, option (b) is validated empirically and Stage 3's
design is unblocked.

### §3.4 Experiment I4 — One inference rule as data

**Purpose:** FIRST real inference rule implemented as a
`.dag` function. The rule chosen is the same as I0's paper
exercise: literal type filling for Value nodes.

**Implementation:** `dsl/infer/literal_types.dag`. The
function takes a Dag and returns a new Dag in which every
Value node's `result_port` has its `Port.state` populated
with the correct literal type.

```
fn fill_literal_types(d: Dag) -> Dag =
  fold(d.nodes, d, |acc, node|
    match node {
      Value(v) => acc.with_port_type(v.result_port, literal_type_of(v.payload))
      _        => acc
    }
  )

fn literal_type_of(payload: AtomPayload) -> TypeShape =
  match payload {
    IntLit(_)    => Int
    BoolLit(_)   => Bool
    StringLit(_) => String
    _            => ??? // fail-closed; not every AtomPayload is a literal
  }
```

**Success criterion:** run `fill_literal_types` on a test
Dag (e.g., `let x: Int = 1 + 2`'s parsed form), compare the
resulting port states against the Rust `infer.rs` output.
Byte-identical match on every Value node's port state.

**Failure criterion:** (a) the fold pattern can't thread the
accumulator Dag through the node walk (this would be a write-
surface problem), (b) the `match node` pattern can't extract
the payload correctly (pattern-with-payload problem),
(c) `literal_type_of` can't express the exhaustive match
(coproduct modeling problem).

**Status:** first experiment after I3. Unblocked by I3's
write-surface validation.

### §3.5 Experiment I5 — Identifier resolution as data

**Purpose:** first inference rule that needs to walk the
**ambient scope**, not just a single node. Scope is more
complex than node-local data because it depends on the
enclosing expression's hierarchy.

**Implementation:** `dsl/infer/identifier_resolution.dag`.
The function walks every `Var` node in the Dag, looks up its
name in the appropriate scope, and sets the var's port state
to a reference to the resolved declaration.

The hard part is **representing scope in `.dag` data**. Two
options:

- **Option a:** a `Scope` is a `.dag` type that threads
  through the walk. Each recursive call passes the current
  scope plus any new bindings.
- **Option b:** scope is a computed property of the Dag's
  structural position. A `Var`'s scope is derived from its
  enclosing Bind / Branch / Loop.

Option (b) is thesis-aligned ("scope is structural, not
auxiliary state") but requires the substrate to carry enough
information that scope can be computed purely from position.
Option (a) is simpler but introduces a new stateful concept.

**Success criterion:** for a test Dag containing Vars and
Declarations in various scopes, the `.dag` form produces the
same resolved-var port states as Rust's `infer.rs`.

**Failure criterion:** scope representation forces a new
substrate concept that isn't already present. This is a
substrate gap that would need to close before I5 can
succeed.

**Status:** after I4. Forces the decision on scope
representation, which is a real substrate-shape question.

### §3.6 Experiment I6 — Template substitution as data

**Purpose:** the heart of polymorphism and the place where
v2's `SubstStack` mechanism lives. If this works as data,
the hardest inference sub-concern is cracked.

**Implementation:** `dsl/infer/template_substitution.dag`.
Walks every `Instantiation` node in the Dag, substitutes the
template arguments into the template's body, and produces a
new Dag with the substituted form populated.

The critical dependency is **Prereq 0 (template
instantiation extension for function-typed parameters)** from
the reflection design doc. Once Prereq 0 lands, function-
typed template arguments are first-class, and I6 can
substitute them the same way type arguments get substituted.

**Success criterion:** for a test Dag containing generic
declarations and their instantiations (e.g.,
`Monoid<Int>` inhabiting `Monoid<T>`), the substitution pass
produces the same monomorphized forms as Rust's `infer.rs`.

**Failure criterion:** template substitution needs a
substrate concept that isn't there. Specifically: if v3's
current `SubstStack` mechanism doesn't generalize to a
functional substitution fold, this experiment surfaces it.

**Status:** after I5. Depends on Prereq 0 (reflection PR
slate). Harder than I5 — probably 3-4 weeks of focused work.

### §3.7 Experiment I7 — Full inference pass on a minimal program

**Purpose:** assemble I4, I5, I6 into a complete inference
pass. The first end-to-end test of inference-as-data.

**Implementation:** `dsl/infer/full_pass.dag`. Composes the
three sub-experiments into a fixpoint-or-sequential pipeline
(depending on whether the sub-passes have cross-dependencies).
Runs on a minimal program — probably `let x: Int = 1 + 2`.

**Success criterion:** after running `full_pass.dag` on the
test program's Dag, every port state matches the Rust
`infer.rs` output byte-for-byte, and every diagnostic
produced matches in shape and placement.

**Failure criterion:** the sub-passes compose but produce
different output than Rust's inference. This is the hardest
kind of failure to diagnose — one of the sub-passes is
subtly wrong, or the composition order matters in a way I4/
I5/I6 didn't expose. The per-stage fixed-point check from
`src/v3/SELF_HOSTING.md` §11 is the diagnostic tool here: it
identifies which sub-pass is the first to diverge.

**Status:** after I4, I5, I6 all pass. The capstone of the
experiment sequence for the inference-as-data claim.

### §3.8 Experiment I8 — Self-analysis

**Purpose:** empirical validation that the inference-as-data
work is self-consistent. Run `lens_unused_parameters.dag`
(from I1) against the `.dag` inference sources from I4, I5,
I6, and I7. Assert: zero findings.

**Implementation:** trivial — just invoke the existing lens
on the existing inference files.

**Success criterion:** zero unused parameters across all
inference sources. This is a weak claim on its own (unused
parameters are rare in well-written code), but it's the
simplest possible self-consistency check.

**Failure criterion:** if the lens reports findings in the
inference sources, either (a) the inference code has real
bugs, or (b) the lens is missing something about how
inference code uses its parameters (e.g., some parameters
are used via reflection-at-runtime that the lens can't
walk). Either way, the failure is diagnostic.

**Status:** last experiment. Cheap to run once I4-I7 have
landed.

**The bigger self-analysis win** is running `lens_cost.dag`
(from I2) on the inference sources and comparing the reported
cost against the Rust `infer.rs` source's cost (via some
cost-equivalent Rust analysis). This isn't a blocker for I8
passing; it's a bonus measurement of the thesis's
physics-plus-lens compression claim.

---

## §4. Sequential vs parallel

The experiment sequence is **strictly sequential** for the
following dependencies:

```
I0 (gate) → I1 → I2 → I3 (write surface) → I4 → I5 → I6 → I7 → I8
```

Each experiment's failure could invalidate the subsequent
ones, so early experiments must pass before later ones start.
Specifically:

- **I0 is a gate.** If I0 fails, the whole sequence stops
  and the decidability wall gets addressed.
- **I1 and I2 are already scoped into the reflection PR.**
  They don't block on each other directly, but both depend
  on the reflection PR shipping. I2 can start as soon as I1
  has a working infrastructure even if I1 hasn't fully
  passed.
- **I3 is the write-surface gate.** I4 through I8 all depend
  on I3's write-surface mechanism. If I3 fails, the fallback
  to Option (c) from §2.1 kicks in.
- **I4 → I5 → I6** each exercise a different sub-concern but
  could in principle run in parallel once I3 has landed. In
  practice, bundling them sequentially gives each experiment
  a clean success criterion before the next one starts.
- **I7** is a capstone that requires I4, I5, I6 all passing.
  It can't start until they're done.
- **I8** is cheap and runs after I7.

**Parallelization is possible at the I4/I5/I6 level** if two
implementers are working on the sequence simultaneously. The
sequence above assumes one implementer; a second implementer
could pick up I5 while the first is on I4. Coordinate to
avoid substrate-extension collisions.

---

## §5. Relationship to existing work

**`docs/substrate-reflection-design.md`:** the reflection
framework provides the foundation for I1 (and by extension
I2). The reflection prereq slate (Prereqs 0-5) lands before
any of I3-I8 start. Prereq 0 specifically (template
instantiation via SubstStack extension) is the dependency
for I6.

**`src/v3/SELF_HOSTING.md` §5 (Stage 3 — `infer.dag`):** this
experiment sequence directly feeds into Stage 3. The open
question in §5 — "how is inferred state represented?" — is
answered by I3's write-surface validation. If I7 passes,
Stage 3's implementation is a direct extension of the I4-I6
code.

**`docs/v3-validation-experiments.md`:** this doc complements
the v2-era experiment log. The v2 experiments (1-6) validated
the thesis at v2's scale; the inference-as-data experiments
(I0-I8) validate it at the self-hosting scale. Results land
in the same format — pass / pass-with-partials / fail, with
specific gaps named per outcome.

**`docs/substrate-reflection-design.md` §12.5 (consumer
migrations) and §12.6 (self-hosting horizon):** the
consumer-migration work (M1 complexity, M2 ownership, etc.)
happens AFTER reflection and BEFORE inference-as-data. The
experiment sequence here assumes reflection and consumer
migrations have both landed in some form. If consumer
migrations are in-flight when the experiment sequence
starts, they provide additional test corpus but also
additional risk — a consumer migration that fails might
indicate a substrate gap that inference-as-data would also
hit.

---

## §6. Open questions

1. **Which representative rule for I0?** The doc recommends
   **literal type filling** because it's the simplest
   inference rule in `infer.rs`. An alternative is **scope
   resolution**, which is harder but more likely to surface
   structural issues early. Recommendation stands with
   literal type filling; if it passes trivially, run a second
   I0 on scope resolution to stress-test.

2. **How is scope represented in `.dag` data for I5?** Two
   options in §3.5 (threaded state vs. structurally-derived
   from Dag position). Decision deferred until I5 starts —
   I4 doesn't need scope, so the decision can wait until I4
   validates the write surface.

3. **What's the minimal program for I7?** The doc recommends
   `let x: Int = 1 + 2` because it's concrete and exercises
   literal type filling + operator resolution. A slightly
   larger program (e.g., `fn add(a: Int, b: Int) -> Int =
   a + b`) would also exercise identifier resolution and
   pattern-with-payload. Pick whichever surfaces more
   substrate gaps; larger is better if the cost is bounded.

4. **If I0 surfaces a decidability wall, what's the escape
   hatch?** Three options: (a) change the rule in Rust to fit
   the decidability model, (b) weaken the decidability
   invariant (strongly rejected — the invariant is
   load-bearing), (c) abandon inference-as-data for that rule
   class and keep Rust as the implementation. Option (a) is
   preferred; Option (c) is the fallback if the rule is
   structurally undecidable.

5. **Does I8 actually catch anything?** Running
   `lens_unused_parameters` on inference source is a weak
   check; unused parameters are rare. A stronger self-
   analysis check would be running a SCHEMA-DIFF lens (from
   `src/v3/SELF_HOSTING.md` §10) between I4/I5/I6 and their
   Rust equivalents, reporting structural differences in the
   algorithms. That's out of scope for I8 as currently
   framed but could be added as I9 later.

6. **I0 results (to be filled in when I0 runs).** This
   section gets appended below when an implementer runs I0
   as a paper exercise. Include: the rule chosen, the
   transliterated pseudo-`.dag` form, pass/fail for each of
   the four decidability structural elements, any surprises,
   and recommendation for I1 to proceed or not.

---

## §7. When this doc updates

Living design note. It evolves as:

- **I0 runs** and its result populates §6 Q6.
- **Each experiment lands** and its per-experiment section
  graduates from "plan" to "in flight" to "result," with the
  actual outcome replacing the success/failure speculation.
- **Substrate gaps surface** during experiments — each
  becomes a tracked item, either scoped into the experiment
  that surfaced it or pushed to a follow-up PR.
- **Open questions resolve** — §6 shrinks as the decisions
  land.

**When this doc is complete.** All eight experiments have
either passed or hit a named gap. The inference-as-data
claim is validated empirically OR the specific substrate
constraints that block it are documented. Either way, the
Stage 3 `infer.dag` migration plan has concrete ground truth
to execute against.
