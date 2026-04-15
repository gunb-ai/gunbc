# v3 Self-Hosting — Design Note

> **Parent docs:** `THESIS.md`, `docs/substrate-reflection-design.md`
> (§12.6 for the brief), `src/v3/ROADMAP.md` (M3 milestone).
>
> **Purpose:** specify how v3's compiler pipeline migrates from
> Rust into `.dag`, how the stages interact with the reflection
> framework and consumer migrations, and what "self-hosting"
> means concretely for this project.

---

## §1. What self-hosting means for v3

**Self-hosting** means v3's entire compiler pipeline — parse,
lower, infer, emit — is written in `.dag`, compiled by v3's
own compile loop, and produces the same byte-for-byte output as
the current Rust stage0. The Rust code at `src/v3/compiler/src/`
becomes a **bootstrap seed**: kept for fresh-checkout
bootstrapping and for the initial compilation of the `.dag`
pipeline files, but no longer the authoritative compiler. The
"real" compiler is the `.dag` one; Rust stage0 exists to get it
off the ground.

**v2 is already self-hosting.** `src/v2/` contains the v2
compiler written in v2's own `.dag` language plus a minimal
Rust stage0 (`src/v2/stage0/`) that can compile the `.dag`
source. v2's pipeline stages — `02_parse.dag`, `03_resolve.dag`,
`04_infer.dag`, `05_emit_rust.dag`, etc. — are `.dag` functions
operating over v2's IR. v3 is catching up to v2's self-hosting
model, not inventing it.

**What changes with self-hosting:**

- **The compiler becomes a substrate fact.** The pipeline stages
  are ordinary Declarations in the Dag the compiler consumes.
  A lens can analyze the compiler's own behavior the same way
  it analyzes any other `.dag` program. `lens_complexity`
  running on `parse.dag` reports the parser's cost. This is
  the thesis's "substrate describes everything including
  itself" claim made empirical.
- **New target languages are declarations.** Once
  `emit.dag` reads the realization spec and produces target
  source, adding a new target (Python, Go, TypeScript, Swift)
  is a new realization spec file, not a compiler rewrite. The
  thesis's "one file edit per new feature" metric applies to
  targets, not just to types.
- **Ingestion languages become declarations too.** Once
  `parse.dag` reads a grammar declaration and produces
  `SurfaceItem` trees, ingesting Python (or C++, or any other
  source language) is a new grammar spec file. The omni-ingestion
  mechanism the user named earlier falls out of grammar-as-data.
- **The bootstrap loop compresses.** Every round of compiler
  development becomes "edit the `.dag` source, run the existing
  compiled pipeline against the edits, verify the result matches
  the previous pipeline." When the compiler compiles itself and
  the output matches the pre-edit compiler, the edit is sound.

---

## §2. Dependency order

```
L0 — Substrate stable                  [SHIPPED at M1(3)]
      │
L1 — Reflection framework              [in design — docs/substrate-reflection-design.md]
      │    • substrate.dag declares Dag / Behavior / etc.
      │    • field access, pattern match with payload, lambda,
      │      higher-order calls via template instantiation
      │    • first lens (lens_unused_parameters) migrates
      │    • list.dag loads
      │
L2 — v2 consumer migrations            [see docs/substrate-reflection-design.md §12.5]
      │    M1: dsl/lenses/complexity.dag  (5490 lines from v2)
      │    M2: dsl/lenses/ownership.dag   (719 lines)
      │    M3: dsl/lenses/effects.dag     (66 lines)
      │    M4: dsl/lenses/trace.dag       (223 lines)
      │
L3 — Pipeline stages in .dag           [THIS DOC]
      │    Stage 1: emit.dag   — Dag → target source strings
      │    Stage 2: lower.dag  — SurfaceItems → Declarations + Behaviors
      │    Stage 3: infer.dag  — Dag → Dag with port state
      │    Stage 4: parse.dag  — tokens → SurfaceItems
      │
L4 — Full self-hosting (M3)            [long-term]
           v3's compiler is .dag code
           Rust stage0 is vestigial (bootstrap seed)
```

**Gating rules:**

1. **L2 cannot start until L1 ships.** Consumer migrations read
   substrate facts through the reflection framework; no
   reflection, no lens migrations.
2. **L3 cannot start until L1 ships AND at least M1 of L2 is
   underway.** Consumer migrations prove the reflection
   framework works on real analysis code before the compiler
   itself migrates. Without consumer migrations as a test corpus,
   pipeline migrations are a leap of faith.
3. **L3 stages proceed bottom-up.** Emit first (easiest, already
   half-structured), then lower, then infer, then parse. Each
   later stage's migration benefits from the earlier stages
   being in `.dag` form (e.g., `lower.dag` can call `emit.dag`
   for debug-dump purposes once both are in `.dag`).
4. **L4 is the state after all L3 stages ship.** There is no
   separate "make self-hosting work" milestone; it's the
   emergent consequence of completing L3.

---

## §3. Stage 1 — `emit.dag`

**Current:** `src/v3/compiler/src/emit_rust.rs` (~340 lines
today; will grow as FieldBinding lookup lands in the reflection
PR).

**What it does:** walks a compiled `Dag`, looks up realization
entries in `rust.dag`, and produces Rust source code as a
string. Pure walk + template substitution + realization lookup.
No inference, no mutation, no cross-module resolution.

**Why emit first:** it's the easiest stage to port because it's
already structured as a walk over the substrate. Most of its
"logic" is table-driven (the `rust.dag` realization index), and
the table is already in `.dag` form. The `.dag` version of
`emit_rust` reads the same realization entries, walks the same
Dag, and concatenates the same template output — just expressed
in `.dag` instead of Rust.

**Blocker dependencies:**

- L1 (reflection) — needs substrate types in `.dag` to walk
- `std/string.dag` — string concatenation, formatting,
  placeholder substitution (the `%N/%T/%V/...` template language)
- L2 M3 (effects) is recommended but not strictly required —
  having the effects lens running validates the lens
  infrastructure independently before pipeline stages start

**Expected port size:** ~300-500 lines of `.dag` (roughly
comparable to the Rust version, possibly slightly longer due
to `.dag`'s more verbose function definitions). The algorithm
is the same.

**Substrate extensions surfaced by the port:**

- **`std/string.dag`** — doesn't exist at M1(3) scope. Adding
  it means declaring `String` as more than an opaque primitive:
  concat, length, replace, substring, split, format. Likely a
  `Monoid<String>` inhabitance plus per-operation declarations.
  This is its own follow-up, but it's cheap if done right.
- **Template substitution as a data operation.** The existing
  emit_rust has `render_template` in Rust that walks `%N/%T/%V`
  placeholders and substitutes. The `.dag` version needs this
  as a `.dag` function. Options: (a) inline the substitution
  logic in `emit.dag`, (b) model templates as a substrate
  concept with typed placeholder slots and let the renderer be
  generic. (b) is more thesis-aligned but bigger.

**Acceptance criteria:**

- [ ] `src/v3/spec/emit.dag` (bootstrap staging location) exists
- [ ] Compiles under the grammar + library available after L1 ships
- [ ] For every v3 test that currently invokes `emit_rust.rs`,
      the `.dag` version produces byte-identical Rust output
- [ ] The Rust `emit_rust.rs` is deleted once parity holds

**Open questions:**

- How does `.dag`'s IO boundary work? `emit.dag` needs to
  produce a `String` and return it. The caller (test, CLI)
  receives the string. That's a normal function return value;
  no new machinery needed. But **writing** to a file is an
  effect, and effect modeling in v3 is unresolved. Solution:
  `emit.dag` returns the string, the caller writes it. Effects
  stay at the boundary.
- How does the `.dag` version bootstrap? Rust stage0 compiles
  `emit.dag` to a Rust function, and the Rust test suite calls
  that function. That's the standard "compile .dag to Rust and
  run" pattern. No meta-circularity at this stage.

---

## §4. Stage 2 — `lower.dag`

**Current:** `src/v3/compiler/src/lower.rs` (~2000 lines).

**What it does:** takes parsed `SurfaceItem` / `SurfaceExpr`
trees from `parse.rs` and produces `Declaration`s (type
declarations, function declarations, data declarations) plus
sub-DAG `Behavior`s (function bodies, nested expressions).
Walks the surface tree, allocates DeclarationIds, resolves
name references, runs the `resolve_pending_identifiers` sweep
for cross-file forward references.

**Why second:** lower is the second cleanest stage after emit
because it's a walk + construction. The algorithm is "for each
SurfaceItem, build the corresponding Declaration(s) and
Behavior(s)." Pattern matching on SurfaceItem variants is
available post-L1. The tricky parts (forward reference
resolution, scope management) have well-defined shapes.

**Blocker dependencies:**

- L1 (reflection)
- Stage 1 (`emit.dag`) — not strictly a blocker but recommended,
  because a `.dag` lower that wants to debug-dump an intermediate
  Dag benefits from a `.dag` emitter
- `SurfaceItem` and `SurfaceExpr` must be declared in the
  substrate. Today they're Rust enums in `parse.rs`. For
  `lower.dag` to pattern-match on them, they need to become
  `.dag` Disj declarations. This is a prerequisite substrate
  addition (probably `src/v3/std/surface.dag` declaring the
  surface AST).

**Expected port size:** ~1500-2500 lines of `.dag`. Comparable
to the Rust version. The algorithm is a walk + construction,
so size tracks the number of surface-form variants.

**Substrate extensions surfaced by the port:**

- **Surface AST as substrate declarations.** `SurfaceItem`,
  `SurfaceExpr`, `SurfacePattern`, `SurfaceType`, etc. become
  Disj declarations in `std/surface.dag`. This parallels
  what reflection does for `Dag`/`Behavior` — it declares the
  parser's output in the substrate so downstream stages can
  pattern-match on it.
- **Symbol table / scope machinery.** Today `lower.rs` uses a
  `HashMap<String, DeclarationId>` for per-scope symbol lookup.
  The `.dag` version needs either a substrate construct for
  scopes (e.g., `type Scope { parent: Scope?, bindings: Map
  <String, DeclarationId> }`) or a functional approach (pass
  the scope as a value through recursive calls).
- **Cross-file forward reference resolution.** The
  `resolve_pending_identifiers` sweep walks unresolved stubs and
  fills them in. In `.dag` form, this is a traversal function
  that takes a Dag-with-stubs and returns a Dag-without-stubs.
  Straightforward but needs the substrate to expose the stub
  state as a field.

**Acceptance criteria:**

- [ ] `src/v3/spec/lower.dag` exists, depends on `surface.dag`
- [ ] Compiles under the grammar + library after L1
- [ ] For every input the Rust lower handles, the `.dag` version
      produces byte-identical output (same declarations, same
      NodeIds, same DeclarationIds — modulo deterministic
      allocation order)
- [ ] The Rust `lower.rs` is deleted once parity holds

**Open questions:**

- **Allocation order determinism.** Rust `lower.rs` allocates
  DeclarationIds as it walks. The `.dag` version needs the same
  allocation order for byte-identical output. Either (a) the
  `.dag` version is written to allocate in the same order, or
  (b) the allocation order is a substrate fact (each declaration
  has a canonical-order field), or (c) byte-identity is relaxed
  to "semantically equivalent" which is harder to test.

---

## §5. Stage 3 — `infer.dag`

**Current:** `src/v3/compiler/src/infer.rs` (~1100 lines).

**What it does:** walks a Dag whose ports have
`PortState::Uninferred` state, runs type inference via the
substitution stack, populates each port with
`PortState::Resolved(TypeShape)` or `PortState::Unresolved`.
Handles template instantiation, inhabitance walks, operator
dispatch, etc.

**Why third:** infer is the trickiest stage because it's
mutation-shaped in Rust. The Rust version walks the Dag and
updates port state in place. A `.dag` version has to express
this functionally: `fn infer(d: Dag) -> Dag` that returns a new
Dag with the inferred state.

**Blocker dependencies:**

- L1 (reflection)
- Stages 1 and 2 (`emit.dag`, `lower.dag`) — highly recommended,
  because `infer.dag` needs to be debuggable end-to-end
- **Substrate decision: how is inferred state represented?**
  Option A: ports carry mutable state fields that the `.dag`
  infer function sets. Option B: the inferred state lives in a
  separate "inference result" Dag that the caller merges with
  the input Dag. Option C: infer returns a Dag-diff (a list of
  (PortId, TypeShape) tuples) that the caller applies. **This
  is the deepest substrate question for L3** and blocks
  `infer.dag` until it's answered.

**Expected port size:** ~1000-1500 lines of `.dag`. The
algorithm is the same as Rust's, but the functional expression
may be slightly longer due to avoiding mutation.

**Substrate extensions surfaced by the port:**

- **Inferred state representation** — the core open question.
- **Substitution stack as a substrate type.** Today `SubstStack`
  is a Rust data structure threaded through inference calls.
  The `.dag` version needs it as a declared type, probably a
  `List<TemplateArgument>` or similar.
- **Port-state Disj.** `PortState` is currently a Rust enum with
  three variants. Declaring it as a `.dag` Disj and letting
  `.dag` code pattern-match on the states is a L1 prerequisite
  (it's a substrate type that reflection wants anyway).

**Acceptance criteria:**

- [ ] `src/v3/spec/infer.dag` exists
- [ ] Compiles under the grammar + library after L1
- [ ] For every input the Rust infer handles, the `.dag` version
      produces byte-identical port state output
- [ ] The Rust `infer.rs` is deleted once parity holds

**Open questions:**

- **Mutation vs return-new-Dag.** Substantive design decision.
  Suggestion: go functional. Infer takes a Dag, walks it,
  returns a Dag with the additional facts populated.
  Implementation detail: the "return a new Dag" is a copy at the
  top level but most substrate data (declarations, behaviors)
  stays shared via structural sharing. Only the port state
  changes, and port state is a per-port field, so the copy cost
  is bounded by the port count.
- **Fixpoint iteration.** Inference sometimes requires multiple
  passes (e.g., cross-referencing forward declarations). The
  functional form of fixpoint is `iterate(initial, step_fn)
  until step_fn(x) == x`. This needs the `==` equality check
  on Dag values, which needs the substrate to support structural
  equality. Probably a `std/eq.dag` prerequisite.

---

## §6. Stage 4 — `parse.dag`

**Current:** `src/v3/compiler/src/parse.rs` (~1600 lines).

**What it does:** takes source text (or `List<Token>` after
tokenization), produces `SurfaceItem` / `SurfaceExpr` trees.
Hand-written recursive-descent parser with custom rules per
syntactic construct.

**Why last:** parse is hardest AND most transformative. The
intermediate step (port `parse.rs` as-is to `.dag` functions
operating on `List<Token>`) is substantial but tractable. The
end state is much bigger: **grammar-as-data.** Parser rules
become `.dag` declarations, the parser itself is a generic
rule-interpreter. At that point, "ingest Python" becomes a new
grammar spec file — the parser is no longer v3-specific.

**Blocker dependencies:**

- L1 (reflection)
- Stages 1, 2, 3 (`emit.dag`, `lower.dag`, `infer.dag`) — all
  required for the meta-circular bootstrap (see §7)
- `std/string.dag` for source-text operations
- Token type as a substrate declaration (parallels Surface AST
  declarations from Stage 2)

**Expected port size — two phases:**

**Phase 4a — direct port.** `parse.dag` as a literal port of
`parse.rs`, one function per parse rule, recursive descent.
~1500-2000 lines of `.dag`. Produces identical Surface tree
output.

**Phase 4b — grammar-as-data.** Parser rules become `.dag`
declarations in a grammar spec file (`dsl/grammar/v3.dag` or
similar). The parser is a generic rule-interpreter that reads
the grammar spec and walks tokens accordingly. Phase 4b replaces
Phase 4a's hand-written rules with a data-driven form. Separate
project, own design note.

**Substrate extensions surfaced by the port:**

- **Token type as substrate declaration.** Parallels Surface
  AST from Stage 2. `std/token.dag` declares the token Disj.
- **Grammar production as substrate declaration** (Phase 4b).
  A new concept: a grammar rule is a declaration describing
  "when you see these tokens in this order, produce this
  SurfaceItem variant." Probably a `type Rule { name: String,
  pattern: List<TokenPattern>, construct: SurfaceConstruct }`
  or similar. This is a design-heavy addition.
- **Error recovery.** Parser error recovery (how to continue
  parsing after a malformed token sequence) is currently hand-
  coded in `parse.rs`. The `.dag` version needs a structural
  form. Probably out of scope for Phase 4a, in scope for 4b.

**Acceptance criteria (Phase 4a):**

- [ ] `src/v3/spec/parse.dag` exists
- [ ] Compiles under the grammar + library after L1 and earlier
      pipeline stages
- [ ] For every input file the Rust parser handles, the `.dag`
      version produces identical Surface tree output
- [ ] The Rust `parse.rs` is deleted once parity holds (but
      `tokenize.rs` stays as the stage0 tokenizer for now)

**Acceptance criteria (Phase 4b — separate project):**

- [ ] Grammar declaration in `dsl/grammar/v3.dag`
- [ ] Generic rule-interpreter replaces hand-written rules
- [ ] Adding a Python grammar spec (experimental) parses a
      subset of Python source

---

## §7. The meta-circular bootstrap

Once all four pipeline stages are in `.dag` form, the bootstrap
loop becomes self-referential. Here's the mechanics:

**Current state (pre-self-hosting):**

```
Rust stage0 (parse.rs + lower.rs + infer.rs + emit_rust.rs)
      │
      │ compiles
      ▼
User .dag source → Rust target source → rustc → executable
```

**Pre-self-hosting but post-L3 (pipeline in .dag):**

```
Rust stage0 (kept as bootstrap seed)
      │
      │ compiles
      ▼
src/v3/spec/{parse,lower,infer,emit}.dag → compiled pipeline
      │
      │ compiles
      ▼
User .dag source → compiled pipeline → Rust target → rustc → executable
```

The compiled pipeline stages REPLACE the Rust stage0 at the
compile step. Rust stage0 only runs during the initial
bootstrap of the pipeline stages themselves.

**Self-consistency check (the meta-circular test):**

```
Step 1: Rust stage0 compiles {parse,lower,infer,emit}.dag
        → produces compiled-pipeline-v1 (Rust code)
Step 2: compiled-pipeline-v1 compiles its own source files
        ({parse,lower,infer,emit}.dag)
        → produces compiled-pipeline-v2 (Rust code)
Step 3: Assert compiled-pipeline-v1 == compiled-pipeline-v2
        (byte-identical)
```

If Step 3 holds, the pipeline is self-consistent: compiling
the pipeline with itself produces the same output as compiling
it with stage0. That's the self-hosting fixed point. v2 has
this test; v3 will too.

**When Rust stage0 is vestigial.** After self-consistency is
proven, the Rust stage0 can be maintained as a bootstrap seed
(needed for a fresh checkout with no pre-compiled binary), but
it is NOT the source of truth. Any bug fix or improvement goes
to the `.dag` pipeline first; stage0 is updated to match only
when necessary (e.g., if the `.dag` pipeline grows a new
substrate feature that the stage0 needs to be able to parse).

The long-term story is: **stage0 is a minimal seed capable of
compiling the .dag pipeline, nothing more.** It stops growing
once it can produce a working `parse.dag` output. Every feature
after that lives in the `.dag` pipeline and is compiled by the
pipeline compiling itself.

---

## §8. Open questions

1. **Which pipeline stage goes first — `emit.dag` or something
   smaller?** §3 recommends emit because it's already walk-shaped
   and touches the realization index the reflection PR is
   already modifying. But emit is ~340 lines; a smaller stage
   might give a faster first data point. Counter-argument:
   there isn't a smaller stage — parse/lower/infer are all
   larger. Decision: emit first, as the doc currently says.
2. **Inferred state representation (Stage 3's core question).**
   Option A (mutable field), B (separate inference-result Dag),
   or C (Dag-diff). Each has different substrate implications.
   Worth a design note of its own before Stage 3 starts.
3. **Grammar-as-data (Stage 4b) scope.** Phase 4a is a direct
   port, cheap. Phase 4b is a separate project that changes
   the parser's fundamental shape. Decision: plan them
   separately, land 4a first, let 4b be its own design note.
4. **Stage0 minimum feature set.** What does the Rust stage0
   need to be able to parse in order to compile the first
   version of `parse.dag`? Probably a subset of v3's current
   grammar. Worth enumerating before Stage 4 starts so stage0
   doesn't drift larger than necessary.
5. **Meta-circular test infrastructure.** The self-consistency
   test in §7 needs tooling: run pipeline-v1 on its own source,
   diff the output against pipeline-v2. v2 has this test
   (`cargo test -p v2-compiler-tests ci_freshness` and
   `ci_fixed_point`). v3 will need the same.
6. **Interaction with L2 consumer migrations.** If `lens_complexity`
   is running on v3's pipeline stages during their migration,
   the lens's output should reveal whether the ported stage is
   "simpler" than the Rust version (fewer cost-algebra
   reconstructions, more substrate-derived facts). This is a
   measurable thesis claim — the migration should shrink
   complexity's reported cost in measurable ways. Needs
   explicit tracking during the migration.

---

## §9. Relationship to v2

v2 is the oracle and the reference implementation. Every v3
pipeline stage migration should:

1. Read the corresponding v2 `.dag` file as prior art
2. Identify which parts of v2's implementation are
   reconstruction heuristics (the "annotate_*" helpers in
   complexity are the clearest examples) and plan to dissolve
   them
3. Identify which parts of v2's implementation are genuine
   structural work and port those directly
4. Track any v2 feature the v3 substrate doesn't yet support as
   a prerequisite substrate extension

v2's pipeline files are the best reference for "what does this
stage actually need to do." Not to copy line-for-line — v3's
substrate is cleaner — but as the empirical ground truth for
"compilation works because this code runs correctly." If v3's
`.dag` port produces output that v2's pipeline rejects, either
v3 has a bug or v2 has a legacy assumption that v3 is
deliberately removing. Investigate either way.

**v2 pipeline files, for reference:**

- `src/v2/02_parse.dag`
- `src/v2/03_normalize.dag` / `src/v2/03_resolve.dag`
- `src/v2/04_infer.dag` / `src/v2/04_lookup.dag` /
  `src/v2/04_resolve.dag` / others
- `src/v2/05_emit_rust.dag` (+ `_python.dag`, `_go.dag`)

These are multi-thousand-line files. Direct ports are not
expected; the reference is for architectural patterns, not
source.

---

## §10. When this doc updates

`SELF_HOSTING.md` is a living design note. It evolves as:

- Each pipeline stage migration starts — the relevant section
  graduates from "plan" to "in-flight," with real
  implementation details replacing the current speculation.
- Substrate extensions surface during migration — each becomes
  a tracked item, either moved into a follow-up prereq PR or
  absorbed into the stage's own scope.
- Open questions resolve — the list in §8 shrinks as decisions
  land.
- The meta-circular bootstrap test lands — §7 moves from
  design to documentation of the real test.

**When this doc is complete.** The v3 compiler is fully
self-hosting and Rust stage0 is vestigial. At that point this
doc is archived as a historical record of the migration path,
and ongoing compiler work happens in the `.dag` pipeline files
directly.
