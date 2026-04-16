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
L1 — Reflection framework              [prereqs shipping — reflection PR next]
      │    • Prereq 0 (HoF via 1c): #460 merged
      │    • Prereq 1 (FieldProject): #458 merged
      │    • Prereq 2 (Path.binding): #458 merged
      │    • Prereq 3 (contextual lambda): #460 merged
      │    • Prereq 4 (list.dag bootstrap): #463 in flight
      │    • Prereq 5 (pipe sugar): #462 merged
      │    • Reflection PR: next after prereq slate completes
      │    • first lens (lens_unused_parameters) migrates in reflection PR
      │
      ├── L2 — v2 consumer migrations   [parallel with L2.5]
      │    │    M1: dsl/lenses/complexity.dag  (5490 lines from v2)
      │    │    M2: dsl/lenses/ownership.dag   (719 lines)
      │    │    M3: dsl/lenses/effects.dag     (66 lines)
      │    │    M4: dsl/lenses/trace.dag       (223 lines)
      │    │
      └── L2.5 — Per-stage domain modeling [parallel with L2]
           │    Emit domain:  extdeps/languages/ completeness,
           │                  std/formatting.dag
           │    Lower domain: std/surface.dag (surface AST types)
           │    Parse domain: std/token.dag, std/encoding.dag,
           │                  extdeps/platform/filesystem.dag,
           │                  grammar productions as data
           │    Infer domain: std/inference.dag, std/scope.dag,
           │                  std/substitution.dag
           │                  (blocked on I3 write-surface experiment)
           │
L3 — Pipeline stages in .dag           [blocked on L2.5 model for each stage]
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
2. **L2.5 cannot start until L1 ships.** Domain modeling declares
   types in `std/` and `extdeps/` using the same reflection
   infrastructure. Does NOT need L2 lens migrations to be done
   first — L2 and L2.5 are independent parallel work streams
   (L2 writes lenses in `dsl/lenses/`, L2.5 declares types in
   `std/` and `extdeps/`).
   **Distinction from the decorative-modeling pattern.** ROADMAP
   Track 17 warns that types with zero consumers are decorative.
   L2.5 is NOT that pattern. Each L2.5 type has a **named
   consumer**: the L3 stage implementation that reads it. The
   four-step sequencing in §2.2 ties modeling to implementation
   atomically — Step 1 (model review) exists BECAUSE Step 3
   (implementation) will consume the types. If an L2.5 type has
   no Stage N implementation that reads it, it doesn't belong in
   L2.5; it's decorative. The test is: "which function body in
   L3 reads this type?" If you can't name one, don't declare it.
3. **L3 stage N cannot start until L2.5's model for stage N is
   reviewed.** Implementation fills in function bodies over
   already-declared types. The model IS the prerequisite (§2.2).
   L2 should have at least M1 underway to prove the reflection
   framework works on real analysis code before pipeline stages
   start.
4. **L2.5 stages proceed in L3's implementation order.** Emit
   domain first (partially exists, smallest gap), lower domain
   second, parse domain third, infer domain last (blocked on I3
   write-surface experiment). Each domain model runs ahead of
   its stage's implementation by at least one step.
5. **L3 stages proceed bottom-up.** Emit first (easiest, already
   half-structured), then lower, then infer, then parse. Each
   later stage's migration benefits from the earlier stages
   being in `.dag` form (e.g., `lower.dag` can call `emit.dag`
   for debug-dump purposes once both are in `.dag`).
6. **L4 is the state after all L3 stages ship.** There is no
   separate "make self-hosting work" milestone; it's the
   emergent consequence of completing L3.

**L2.5 within each stage — the four-step sequencing from §2.2:**

| Step | What | When | Output |
|---|---|---|---|
| 1. Model review | Declare input/output types in `std/` and `extdeps/`. Apply invariants. Scrutinize structural decomposition. | L2.5 | `.dag` type files, reviewed |
| 2. Pipeline slot | Add the stage's typed function signature to the pipeline composition. `ExternalRealization` body. | L2.5 → L3 boundary | Pipeline `.dag` updated |
| 3. Implementation | Fill in the function body. Types are already declared; implementation is bounded by the model. | L3 | Stage `.dag` file |
| 4. Parity test | The `.dag` version produces byte-identical output to the Rust version. Delete the Rust version. | L3 completion | Rust file deleted |

### §2.1 The pipeline is a `.dag` composition with structural consumers

v2 is already self-hosting via `.dag` composition. `compile.dag`
calls stage functions through `let` bindings — it IS a dependency
graph in the same sense any `.dag` program is. The gap is not
that v2 "lacks a pipeline graph." The gap is:

1. **`bootstrap.dag`'s stage metadata has no consumers.**
   `CompilerStage`, `TransformContract`, `ChangeClassification`,
   `BootstrapStrategy` — 195 lines of type vocabulary declared,
   zero code reads any of it. The stage contracts are decorative.
2. **No structural self-analysis.** No lens runs on the pipeline
   itself. The compiler cannot analyze its own compilation steps
   the way it analyzes user programs.
3. **Stage boundaries are convention, not enforced.** Each stage's
   output type is `Dag` — which is also its input type. Nothing
   structurally distinguishes "Dag with all ports inferred" from
   "Dag with ports still Uninferred." The convention holds because
   the stages are called in the right order, not because the type
   system prevents calling them in the wrong order.

v3's pipeline keeps the `.dag` composition shape v2 already has
and adds **structural consumers** that make the stage metadata
load-bearing:

```
fn compile(source: String, file: String, spec: LanguageSpec) -> EmitResult {
  let parsed   = parse(source, file)
  let lowered  = lower(parsed.module)
  let inferred = infer(lowered.dag)
  emit(inferred.dag, spec)
}
```

Ordinary `.dag` — four `let` bindings with explicit dependencies.
The compiler sees this as a dependency graph. What's new is what
READS that graph:

1. **Stage contracts consumed by lenses.** A lens verifies "does
   `infer` produce a Dag where all ports have Resolved state?"
   This is a **lens-checked property**, not a type-level guarantee
   — the output type is still `Dag`, not a hypothetical
   `InferredDag` wrapper. The enforcement mechanism is the same
   as for any other `.dag` invariant: a lens reads the output and
   reports violations. If a future substrate version introduces a
   wrapper type that makes unresolved ports unrepresentable at the
   type level, that's a stronger guarantee — but the starting
   point is lens enforcement, same as `lens_layer_opacity` and
   `lens_unused_parameters`.
2. **Per-stage fixed-point consumed by regen.** The pipeline's
   typed stage boundaries give the fixed-point test structural
   comparison points. Instead of a file-level diff, regen compares
   pass1 and pass2 at each stage boundary — structural diffs, not
   textual diffs.
3. **Independent stages parallelize automatically.** Two
   post-inference analyses with no data dependency run
   concurrently by default. The thesis's "parallelism is not a
   feature, it is the default" applied to the compiler.
4. **The compiler is analyzable by its own lenses.**
   `lens_complexity` on `parse` reports the parser's cost.
   `lens_unused_parameters` on `infer` reports dead parameters.
   Self-analysis extends from "a lens analyzes a lens" to "the
   compiler analyzes itself."
5. **Schema migration patches read the pipeline.** When a stage's
   input/output type changes, the pipeline's typed composition
   shows which downstream stages are affected.

**Interaction with the L3 migration order.** Stages still
implement bottom-up (emit → lower → infer → parse). During
migration, each stage starts as `ArrowBody::ExternalRealization`
(backed by the Rust implementation) and transitions to
`ArrowBody::UserDefined` when the `.dag` body lands. The pipeline
composition doesn't change during this transition — it calls each
stage function regardless of whether the body is Rust-backed or
`.dag`-backed. Same pattern as `Int.add` being
`ExternalRealization` pointing at Rust `i64 + i64` today.

**The pipeline declaration ships FIRST (model before implement).**
Each stage implementation fills a slot in the declaration.
`ExternalRealization` is the honest scaffold for "this stage
exists but is still Rust." The pipeline is structurally valid at
every intermediate state.

### §2.2 Model every stage's domain to spec before implementation

v2 achieved self-hosting incrementally — types grew fields as
needed, coproducts grew variants as needed, contracts were added
after the fact. The result: `complexity.dag` is 5490 lines of
reconstruction heuristics compensating for facts that weren't
modeled upstream. The pipeline works, but the modeling was
done in parallel with implementation, and the implementation
kept outrunning the model.

v3 inverts this: **every pipeline stage models its inputs and
outputs to spec BEFORE the stage body is implemented.** Stage
implementation is filling in function bodies over already-declared
types — the same model-before-implement discipline the thesis
demands of user programs, applied to the compiler itself.

Concretely, each stage's domain model is reviewed and scrutinized
before any implementation starts:

- **Parse domain.** What IS a source file? Model files, encodings
  (ASCII/UTF-8), line endings, BOM handling as structural types in
  `std/`. Model the platform's filesystem interface as an extdep.
  Model the token types structurally — not a flat `TokenKind` enum
  that grows arms, but dimensional types where the thesis's
  structural decompression invariant applies (e.g., delimiters as
  `{ shape: BracketShape, side: Side }`, keywords as data in a
  table). Model the grammar productions as data, not as hand-coded
  recursive-descent functions. Every input the parser touches is
  declared, not assumed.

- **Lower domain.** What ARE the surface forms? Model
  `SurfaceItem`, `SurfaceExpr`, `SurfacePattern` as proper `.dag`
  Disj declarations with the normal invariants applied — structural
  decompression, no flat coproducts where dimensional records
  belong, every variant grounded in the thesis's four dissolution
  patterns. The surface AST becomes `std/surface.dag` — sibling to
  `std/substrate.dag`, analyzed by the same lenses.

- **Infer domain.** What ARE the inference rules? Model type
  resolution, template substitution, operator dispatch as data
  operations over the substrate. The cross-stage facts — input
  port state, output port state, diagnostic types, type shapes —
  are declared types in `std/`. The walker-local state —
  substitution stacks, scope structures, intermediate unification
  state — stays as implementation details inside `infer.dag`'s
  function bodies, NOT in `std/` (see substrate-vs-implementation
  split below).

- **Emit domain.** What IS a target language? Model the language
  spec structurally — not just "what types exist" but the full
  emission surface: indentation conventions, line-length limits,
  import ordering rules, formatting conventions, comment syntax.
  Model the target's type system as data in `extdeps/languages/`.
  The emitter reads ALL rendering decisions from declared data;
  the emitter code itself is a generic walk that never contains
  target-specific knowledge.

- **Between stages.** Model the stage boundaries as typed
  contracts. What's preserved across each boundary, what's
  recomputed, what diagnostics flow forward. The contracts are the
  pipeline's function signatures — first-class, not decorative.
  A lens verifies each contract holds.

**Substrate vs implementation — what goes in `std/` and what
doesn't.** Not every type the compiler uses belongs in `std/`.
The rule: **if a type crosses a stage boundary or is consumed by
a lens, it goes in `std/`. If it's internal to one stage's walk,
it's implementation.**

| Category | Goes in `std/` | Stays in stage `.dag` body |
|---|---|---|
| Input/output types | `SurfaceModule`, `Dag`, `EmitResult` | — |
| Diagnostic types | `Diagnostic`, `SourceSpan` | — |
| Substrate types | `Declaration`, `Behavior`, `TypeConnective` | — |
| Target language types | `LanguageSpec`, `Realization` (in `extdeps/`) | — |
| Token types | `Token`, `TokenKind` | — |
| Surface AST types | `SurfaceItem`, `SurfaceExpr` | — |
| Walker-local state | — | `SubstStack`, `Scope`, `UnificationState` |
| Intermediate inference state | — | Per-port resolution tracking |
| Helper accumulators | — | `fold` locals, working sets |

Walker-local state may graduate to `std/` IF a downstream consumer
needs it — for example, if a lens needs to inspect substitution
state mid-inference. But the default is implementation-local until
a cross-boundary consumer surfaces. "Model to spec" means the
BOUNDARIES are modeled, not the internals.

**The v2 lesson this prevents.** v2's pipeline stages accreted
types reactively: a stage needed a fact, so a field was added to
the IR, and downstream stages adapted. The adaptation was often
reconstruction — re-deriving the fact from incomplete data instead
of receiving it as a typed edge from the stage that computed it.
Modeling to spec upfront means the typed edges exist BEFORE any
stage runs, so reconstruction has no reason to form. The
`annotate_descent_evidence` pattern (33 heuristics in v2's
complexity pass) is the canonical failure mode this prevents.

**The principle:** the `.dag` compiler gets the `.dag` treatment
at its boundaries. Every type that crosses a stage boundary or is
consumed by a lens — files, tokens, surface trees, substrate
declarations, target language constructs, platform capabilities —
is modeled with the same rigor as any user-facing `.dag`
declaration. Structural decompression applies. Layer opacity
applies. Facts flow forward applies. No special exemption for
"this is compiler-internal." The invariants are universal at
boundaries; walker-local implementation details are free to use
whatever internal representation the stage author finds most
natural.

**Practical sequencing.** For each stage (emit → lower → infer →
parse), the work is:

1. **Model review.** Declare the stage's input/output types in
   `std/` and `extdeps/`. Apply the normal invariants. Review and
   scrutinize the model — structural decompression, no smuggled
   coproducts, no decorative fields. Draw on v2's pipeline files
   as empirical reference for "what does this stage actually need."
2. **Pipeline slot.** Add the stage's typed function signature to
   the pipeline composition. `ExternalRealization` body.
3. **Implementation.** Fill in the function body. The types are
   already declared; the implementation is bounded by the model.
4. **Parity test.** The `.dag` version produces byte-identical
   output to the Rust version for every test fixture. Delete the
   Rust version.

Step 1 is where the hard thinking happens. Steps 2-4 are
engineering bounded by Step 1's model.

### §2.3 Language-agnostic compiler core (Shape A only)

The compiler knows only substrate primitives: the six type
connectives (Atom, Conj, Disj, Arrow, Cardinality, Instantiation)
and the five L1 behaviors (Value, Transform, Branch, Loop, Bind).
It does NOT know what language it is parsing or emitting. `.dag`
is a grammar spec the parser engine reads. Rust is a language spec
the emission engine reads. Both are data. The compiler is the
engine, not the specs.

**Scope: Shape A targets only.** The thesis (§"Omni-emission")
makes a load-bearing distinction between Shape A (compiler-owned
programming-language emission: Rust, Python, Go, TypeScript) and
Shape B (user-program artifact generation: YAML, Terraform, SQL,
English docs, SPICE netlists). Treating Shape B artifacts as
compiler render targets is explicitly called a category error in
the thesis. This section's `compile(...)` parametrization ranges
over **Shape A targets only**. Shape B artifacts are outputs of
`.dag` user programs that walk typed values and produce strings —
the compiler emits the `.dag` PROGRAM (Shape A), and the program's
OUTPUT is the Shape B artifact. English, YAML, and SPICE are
never `EmissionTarget` values.

**The pipeline, with elaboration separated from parsing.**
MODELING.md's four-layer stack separates surface sugar from the
composition layer from the semantic kernel. `GrammarSpec` owns
syntax (tokens → surface tree). A separate `ElaborationSpec` maps
source-language surface forms to substrate declarations — the
lowering semantics. Collapsing both into `GrammarSpec` would
leave elaboration as implicit compiler behavior rather than
declared authority.

```
fn compile(
  source: String,
  grammar: GrammarSpec,            // syntax: tokens → surface tree
  elaboration: ElaborationSpec,    // semantics: surface → substrate
  target: EmissionTarget,          // Shape A language + rendering
) -> EmitResult {
  let parsed   = parse(source, grammar)
  let lowered  = lower(parsed, elaboration)
  let inferred = infer(lowered.dag)
  emit(inferred.dag, target)
}
```

Four parameters. `grammar` owns what surface forms exist.
`elaboration` owns how those forms become substrate declarations.
`target` owns how substrate projects onto a programming language.
The compiler core reads all three as data; it contains zero
language-specific knowledge.

**Compositional opacity at the I/O boundary.** The thesis says
"HTTP does not know or care what transport it is running over."
This constraint says: the compiler does not know or care what
language it is parsing or emitting. Below-boundary details
(syntax, keywords, operators, formatting, memory model, import
system) are unnameable to the compiler core. The compiler reads
structural edges from the specs; it never asks "am I compiling
Rust right now?"

The rename test from THESIS.md §"Compositional layering" extends:
rename every `.dag` keyword in the grammar spec, rebuild, verify
the compiler still works. If it does, the compiler is language-
agnostic. If it doesn't, there's a hardcoded language assumption
in the core.

#### §2.3.1 Emission/ingestion cost model

Any Shape A programming language that can represent the
substrate's primitives can be a compiler target. The question
isn't "can we emit to X?" — it's "at what cost?"

For each substrate primitive (Conj, Disj, Arrow, etc.) and each
L1 behavior, the `LanguageSpec` declares: how the target
represents it, and what the representation costs in space and
time.

The cost is structural and measurable. A well-matched target
(Rust) has expansion factor k ≈ 1 per nesting level — nested
structs are just nested structs, depth doesn't matter, total cost
is O(|substrate|). A poorly-matched Shape A target might have
k > 1, growing with nesting depth. The cost of emission increases
exponentially with structural depth for targets whose native
constructs don't match the substrate's compositional primitives.

This IS CX applied to emission itself. The `LanguageSpec` carries
a cost model alongside its representation declarations. The
compiler computes emission cost statically from the spec.

**Shape B cost model is analogous but separate.** A Shape B
`.dag` program (e.g., one that walks substrate declarations and
produces English documentation) has its own cost model: the
expansion factor per nesting level in the output format. English
has k >> 1 (subordinate clauses for each nesting level). YAML has
k ≈ 1 (indentation scales linearly). But these costs live in the
Shape B user program's own type spec, not in the compiler's
`LanguageSpec` — because the compiler doesn't own Shape B
emission.

**Ingestion has an additional dimension: information deficit.**
For emission, the compiler has full substrate and projects it —
the cost is expansion. For ingestion, the compiler has external
source and lifts it in — the cost is expansion PLUS deficit: how
much structural information does the source NOT carry that the
substrate needs?

- `.dag` ingestion: zero deficit. All structure explicit.
- Python ingestion: high deficit. Types, ownership, effects,
  complexity bounds are implicit or absent.
- REST API spec ingestion: medium deficit. Types present (JSON
  Schema), effects partially present (HTTP methods imply
  idempotency), but complexity bounds absent.

The deficit is a measurable property of the `GrammarSpec` +
`ElaborationSpec` pair. Ingestion cost = parsing cost (from
grammar) + reconstruction cost (from elaboration, proportional
to the deficit).

#### §2.3.2 Emission layering: spec vs rendering conventions

The emission pipeline has compositional layers with opacity
between them:

```
Layer 3: RenderingSpec (format, naming, idioms)  — declared conventions
          │ composes on
Layer 2: RepresentationMapping                   — substrate ↔ target
          │ reads
Layer 1: LanguageSpec (facts from spec doc)      — hard constraints
          │ applied to
Layer 0: Substrate (Bit, Conj, Disj, ...)        — what the compiler knows
```

**Layer 1: Language specification (facts).** What IS valid Rust?
Comes from the actual language's specification document. Modeled
in `extdeps/languages/rust/spec.dag`. Hard constraint — violating
Layer 1 produces invalid target code.

**Layer 2: Representation mapping.** How do substrate primitives
map to Layer 1's constructs? Conj → `struct`, Disj → `enum`,
Arrow → `fn`. The bridge between substrate and target.

**Layer 3: Rendering conventions.** Declared rendering rules
ON TOP of the spec. `rustfmt`-compatible formatting, `snake_case`
naming, line-length limits, `Result<T,E>` over `panic!`, import
ordering. These are **declared structural facts** with the same
authority as any other `.dag` declaration — not "preferences" or
"annotations." When present, they are hard constraints on the
emitter's output formatting. "Optional" means the emitter CAN
run without them (producing valid-but-unstyled output), NOT that
they are soft suggestions. Different projects compose different
rendering specs on the same language spec.

**Opacity between layers is load-bearing.** Layer 1 doesn't know
about Layer 3. Layer 3 doesn't know about Layer 0. When you
change a rendering convention, Layers 0-2 are untouched. When you
change a substrate type, Layer 3 is untouched.

```dag
// Layer 1 — facts from the target language's specification
type LanguageSpec {
  constructs: Map<ConnectiveKind, TargetConstruct>
  syntax: SyntaxRules
  type_system: TypeSystemSpec
  module_system: ModuleSpec
}

// Layer 3 — declared rendering conventions
type RenderingSpec {
  naming: NamingConventions
  formatting: FormatRules
  lint: List<LintRule>
  idioms: List<IdiomSpec>
}

// The composition — the emitter reads this
type EmissionTarget {
  language: LanguageSpec       // what's valid (required)
  rendering: RenderingSpec?    // how to format (optional)
}
```

When `rendering` is None: valid-but-unstyled output. When present:
fully-styled output per the declared conventions.

The same layering applies to ingestion: Layer 1 is the source
grammar (facts), Layer 3 is declared parsing conventions (error
recovery strategy, diagnostic phrasing, ambiguity resolution).
Opacity holds: the diagnostic phrasing doesn't know about the
grammar rules.

#### §2.3.3 Design challenges

1. **GrammarSpec expressiveness.** What grammar formalism? Pure
   CFG can't handle operator precedence or layout sensitivity. PEG
   has ordering sensitivity. EBNF + precedence declarations covers
   most practical languages. The formalism choice IS the L2.5
   parse domain decision.
2. **ElaborationSpec design.** How are source-language surface
   forms mapped to substrate declarations? This is the semantic
   bridge between "what the grammar produces" and "what the
   substrate carries." For `.dag` the mapping is mostly 1:1
   (surface items lower to declarations directly). For other
   languages the mapping is richer (Python class → Conj with
   method Arrows, C++ template → Instantiation). The elaboration
   spec must be expressive enough for arbitrary source languages
   without growing the compiler core.
3. **LanguageSpec completeness.** The spec must cover the full
   emission surface: type decoration, import systems, module
   organization, async models, memory management, concurrency,
   build system integration. If the emitter ever needs per-target
   code, the spec is incomplete.
4. **The bootstrap seed.** The compiler reads grammar specs to
   parse. Grammar specs are `.dag` files. To parse a grammar spec,
   the compiler needs a parser that reads a grammar spec.
   Chicken-and-egg. The seed parser parses at least the
   grammar-spec format without reading a spec — that's the
   irreducible kernel. How small can the seed parser be?
5. **Per-language spec authoring cost.** Writing a `LanguageSpec`
   + `ElaborationSpec` + `RenderingSpec` for a complex language
   (Rust, C++, Python) is weeks of work. One-time cost per
   language, not per project — but worth naming honestly. The
   thesis promise is "adding a new target costs one spec file."
   True; writing that spec set is the investment.

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

**This is a research problem, not an engineering problem.** Every
other pipeline stage migration (emit, lower, parse) is engineering
with known patterns. Inference-as-data — "type inference rules
expressed as `.dag` data read by a generic walker" — has no
production precedent. Theorem provers (Coq/Lean/Agda) get partway
there with tactic languages but all have hand-coded cores. v3's
bounded substrate (six connectives, five behaviors, decidable
iteration) gives more leverage than a general HM setting, but the
approach is still unproven. Budget this stage as open-ended R&D,
not scoped work, and front-load the experiment sequence
(`docs/inference-as-data-experiments.md`) to surface blockers
before committing to the full port.

**I0 result (2026-04-15):** the decidability paper exercise
passed. Two representative inference rules (literal type filling,
bounded Disj walk) are decidability-compatible under `.dag`'s
bounded-iteration invariant. No blocker found. This narrows the
open risk from "is this even possible?" to "which write-surface
option?" See `docs/inference-as-data-i0-result.md` for the full
check. The next experiments (I1-I8) run sequentially after
reflection lands.

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
  `infer.dag` until it's answered. I0's pass confirms the rules
  themselves are decidable; what remains is the Dag → Dag write
  surface.

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

## §10. Schema migration as a structural operation

v2's bootstrap pain reveals three failure modes that v3 must
NOT repeat:

1. **Fixed-point failures report at the wrong layer.** When
   `regenerate-stage0.sh`'s pass1 output differs from pass2,
   the error is "fixed-point failed" with a file-level diff.
   Localizing which pipeline stage diverged requires hours of
   hand-bisection. ROADMAP's M5 Phase 4 Level 1 names the fix
   (per-stage structural diffs during regen); it was never
   shipped.
2. **Manual stage0 edits come from TwoPhase schema changes,
   not drift.** A schema change that isn't backward-compatible
   (new required field, removed field, rename) forces a manual
   Rust patch to bridge one bootstrap cycle. v2 has a model
   for this (`dsl/gunbc/bootstrap.dag`'s `BootstrapStrategy`,
   `ChangeClassification`, `TransformContract`) but **the model
   has zero consumers**. 195 lines of decorative modeling; the
   regen flow doesn't read any of it.
3. **`dsl/gunbc/bootstrap.dag` is decorative.** The types
   `CompilerStage`, `TransformContract`, `ChangeClassification`,
   `BootstrapStrategy`, `FixedPointCheck`, `FieldPropagation`
   exist but no code consumes them. v2 shipped with the
   blueprint for fixing its own self-hosting pain, not wired.
   This is the Track 17 "decorative types" failure mode
   `ROADMAP.md` keeps naming.

**v3's structural answer: make schema migration a compile-time
operation over the substrate's own declarations.** The schema
diff is a lens. The patch is a data structure. The bridge build
is a mechanical application of the patch. Zero manual edits.

**Why this is possible in gunbc specifically.** gunbc emits
Rust **text**, not binary blobs. In GCC-style bootstrap, each
stage's output is an executable; you can compare bit-for-bit
and run it, but you can't structurally transform it. In gunbc,
every stage's output is source code — which means a schema
migration becomes a **patch over that source code**. The old
compiler doesn't need to understand the new schema; it emits
its own Rust, and a separate transformation applies the delta
the new schema implies. The patch applier operates on source
text, not on binaries.

### §10.1 The five-step schema migration pipeline

A schema change migrates without manual edits via this sequence:

1. **Structural diff.** A `.dag` lens walks old and new
   `substrate.dag` declarations, compares them, and produces a
   `ChangeClassification` — a list of `ChangeKind` entries each
   describing one delta. Built over reflection (§3 of the
   reflection design doc); the diff reads declarations as data
   because reflection makes them data.
2. **Patch generation.** A second `.dag` lens reads the
   `ChangeClassification` and emits a structural patch over
   the compiler's Rust source. Each `ChangeKind` has a patch
   template; the generator composes templates.
3. **Bridge build.** The committed stage0 compiles the OLD
   `.dag` source → reproduces the committed Rust state. The
   patch from step 2 applies to that Rust → patched Rust →
   rebuild → **bridge binary that understands the new schema**.
4. **Bridge compile.** The bridge binary compiles the NEW
   `.dag` source → stage1 Rust. (This is where the new schema
   is first seen by a compiler that knows about it.)
5. **Fixed-point verification.** Stage1 compiles the NEW `.dag`
   source → stage2 Rust. Assert `stage1 == stage2` by
   construction. If the patch was correct, convergence holds
   in one step.

Every step is either structural (walks substrate as data) or
mechanical (applies a patch). Zero manual intervention. Every
decision is either "what does the diff lens report" or "what
does the patch template expand to" — both are data.

### §10.2 `ChangeKind` enumeration

The schema-diff lens produces one of these per delta. Phase 1
targets the first five (~80% of real changes in v2 history);
Phase 2 subsumes the rest as L3 completes.

| ChangeKind | Example | Patch shape | Phase |
|---|---|---|---|
| **`AddField { decl, name, type, default }`** | Adding `result_port: PortId` to `BindNode` | Walk every `BindNode { … }` literal in Rust source, insert `result_port: <default>` in the initializer. Add field to the struct def. | 1 |
| **`AddVariant { enum, name, payload_type }`** | Adding a new `Behavior::NewThing(NewThingNode)` variant | Add variant to the enum def. For every `match behavior` in Rust source that's not `#[non_exhaustive]`, add a `Behavior::NewThing(_) => unimplemented!("scaffolded")` arm with a ratchet comment. New emitter/infer paths for the variant land via `rust.dag` and `infer.dag` declarations — NOT as hand-written Rust. | 1 |
| **`RenameField { decl, old, new }`** | `BindNode.value` → `BindNode.result_port` | Mechanical find-and-replace on the field name, scoped to the containing struct. Handles field-access sites and struct-literal sites. | 1 |
| **`RenameType { old, new }`** | `Declaration` → `DeclarationRef` (the v3_l1 sentinel rename warm-elk just landed) | Find-and-replace on the type name. Needs to handle both `type Declaration` in `.dag` and all Rust-side consumers. Scoped to not touch unrelated identifiers with the same string. | 1 |
| **`RemoveField { decl, name }`** | Deleting a deprecated field | Audit every reader of the field. If the field has become derivable from other fields, route readers through the derivation. If not, fail-closed — manual review required. Phase 1 tolerates "fail with diagnostic listing every reader" as acceptable output. | 1 |
| **`ChangeFieldType { decl, name, old_type, new_type }`** | `String` → `DeclarationId` (exactly what happened in PR #445's name-to-id migration) | Requires a **value converter** — a `.dag` function that takes `old_type` and returns `new_type`. For simple cases (string → structured ID via lookup) the converter is declarable. For complex cases the converter is a follow-up PR. | 1 for simple converters, 2 for complex |
| **`NewInferenceRule`** | Adding a new inference rule for a new substrate variant | Not a patch — it's additive `.dag` source the old compiler doesn't understand semantically but CAN parse. Falls out of Prereq 0 (template instantiation) + L3 Stage 3 (infer as `.dag`). | 2 |
| **`NewEmitterPath`** | Adding emit support for a new variant | Not a patch — it's a new `rust.dag` declaration. The emitter reads `rust.dag` as data. Falls out of L3 Stage 1 (emit as `.dag`). | 2 |
| **`Arbitrary`** | Anything not classifiable above | Fail-closed, escape-hatch "manual stage0 edit required." Tracked by a numeric ratchet; each use reduces the ratchet budget. | 1 escape hatch |

**Phase 1 covers the common cases.** AddField, AddVariant,
RenameField, RenameType, RemoveField, and simple
ChangeFieldType. In v2's history, these account for the vast
majority of schema changes. The escape hatch (`Arbitrary`)
handles the remaining edge cases with an explicit manual-edit
receipt — not silently, visibly.

**Phase 2 is not a separate design project.** When L3 (pipeline
stages in `.dag`) completes, the compiler's own behavior is
expressed as declarations — new inference rules, new emitter
paths, new lowering paths are all data edits. At that point,
every schema change classifies as one of the Phase 1 patches
over the substrate declarations, and the "new semantics"
cases become declaration additions in the companion files
(`rust.dag`, `infer.dag`, etc.). The Phase 2 patch language
is the declaration language; no separate Phase 2 effort.

### §10.3 Patch language technology — Option C upfront

Three ways to build the patch layer in Phase 1:

- **Option A — syn-based Rust transformer.** Build the patch
  generator in Rust using `syn`/`quote` to parse old Rust,
  structurally transform, emit new Rust. Rust-native, leverages
  existing tooling. Fastest path to a working Phase 1. **But**
  creates a Rust-side scaffold that has to retire when L3
  completes — and retirement friction tends to delay retirement.
- **Option B — string-level search-and-replace.** Simpler but
  brittle, silent failures on edge cases. **Rejected.**
- **Option C — native `.dag` patch language, compiled into
  Rust-source transformations at emit time.** The patch
  generator is a `.dag` lens that reads `ChangeClassification`
  declarations and produces patch declarations. The patch
  applier is another `.dag` program that walks those
  declarations and emits transformed Rust source via the same
  `emit_rust` pipeline everything else uses. No Rust-side
  scaffold; the patch language is `.dag` from the start.

**Decision: Option C upfront.** Rationale:

1. **Front-loads the decisions.** Going C upfront means the
   schema-migration work directly exercises reflection's second
   consumer (the schema-diff lens) and forces any substrate
   extensions the lens surfaces to land as prerequisites.
   Option A would defer those decisions behind a Rust scaffold
   that later has to retire.
2. **Phase 2 falls out of Phase 1 without a redesign.** In
   Option C, the patch language IS `.dag`. As L3 stages
   migrate, the emitter-side components of the patch language
   come into `.dag` alongside the rest of the emitter. The
   patch language doesn't shrink or dissolve; it grows through
   the same migrations everything else does.
3. **The patch language is the first real user of reflection
   beyond lenses.** Building it validates that reflection works
   for consumers that aren't toy analyses. That's the
   strongest possible test of the framework.
4. **Same discipline as lenses-as-`.dag`-native.** If the
   project's commitment is "lenses are `.dag` programs, not
   Rust modules," then patches should be `.dag` programs too.
   Different abstraction, same rule.

**The cost of Option C is that Phase 1 takes longer to ship
than Option A would.** That cost is absorbed by the explicit
commitment: schema migration is a follow-up PR AFTER reflection
lands, not bundled with it. We're already paying for the wait
by staging the work; we might as well pay it for the purer
design.

### §10.4 Phase 1 as follow-up PR

Phase 1 schema migration is its own PR, NOT part of the
reflection PR. Reasons:

- The reflection PR is already carrying substantial scope
  (substrate declarations, prereq slate, `lens_unused_parameters`
  migration). Bundling schema migration doubles that scope.
- Phase 1 needs reflection + `std/string.dag` + potentially
  `std/ast/rust.dag` (declaring Rust's AST as a substrate type
  for the patch emitter to walk). These are substantial
  prerequisites that aren't needed for the reflection PR's
  core.
- Separating the PRs gives clean review focus. Reflection's
  reviewers don't have to reason about schema migration; schema
  migration's reviewers don't have to re-litigate reflection.

**Acceptance criteria for the Phase 1 PR:**

- [ ] `dsl/lenses/schema_diff.dag` exists, walks two Dag values
      from two substrate versions, and produces a
      `ChangeClassification`
- [ ] `dsl/lenses/schema_patch.dag` exists, reads a
      `ChangeClassification` and produces a structural patch
      declaration
- [ ] Patch applier emits transformed Rust source for AddField,
      AddVariant, RenameField, RenameType, RemoveField, and
      simple ChangeFieldType
- [ ] `Arbitrary` escape hatch exists with a numeric ratchet
      (starts at some count representing "manual edits allowed,"
      monotonically decreasing toward zero)
- [ ] At least one real v2 → v3 schema migration history
      incident is reproduced via the pipeline, with byte-identical
      output to whatever v2 produced historically
- [ ] `dsl/gunbc/bootstrap.dag`'s decorative types (`CompilerStage`,
      `TransformContract`, etc.) are either wired to real
      consumers or deleted as dead code. The decorative-modeling
      debt from v2 is retired.

**Dependency on reflection:** Phase 1 cannot start until
reflection ships (it needs the substrate-as-data for the
schema-diff lens). Phase 1 CAN proceed in parallel with L2
consumer migrations once reflection lands — they're independent
work streams.

---

## §11. Per-stage fixed-point from day one

**Problem.** v2's regen fixed-point check fires at a file-level
diff. When pass1 != pass2, the output is "some file differs"
with no indication of which compiler stage produced the
divergence. The 2024 dark-emu-36-pr3 incident spent hours
localizing the divergence to the Resolve stage by hand.

**Fix (v3 should build from day one).** The regen loop captures
per-stage output and compares pass1 and pass2 at every stage
boundary. When divergence is detected, the error names the
first stage that diverged, emits a per-stage structural diff,
and points at the specific declaration or behavior that
differs.

**Implementation shape.** A `.dag` lens walks two Dag values
(pass1 and pass2 outputs) and produces a structural diff. This
is **the same lens** used by the schema-migration work in §10
— `dsl/lenses/dag_diff.dag` or similar. The fixed-point check
in regen reuses it with different inputs: compare pass1 pre-
and-post each pipeline stage.

**Why from day one.** v3's regen flow is simpler than v2's
right now because there's less accreted tooling. Adding per-
stage diffing at first regen is cheap; retrofitting it after
pain is expensive (exactly what v2 is discovering).

**Acceptance criteria:**

- [ ] `regen.sh` (or equivalent) captures per-stage Dag snapshots
      during pass1 and pass2
- [ ] Fixed-point check runs `dag_diff.dag` at each stage
      boundary
- [ ] Divergence reports the first stage that diverged and
      the structural delta
- [ ] Tested against a synthetic divergence (intentionally break
      one stage, verify the error correctly identifies it)

**This section graduates to L1 (part of the reflection PR or
immediate follow-up) because it uses the same schema-diff lens
that §10 builds.** The lens has two consumers: schema migration
(comparing substrate versions) and per-stage fixed-point
(comparing compile passes). Building it once for both is
cheaper than building it twice.

---

## §12. Hand-maintained file inventory at M3 start

**Problem.** v2 has two hand-maintained Rust files in stage0
(`cli_run.rs` for the Run handler, `v2_interpreter.rs` for the
interpreter) that are excluded from the fixed-point diff.
Those two files aren't documented as "permanently hand-
maintained" anywhere authoritative; they're discovered by
people hitting the exclusion list. Neither has a named plan
for eventual dissolution.

**Fix.** v3's M3 plan declares **explicitly** which files are
permanently hand-maintained, with the chicken-and-egg for each.
Declaration not discovery.

**The invariant:** a file in the hand-maintained set MUST have
a receipt that names (a) the specific chicken-and-egg dependency
that makes it hand-maintained, (b) the dissolution trigger that
would retire it, and (c) whether retirement is realistic
(examples: "needs the interpreter in `.dag`" — likely retirable;
"needs a Rust runtime binding for stdin/stdout" — probably
permanent).

**v3's hand-maintained set at M3 start — TO BE FILLED IN during
M3 planning.** Placeholder entries, each with a planned
receipt:

- **`src/v3/compiler/src/bootstrap.rs`** — the file that
  registers the V3_SPECS list and composes the bootstrap Dag.
  Hand-maintained because it's the seed that compiles the
  `.dag` pipeline; it has to exist in Rust before any `.dag`
  can run. **Dissolution:** when the `.dag` pipeline is fully
  self-hosting, `bootstrap.rs` shrinks to "read the files, call
  parse.dag's compiled function" — ~20 lines of glue. Probably
  never zero, but bounded.
- **A minimal Rust tokenizer** — probably needed to bootstrap
  `parse.dag`. The tokenizer is the innermost layer and is
  likely the last thing to migrate. **Dissolution:** when
  `tokenize.dag` (if it lands) can produce the same token
  sequence. Maybe never; tokenizers are string-heavy and the
  cost of a `.dag` version vs a tiny Rust version may not be
  worth it.
- **TBD** as M3 scope crystallizes. Each entry follows the
  receipt template above.

**The rule: the set is declared, not accreted.** Every PR that
adds a file to the hand-maintained set MUST update this section
with the receipt. No silent additions. If the set is larger
than 3 files by M3 start, that's a signal the bootstrap is too
ambitious and scope needs revisiting.

**This section updates as M3 planning begins.** Today it's a
placeholder; it becomes load-bearing when the M3 milestone is
active.

---

## §13. Perf audit pattern

**Lesson from v2.** v2's self-compile perf cliff was never
clones. The actual cost was **fact re-derivation** — functions
named `merge_envs`, `combine_*`, `build_*_from_*` that walked
upstream data and reconstructed facts the upstream already had.
The 2024 dark-emu fix was 6 lines: stop rebuilding the
`InternTable` in merge_envs, thread the existing one through.
68× speedup on reconcile.

**The audit pattern.** A grep for `merge_*`, `combine_*`,
`build_*_from_*` (and similar "take things apart and put them
back together" verbs) finds the candidate sites. For each hit,
ask: **is this function computing a fact that an upstream
stage already had?** If yes, it's a re-derivation. The fix is
to thread the upstream fact through instead of recomputing.

**This is `feedback_perf_is_dependency_modeling` applied
concretely.** The memory entry already says "re-derivation =
poor dependency modeling." §13 gives the audit tool: grep for
verbs of reconstruction, inspect each, thread the upstream
fact where found.

**v3 perf work should start with this audit.** Every
performance concern on v3's self-compile loop goes through the
re-derivation filter first. Clone-counting comes second, if
ever. The thesis-aligned claim is: **if v3 has perf problems,
they'll be in the places where the compiler is recomputing
substrate facts that should be carried forward as typed edges.**

**Cross-reference with consumer migrations.** Every v2 `annotate_*`
helper that becomes a substrate field read in the complexity
lens migration (see `docs/substrate-reflection-design.md` §12.5
M1) is also a perf win by the same mechanism: the facts stop
being reconstructed and start being read.

---

## §14. When this doc updates

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
