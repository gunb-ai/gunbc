# v4 Compiler Architecture — Homomorphism-First Design

> **Status: DRAFT, under discussion.** Forum for review and amendment — comment, push back, propose alternatives. Decisions ratified here flow into implementation briefs.

## What this document is

A design reference for the **v4 compiler** in the `gunbc` project, written before implementation of the compiler's middle and back stages (the ones that turn parsed source into typed source, then into target output or evaluated values). It captures the architectural decisions we want to commit to before code is written, so implementation does not cement the wrong shape.

This doc supplements but does not replace the foundational + ratified design docs listed below. It exists to make one specific question concrete: **what is the compiler, mechanically, if we take "The derived homomorphism" claim in THESIS literally?**

Intended audience:
- Project insiders deciding T-9 / T-10 / T-22 implementation shape.
- Newcomers trying to understand what the compiler is supposed to do.
- Reviewers checking architectural commitments against THESIS, MODELING, and INVARIANTS.

You can read it cold without prior context — the next two sections give that context. If you already know the project, skim to "TL;DR".

## Load-bearing references

This doc composes with — and must stay consistent with — these existing artifacts. Treat any contradiction between this doc and a ratified one of these as a STOP signal.

**Foundational (every change to this doc reviewed against these):**
- [`THESIS.md`](../THESIS.md) — the project's goals + complete claim list. "The derived homomorphism" is the parent thesis this doc operationalizes.
- [`MODELING.md`](../MODELING.md) — modeling rules (M1–M10), especially M9 (DFS the concept DAG).
- [`docs/modeling-discipline.md`](modeling-discipline.md) — the ten practices (Practice 1 fail-closed, Practice 8 fact-bundle, Practice 9 no-prose, Practice 10 derived operations / no walker dissolution).
- [`INVARIANTS.md`](../INVARIANTS.md) — five governing principles (P1 Modeling Faithfulness, P2 Boundary Discipline, P3 Fail-Closed, P5 Progress-Is-Dissolution).
- [`src/v4/TASKS.md`](../src/v4/TASKS.md) — task graph + ratified-decision authority for T-1…T-30+. **T-9 line 418 is the source of the coercion-fold ratification (D2 reversal 2026-05-17).**

**Adjacent ratified design (this doc composes with — read for the EDIT direction and self-modification stories):**
- [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) — **the EDIT-direction companion to this doc.** Defines the agent-side read/edit surface as it existed before the 2026-05-21 orthogonality amendment (`apply_lens(lens, scope, mode)`, `apply_diff(root, Diff)`, candidate-state gating, library-first agent surface, Layer 0–6 build order). This doc now supersedes the lens/gate portions toward Node-in readings + caller policy; `apply_diff` and candidate-state remain authoritative.
- [`docs/design-emit-stage-l25-model.md`](design-emit-stage-l25-model.md) — emit-stage L2.5 model, the substrate-side specification for what this doc names as `translate` + `serialize` (T-10).
- [`docs/design-infer-stage-l25-model.md`](design-infer-stage-l25-model.md) — infer-stage L2.5 model, the substrate-side specification for what this doc names as `ground` (T-9).
- [`docs/design-lens-application-surface.md`](design-lens-application-surface.md) — the pre-amendment substrate-level lens-application surface. Its `Lens / SectionRef / Witness / Outcome` gate shape is inventoried below as a follow-up reshape toward graph-in readings.
- [`docs/design-lens-framework.md`](design-lens-framework.md) — the lens framework as a whole.
- [`docs/design-affected-set-lens.md`](design-affected-set-lens.md) — `affected_set` (T-21), the incremental-rebuild + lens-frontier specialization consumed by this doc's pipeline diagram and by caller policy over candidate-state readings.
- [`docs/design-dissolution-lens.md`](design-dissolution-lens.md) — the dissolution-pattern lens family (L1.x), Practice 10's auto-fix substrate. The (find, transform) convolution view in the read/edit doc's § 6 builds on this.
- [`docs/v4-close-interrogation.md`](v4-close-interrogation.md) — v4 program framing + scope.
- [`docs/v4-dag-rationale.md`](v4-dag-rationale.md) — rationale for `.dag` as the language.
- [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) — the hand-Rust-to-zero trajectory; informs the self-modification / stage0 story below.
- [`docs/design-substrate-lambda-calculus-grounding.md`](design-substrate-lambda-calculus-grounding.md) — substrate grounding semantics.
- [`docs/design-bootstrap-fact-model.md`](design-bootstrap-fact-model.md) — bootstrap fact model.

**Lens-specific design docs** (consult when discussing the lens family for a specific dimension):
- [`docs/design-complexity-lens-behavioral-completeness.md`](design-complexity-lens-behavioral-completeness.md)
- [`docs/design-complexity-tightness-lens.md`](design-complexity-tightness-lens.md)
- [`docs/design-cost-lens-sizevar-dimension-wiring.md`](design-cost-lens-sizevar-dimension-wiring.md)
- [`docs/design-timing-lens.md`](design-timing-lens.md)
- [`docs/design-effect-enumeration-resource-threading.md`](design-effect-enumeration-resource-threading.md)
- [`docs/l1.5-clean-bootstrap-design.md`](l1.5-clean-bootstrap-design.md)

This list is **not exhaustive** — see `docs/design-*.md` for the full set. The doc above names what's load-bearing for the architecture in *this* doc.

---

## Background (skip if you know the project)

### What gunbc is

`gunbc` is a compiler for a language called **`.dag`**. The thesis (one line): *a program is a dependency graph (a "DAG" of typed nodes); when the dependency graph is structurally correct, emission to any target language becomes mechanical translation.* See [`THESIS.md`](../THESIS.md) for the full claim set.

Concretely, the user writes a `.dag` source file — types, functions, services, types-with-effects. The compiler is supposed to:

1. Validate that the dependency graph is structurally coherent (type-checking, termination, no aliased mutation, no cross-target drift, …).
2. Either **emit** that same graph as source code in a target language (Rust, Python, Go, TypeScript, C++, …), or **evaluate** it directly on a concrete runtime.
3. Catch correctness violations that mainstream languages miss — complexity bounds, idempotency, cost, effect tracking — by reading the graph structurally rather than by testing.

The ambition (eventually) is **omni-emission**: from one `.dag` source declaring a system (backend + frontend + database schema + the network glue between them), produce source code for each component AND the inter-component marshaling, with type-fidelity by construction. None of that is in scope for *this* doc — but the compiler's architecture must not preclude it.

### The `.dag` substrate

Programs are trees of **`Node`** values (declared in `src/v4/std/node.dag`). A Node has a kind (one of six type connectives: `Atom`, `Conj`, `Disj`, `Arrow`, `Cardinality`, `Instantiation`) and edges to child Nodes. Computation is expressed via five behaviors (`Value`, `Transform`, `Branch`, `Loop`, `Bind`). Everything user-facing — `fn`, `type`, `service` — is surface sugar over this layer.

The substrate has a fundamental operation: **`fold_node`** (a catamorphism, i.e., generic structural recursion over the Node tree). Every "pass" the compiler runs — type checking, evaluation, code generation, lens analyses — is supposed to be `fold_node(tree, some_algebra)`. The "algebra" is the per-Node-kind step function. This is load-bearing for the architecture below: it means new passes are data (the algebra), not code (a new walker).

External-target languages and frameworks are themselves modeled as `.dag` files under `extdeps/languages/`, `extdeps/frameworks/`, `extdeps/formats/`. So Rust isn't "a thing the compiler knows" — it's `extdeps/languages/rust.dag`, a fact-bundle declaring Rust's grammar and primitive types, consumed by the compiler as data.

### What "homomorphism" means here

A homomorphism is a **structure-preserving map** between two algebraic structures. In our compiler context:

- **Source structure:** a `.dag` program is a Node tree with declared facts (every type asserts what it inhabits — e.g., `Int` inhabits `OrderedRing`).
- **Target structure:** Rust (or Python, or any target) is *also* declared as a Node tree of declared facts in `extdeps/languages/X.dag`. Rust's `i32` inhabits Rust's signed-integer algebra with a 32-bit width fact, etc.
- **The homomorphism:** the mapping from source Nodes to target Nodes that **preserves structure** — operations compose the same way after translation as before. If `compose(f, g)` exists in source, it must map to `compose(f', g')` in target where `f' = h(f)` and `g' = h(g)` and the composition law holds.

The thesis claim: **the compiler should *derive* this map** from the two language models, not hand-author it. A handwritten Rust emitter cements an N×M cost (every new feature × every target). A derived homomorphism is N+M (every new feature in the substrate; every new target as one new model).

This doc operationalizes that claim.

---

## TL;DR — the compiler we want

The v4 compiler is:

1. **One structural primitive** — `fold_node` (catamorphism over Node).
2. **Plus four substrate operations** that aren't `fold_node` but are needed:
   - A **diagnostic carrier** (`Diagnostic + Locus`, fail-closed reporting).
   - **Grammar-as-bidirectional-data** — one declarative grammar model that drives both parsing and serialization. Default law: `parse_target(serialize_target(node)) == node`.
   - **`find_witness`** — the unified **candidate-enumeration-and-check** primitive (rescoped 2026-05-20 — Pass B unification). Given `(source_facts, closed_candidate_set, preservation_predicate, multiplicity_policy)`, mechanically enumerates the **entire** closed candidate set and checks the predicate against **every** candidate; **collects ALL passing candidates** before resolving — exactly-one ⇒ Accepted; zero ⇒ Rejected(NoCandidate); multiple ⇒ apply the **caller-supplied** `multiplicity_policy`. **The primitive does NOT bake in any specific selection policy** — `MultiplicityPolicy` is a typed parameter the caller supplies per use site (canonical source grounding ⇒ `UniqueOnly`; target coercion ⇒ the Q14 `TargetSelectionPolicy` declared by the target language model; lawful rewrites ⇒ rewrite-domain policy). This keeps target-coercion-specific selection (Q14) out of source-grounding and other non-target uses, preventing a target-model policy from silently accepting an ambiguous canonical source grounding. **NO positional-first-wins** — ambiguity detection is required. Decidable by construction over the closed candidate set — **NOT a search**, per `src/v4/TASKS.md` T-9 ratification ("not a search, not research"). The word "find" denotes the operation's *output* (the canonical candidate or a diagnostic), not a search algorithm. **Unifies what were two distinct primitives** (`solve_constraints` and the coercion fold): both are enumeration-and-check over closed candidate sets with different predicates AND different multiplicity policies. `solve_constraints` enumerates canonical-form candidates with a constraint-satisfaction predicate AND `MultiplicityPolicy::UniqueOnly`; the coercion fold enumerates target-language declared inhabitants with an exact-structural-equality-zip-fold predicate AND `MultiplicityPolicy::TargetSelection(target_lang.target_selection_policy)`. The mechanic is the same — closed candidate enumeration + mechanical predicate check + caller-supplied multiplicity resolution — only the predicate, the candidate-source, and the multiplicity policy differ. See "Primitive unification" below.
   - **Typed dependency graph** — per P6, **the substrate's usage graph IS the dependency graph**. Imports, field references, function calls, and Edge-on-`Node` already encode usage relationships; `std/dependency.dag` owns the `DependencyKind` classification + a **dependency lens** fold that projects emergent typed relationships from substrate usage. NOT a parallel authored data structure with `DependencyEdge` / `DependencyReason` carriers (per operator-direct sharpening 2026-05-20: "all the designs in this repo must be dependency managed — i.e. not rederiving other structures, just projecting/viewing them"). Edge orientation `A → B` = "A required before B." SCC condensation + topological-order via fold over the substrate, NOT a separate primitive operating on parallel-authored data.

**Things that were primitives in earlier drafts but are now derived** (Pass B unification):
- **`traverse_node` / `sequence_node` / `bind_outcome`** are `fold_node` invocations with an Outcome-threading algebra; the combinator (not the algebra) interprets `StageDiagnosticPolicy` to thread failures. The algebra is pure-success-accumulator: `fn(R, Edge, R) -> Outcome<R>` (matches the landed `NodeFold<R>.step` shape — third parameter is the already-folded child result `R`, NOT the raw child `Node`; the Outcome wrapping is the only difference from the pure-catamorphism primitive). Single authority for failure handling = combinator. See "Derived combinators" below.
- **`apply_diff`** is `sequence_outcome(diff.edits, fn(edit, candidate) -> fold_node(candidate, substitute_at(edit.at, edit.replacement)))` — sequential Outcome-bind-fold over the Diff's ordered Edits list, threading the candidate root through, with each Edit applied to the *current intermediate state* via `fold_node` substitution. **Sequential semantics matter**: each later Edit's Path resolves against the post-prior-Edit state, NOT the original root; if any Path fails to resolve in the current candidate, the whole Diff fails-closed (P3, per read/edit pipeline doc requirement). The agent-side mutation surface; composes with `fold_node` + `sequence_outcome`, not parallel to them.
- **`LawfulRewriteWitness`** (P7, when implemented) is `find_witness` with an algebra-preserving precondition law as the `preservation_predicate`. Different lawful rewrites = different predicates over `find_witness`, not a separate primitive.
3. **Compile-core has no boundary actions and no global terminal mode.** Graph consumers are **pure data-in / data-out** functions: `ground(ingestion_plan)` produces the grounded graph, `observe(graph, lens_plan)` reads it, `project(graph, projection_plan)` produces artifacts, and `authorize(policy, lens_report, projection_report)` decides terminal release. No implicit text-reading, no implicit text-writing, no implicit "execute," and no public `CompileMode` / `ProductionGoal` enum that chooses among unrelated operations. All real-world I/O — reading source from disk, writing target source to disk, executing on a host, reporting diagnostics to stderr — is **modeled as effects against modeled resources** (`extdeps/file_system.dag` for files, `extdeps/posix.dag`, `extdeps/network.dag`, etc.), composed by *peripheral shims* outside the compile-core surface. The architecture itself doesn't think about text/files/processes; those are orthogonal concerns modeled independently.

Everything else — every "pass," every "stage," every "language-specific" anything — is an **algebra** plugged into `fold_node` or another declared substrate primitive. The compiler knows no specific language code (Rust, Python, even `.dag` itself, beyond an irreducible bootstrap-seed for `dag.dag`); language-specific *data* lives in `LanguageModel` declarations under `extdeps/languages/`.

**Algebras can be higher-order, effectful, constraint-bearing, or fixpoint-seeking (P5).** "Fold" doesn't mean "pure bottom-up." Inherited context, effects, constraints, and fixed-point analyses are first-class — carried by the algebra's signature, never as ad-hoc walkers.

**Dependency management is inherent to the substrate but only under enforced discipline (P6).** Synthesized (child→parent) facts ride `fold_node`'s natural order. Inherited (parent→child) facts ride higher-order carriers (P5). Memoization and incrementality ride content-addressed Node identity. The inherent property breaks the moment any pass walks Nodes outside the declared primitives — Practice 10 enforces this.

The doc that follows defines the orthogonal operation surfaces, why each piece is shaped that way, what's already in the codebase, and what's still open for design.

---

## End-to-end I/O — orthogonal operations

> **Text/data separation + I/O via modeled effects (ratified 2026-05-20, operator-direct).** Compilation operates on **data** (`CoreNode`), not text. Text, files, and shell/OS are **orthogonal concepts modeled independently** in `extdeps/` — there's a small shim to ingest text into `CoreNode`, but the compiler core never sees text or files. **The architecturally pure approach:** all real-world I/O is a modeled effect against a modeled resource (file_system.dag, posix.dag, network.dag, …), composed by peripheral shims outside the compile-core surface.

> **Orthogonality amendment (2026-05-21, operator-direct).** Compilation, lens analysis, artifact projection, and terminal authorization are separate graph consumers. They may consume the same grounded graph and may be orchestrated by one build/session layer, but they are not sub-modes of one another. Do **not** add one universal `SubgraphScope` carrier. The public session shape is a four-phase split: `ground`, `observe`, `project`, `authorize`.

```
// === Four-phase session split — data-in / data-out, no I/O ===
ground(plan: IngestionPlan) -> Outcome<InferredTree>
observe(inferred: InferredTree, plan: LensPlan) -> LensReport
project(inferred: InferredTree, plan: ProjectionPlan) -> ProjectionReport
authorize(policy: TerminalPolicy, lenses: LensReport, projections: ProjectionReport)
  -> Outcome<Validated<ArtifactSet>>
```

Each phase defines its own graph-consumer interface. The caller composes whichever operations it needs through a session layer. There is no closed `CompileMode` / `ProductionGoal` enum bundling unrelated operations into one compile-owned authority. Input language selection belongs inside an `IngestionPlan`, parallel to output selection inside `ProjectionPlan`; it is not a special scalar on the public session surface.

### Peripheral shims for text and file I/O

If a user wants to compile a `.dag` source file from disk to a Rust source file on disk, they compose modeled effects with compile-core. Each step is its own substrate-modeled operation:

```
// Each line is a modeled effect / pure transform — NOT a compile-core boundary action.

source_text   = file_read(path)              // effect against extdeps/file_system.dag
core_result   = ingest_text(source_text,     // shim — composes parse + normalize
                            input_lang) -> Outcome<CoreNode>
if core_result is Rejected:
  reject with diagnostics

core_node     = unwrap(core_result)
grounding     = ground(IngestionPlan {
                 subject: core_node,
                 producer: ingest_explicit_language,
                 params: dag_language_params
               }) -> Outcome<InferredTree>
if grounding is Rejected:
  reject with diagnostics

grounded      = unwrap(grounding)
readings      = observe(grounded, lens_plan)
artifacts     = project(grounded, rust_projection_plan)
release       = authorize(policy, readings, artifacts)
if release is Rejected:
  reject with diagnostics

validated     = unwrap(release)              // Validated<ArtifactSet>
target_text   = unwrap(validated.artifacts[rust_source_artifact])
file_write(out_path, target_text)            // effect against extdeps/file_system.dag
```

- **`file_read` / `file_write`** are modeled effects in `extdeps/file_system.dag` — operations against the modeled file-system resource (carrying `ResourceDependsOn` edges per P6, fail-closed-or-explicit-modeled-default per P3).
- **`ingest_text`** is a peripheral shim — NOT a substrate primitive. Composes parse (using input_lang's grammar) + normalize (using input_lang's sugar dissolutions). The shim exists because text-on-disk is a common user-facing input format, but the compile architecture itself stops at CoreNode.
- **`ground` / `observe` / `project` / `authorize`** are pure session phases. No file handle. No text. No process. `project` may use a translation producer internally, but the file-to-file shim writes only an authorized artifact from the session result.

**Other ingest paths** (no text involved at all):
- Programmatic: a client builds a `CoreNode` via substrate constructors (IDE plugins, code generators).
- Query-driven: `affected_set + apply_diff` produces a new `CoreNode` from an existing one (read/edit pipeline).
- Round-trip: a `TargetNodeTree` from a prior translate is parsed back via the grammar's inverse direction.

All paths produce `CoreNode`; Node-in operations take it from there. **Translate, eval, lenses, and artifact projections don't know how the CoreNode was produced.**

### Why this matters architecturally

P0 + P8 (consequence recomputation + the compiler obeys its own discipline) plus the operator's I/O-as-modeled-effect rule together imply: **there are NO hidden side effects anywhere in the architecture**. Every read, every write, every execute is a modeled effect against a modeled resource, carrying typed dependency edges (P6) and visible to lenses.

This is what makes dry-run work as composition (lens identifies I/O → LawfulRewriteWitness substitutes → eval against a mock runtime). It's also what makes incremental rebuild work (every artifact's provenance is modeled; affected_set walks the typed effect dependencies). It's what makes self-modification safe (the compiler editing its own source goes through modeled `apply_diff`, not implicit file writes).

**The slogan.** *No implicit I/O. Files are not the architecture. Effects are modeled, not assumed.*

### Why text/data are separated

There are multiple legitimate entry points to a `CoreNode` graph:

| Entry point | How `CoreNode` is produced |
|---|---|
| **Text ingest** | `ingest_text(text, input_lang)` — text → parse → normalize → `CoreNode`. The standard `.dag` source-file path. |
| **Programmatic / API** | A client builds the `CoreNode` directly via substrate constructors. IDE plugins, code generators, build systems. |
| **Query-driven rewrite** | `affected_set` + a node-graph transformation produces a new `CoreNode` from an existing one. Future query languages (e.g., write-via-query against an existing program graph) operate here. |
| **Round-trip from translate** | A `TargetNodeTree` from a prior translate can be parsed back via the grammar's inverse direction and ingested as `CoreNode` (for cross-language refactoring / round-trip workflows). |

All paths produce `CoreNode`; Node-in operations take it from there. This matters because:
- The lens / affected-set / query story (much of the v4 wishlist) operates on Node data, not text. Forcing every entry through text would either require lossless code-mod tooling or constrain the substrate's expressiveness.
- The compile pipeline stays pure data-in / data-out. Text I/O is at the ingest/serialize boundaries only.
- The substrate model is uniform: a `CoreNode` is a `CoreNode` regardless of how it was authored. The compile contract doesn't care which entry point produced it.

### Carrier types in this signature

**`CoreNode`** is the substrate-canonical Node — six connectives + five behaviors only, sugar dissolved. This is what `ground(IngestionPlan)` consumes. Lens and projection operations consume the grounded graph (`InferredTree` / future `InferredGraph`) that `ground` returns, not the raw `CoreNode`. (See "Carrier taxonomy" below — `CoreNode` is the post-normalize shape, distinct from `SurfaceNode` and `TargetSurfaceNode`.)

**`LanguageModel`** is a `.dag` model in `extdeps/languages/X.dag` — a fact-bundle declaring X's grammar (bidirectional), primitive types with their facts (width, signedness, range, …), algebra inhabitance (this type inhabits this algebra), sugar dissolutions, binding/scope rules, and version metadata. Resolve/ground/translate consult it for scope rules, inhabitance declarations, and target realization; `ingest_text` consults it for grammar + sugar.

**Runtime evaluation** uses decomposed abstract runtime carriers in `std/runtime.dag` plus concrete runtime fact-bundles in `extdeps/runtimes/*.dag`. Both language models and concrete runtime bundles consume the shared `ModelCore` (primitive types, algebra inhabitance, laws, effect / partiality semantics). Runtime carriers declare concerns that have no emit-side analog: runtime value representation, primitive operation interpretation, execution semantics, resource / effect boundary. See "Carrier shapes — `ModelCore` / `LanguageModel` / runtime carriers."

Each operation has its own output carrier: `InferredTree` for `ground`, `LensReport` for `observe`, `ProjectionReport` for `project`, and `Validated<ArtifactSet>` for terminal `authorize`. Target source and host results are artifact payloads inside projection reports and authorized artifact sets, not direct file-write authority.

> **Note on Lens mode:** the original draft of this doc proposed a third `CompileMode` for running lenses (the analysis framework for complexity / cost / ownership / effects / user-defined dimensions). That remains rejected. **Resolved 2026-05-21 — see P3 below.** Lenses are pure data-producing reads over the grounded graph; enforcement is terminal policy over those readings, not a compile-core mode and not a mechanical compile error.

---

## Architectural premises (ratified — these inform implementation briefs)

### P0 — The compiler exists to make change consequences computable

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

**The problem this compiler solves is not text-to-text code generation.** It's that in ordinary codebases, interface / integration / effect / resource / protocol dependencies are scattered, implicit, and manually synchronized. A small upstream change can break distant downstream behavior without a structural explanation; developers chase fallout via runtime failures, broken tests, and grep.

The v4 architecture's answer:

1. **Model the system as a typed dependency graph.** Interfaces, integrations, protocols, resources, effects, and transformations are first-class typed nodes/edges (P6), not implicit in arbitrary code.
2. **Ground it canonically.** `InferredTree` is the canonical post-grounding graph; ambiguity is a diagnostic, never a downstream surprise (canonical-grounding invariant).
3. **Code, tests, glue, schemas, runtime plans, diagnostics — all are projections of the grounded graph.** Not the source of truth. When a modeled fact changes, the compiler computes the affected dependency frontier (T-21 `affected_set`) and **recomputes** all downstream projections whose inputs changed.
4. **If a projection can no longer be derived** by homomorphism or a declared `LawfulRewriteWitness`, compile fails-closed with a diagnostic.

**Source-of-truth inversion.** This compiler inverts the traditional relationship:

| Traditional codebase | This architecture |
|---|---|
| Code is source of truth | Model is source of truth |
| Interfaces are scattered | Interfaces are typed Nodes |
| Dependencies are implicit | Dependencies are typed edges (P6) |
| Tests detect drift after the fact | Tests are projections; drift is structural |
| Build failures reveal dependencies | Compile-time analysis enumerates affected sites |
| Manually patched cascade after a change | Compiler computes the affected Node frontier + regenerates projections; non-derivable consequences surface as diagnostics |

**Closed-world questions become decidable.** The compiler does NOT claim "all programming becomes decidable." It claims a **specific bounded set of system-level questions** becomes decidable *because the input structure is constrained*: the coercion fold is decidable-by-construction over a closed candidate set; parallelism is provable when dependency/effect/resource facts are complete; interface fidelity is structural because both sides derive from the same Node. See P5 (constraint solving), P6 (dependency taxonomy), P7 (lawful rewrites) for the mechanics; the resulting decidability is bounded and named, not universal.

**The slogan.** *Code is not manually synchronized; consequences are recomputed.*

### P1 — Languages are I/O integration surfaces, not compiler concerns

Both source and target languages are `LanguageModel` data passed in as parameters. The same `rust.dag` model is consumed when emitting Rust AND when parsing Rust as input (bidirectional grammar-as-data). There is no per-target emitter; `emit_rust` does not exist as compiler code. The Rust knowledge lives entirely in `extdeps/languages/rust.dag`.

This is a direct consequence of THESIS's "The derived homomorphism" claim plus [`docs/modeling-discipline.md`](modeling-discipline.md) Practice 10 Row 3, which classifies a hand-written cross-target emitter as "you re-wrote the compiler" — a whole-architecture failure, not a localized smell.

### P2 — Eval is translate-to-runtime

Evaluation is structurally the same operation as translation, with the concrete runtime as the target. The compiler-side machinery (parse, normalize, resolve, ground, then the coercion fold) is shared. Eval differs from translate **only** in the terminal action: execute the resulting structure via the runtime interpretation algebra (eval) vs. serialize it via the target's grammar (translate).

Runtime shape is decomposed per the 2026-05-21 option-C supersession: abstract runtime carriers in `std/runtime.dag`, concrete runtime bundles in `extdeps/runtimes/*.dag`, both over shared `ModelCore`.

### P3 — Lenses are pure readings over graphs

> **Amended 2026-05-21** (operator-direct). The 2026-05-20 `validate_then_compile` framing was too compile-owned: it made lenses look like inputs to compile-core. The corrected shape keeps compile-core narrow and moves orchestration to a session layer: ground, observe, project, authorize.

Lenses are **data-producing reads**. A lens takes a graph and returns a typed reading for that graph's shape: `complexity_lens(function_graph) -> ComplexityReading`, `complexity_lens(whole_program_graph) -> ComplexityReading`, `complexity_lens(single_node_graph) -> ComplexityReading`. Same interface, different graph extent. Do not introduce one generic `SubgraphScope` carrier to express "part of a graph."

**Lens output is not a compile error.** A compile error says "this operation could not be performed" — parse failed, name resolution failed, type grounding was ambiguous, target translation could not be derived. A lens reading says "this is what the lens observed." Terminal policy may decide that a reading blocks release, but that decision belongs in `authorize`, not in the lens and not in compile-core.

**Six commitments:**
1. **The grounded graph is the lens consumption point** — no separate whole-graph vs subgraph carrier. Current v4 names this `InferredTree`; see the Graph naming note below.
2. **Lenses are folds over the graph** sharing the `fold_node` primitive (Practice 10 row 1).
3. **Lens outputs are readings** — typed values/reports. Translate/eval do NOT depend on lens output. Lens output does NOT feed downstream stages of the homomorphism.
4. **Built-in and user-defined lenses share one algebra contract.** The compiler core does NOT import or name any specific lens. The former `04_infer.dag` import of `v4.lens.cost { SymbolicCost }` is resolved as of the 2026-05-21 audit; reintroducing any specific lens import in compiler core would violate this commitment.
5. **Multi-lens execution is dependency-managed.** Lenses with no interdependencies may be coalesced into a shared traversal. Lenses with dependencies (e.g., complexity reads cost) are scheduled by a **lens-dependency DAG**; each stage coalesces all lenses whose inputs are available. No lens may trigger an ad hoc unmanaged re-walk. See Ratified Q6 below — derives from `fold_node` + composed-lens-algebra; NO new substrate primitive.
6. **Correct-by-construction before policy gates.** If a "lens" is actually checking a structural invariant that can be encoded in the type of `InferredTree` or an earlier carrier, push it into that type/model instead of making it a runtime gate. Measurement and report lenses (complexity, cost, effect reads, synthesis reports) remain pure readings. Terminal authorization may still return `Outcome<Validated<ArtifactSet>>`; that carrier belongs to policy over lens readings + projection results, not to compile-core.

**Surface consequence.** `validate_then_compile(source, input_lang, lenses, mode) -> Outcome<Validated<Output>>` is not the architectural destination because it makes compile own lenses and terminal policy. The migration target is a session result exposing four distinct phases: grounding, lens report, projection report, and terminal authorization. The latter three phases are present only under accepted grounding. `Validated<T>` remains valid at the terminal authorization phase, reframed as `Validated<ArtifactSet>`.

### Orthogonal Graph Consumers

The public build/session layer is a composition of four phases:

```
type IngestionPlan {
  subject: CoreNode
  producer: IngestionProducerRef
  params: Node
}

ground(plan: IngestionPlan) -> Outcome<InferredTree>
observe(inferred: InferredTree, plan: LensPlan) -> LensReport
project(inferred: InferredTree, plan: ProjectionPlan) -> ProjectionReport
authorize(policy: TerminalPolicy, lenses: LensReport, projections: ProjectionReport)
  -> Outcome<Validated<ArtifactSet>>
```

`ground` is the mechanical compiler boundary: if it rejects, there is no meaningful graph for lenses or projections to consume. Its input-side contract is plan-shaped for the same reason projection is plan-shaped: a producer reference plus typed `params` may carry an explicit `LanguageModel`, an inferred-language policy, or a richer ingestion/injection convention, but language selection should not be a one-off scalar beside the output plan. `observe` reads the grounded graph and returns lens data. `project` produces artifacts independently per request. `authorize` is the policy gate over readings and projection results.

```
compile_session(
  ingestion_plan: IngestionPlan,
  lens_plan: LensPlan,
  projection_plan: ProjectionPlan,
  policy: TerminalPolicy
) -> CompileSessionResult

type CompileSessionResult
  = GroundingRejected {
      diagnostics: List<Diagnostic>
    }
  | GroundedSession {
      graph: InferredTree
      lens_report: LensReport
      projection_report: ProjectionReport
      terminal: Outcome<Validated<ArtifactSet>>
    }
```

The result carrier is fail-closed by construction: `GroundingRejected` has no lens report, projection report, or terminal release because there is no `InferredTree` for those phases to consume. Only `GroundedSession` may expose readings, projections, or authorization.

`compile_session` is not compile-core. It is the public terminal / build-session orchestrator.

**Graph naming judgment.** The operator question is whether the common carrier should be called `Graph` or whether the existing `InferredTree` / `InferredGraph` should be formalized. My judgment: use `InferredTree` as the current implementation name, but document it as the **public grounded graph carrier** and plan a later rename to `InferredGraph` when the semantic dependency sidecar becomes explicit. Introducing a brand-new `Graph` beside `InferredTree` today risks duplicate authority; using `Graph` as prose keeps the principle clear without adding a second carrier.

**Projection requests use a registry, not a closed enum.** Do not replace `CompileMode` with a larger closed `ProductionGoal = TranslateTo | Eval | GenerateInterface | ...` enum. Projection requests point at a producer reference:

```
type ProjectionRequest {
  id: ArtifactRequestId
  subject: ProjectionSubject
  producer: ProjectionProducerRef
  params: Node
  requirement: ArtifactRequirement
}
```

Examples: `emit_rust_source`, `eval_on_host`, `generate_rest_glue`, `generate_test_claims`, `generate_docs`. Each producer owns typed params and a projection subject. Compile-core does not name the registry entries.

**RegionRef is low-level; subjects are refined per concern.** A low-level region reference can be shared, but lens scopes and projection subjects are not one universal `SubgraphScope`:

```
type RegionRef
  = WholeTree
  | DeclarationRegion { decl: DeclId }
  | BoundaryRegion { boundary: SystemBoundaryId }
  | DependencyClosedRegion { root: Node, witness: DependencyClosureWitness }

type LensScope { region: RegionRef }

type ProjectionSubject
  = WholeProgramSubject
  | DeclarationSubject { decl: DeclId, closure: DependencyClosureWitness }
  | BoundarySubject { boundary: SystemBoundaryId, protocol: ProtocolModel }
  | ClaimSetSubject { profile: TestProjectionProfile }
```

A lens can often run over any selected graph. An artifact projection cannot always be produced from any arbitrary region; its subject type records the extra closure/protocol/profile facts it requires.

**The hard substrate work** that 1–6 imply:
- A `LensAlgebra<F>` carrier shape that both built-ins and user-defined lenses satisfy.
- The `LensAlgebra<F>` carrier shape + composed-lens-algebra construction (per Ratified Q6 — NOT a substrate primitive; derives from `fold_node`).
- Stable `InferredTree` shape that won't break the lens contract as new substrate facts are added.

### P4 — Glue derivation is composed homomorphism, orthogonal to the compiler

For the eventual full-stack omni-emission story (one `.dag` source → backend Rust + frontend JS + Postgres schema + the wire-format glue between them), inter-module marshaling is structurally a **composed homomorphism**: source → wire-format model → target. Same primitive (the coercion fold), applied twice through a shared transport model.

For this to work, two substrate slots must exist:
- `extdeps/protocols/` — transport semantics (REST, GraphQL, gRPC, …). **Currently missing.**
- `std/system.dag` or equivalent — module decomposition substrate. **Currently informal.**

Out of scope for the initial single-target compiler; in scope for the architecture (must not cement a shape that prevents glue derivation later).

### Terminology: catamorphism / homomorphism / coercion

Three terms recur in this doc at three different levels. Disambiguating them up front prevents confusion:

| Term | What it is | Level |
|---|---|---|
| **Catamorphism** | A *shape of computation* — generic structural recursion over a tree (the fold pattern). `fold_node` is the substrate primitive that provides this. | Mechanism |
| **Homomorphism** | A *structure-preserving map* between two algebraic structures — operations compose the same way after the map. Translation IS the derived homomorphism. | The noun (the *what*) |
| **Coercion (fold)** | A specific *task* — verify that a candidate homomorphism is valid (structure preserved). The coercion fold is a catamorphism whose algebra zip-walks two canonical Node groundings checking structural equality. Per TASKS.md T-9 line 418 (D2 reversal, 2026-05-17 ratified) — never call it "search" or "engine." | The verb (the *how-we-check*) |

In one sentence: **the coercion fold is a catamorphism whose job is to verify a homomorphism.** Translation produces the target via the homomorphism; coercion verifies the target preserves source structure; catamorphism is the underlying mechanic both use. The doc uses these as distinct concepts; if a passage reads as conflating "the coercion fold" with "the homomorphism" generically, that's sloppy wording to flag — they sit at different levels.

### P5 — Fold discipline does not imply purely local bottom-up computation

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

Compiler stages are expressed as folds/traversals over Node, but their carriers may be **higher-order**, **effectful**, **constraint-bearing**, or **fixpoint-seeking**. Non-local mechanisms — scope resolution, type constraint solving, the coercion fold's structural-equality check, fixed-point analyses — must be **declared substrate machinery** (named primitives) and must produce **checkable witnesses** or **fail-closed diagnostics**. They must **never** appear as ad hoc compiler logic.

Practical implications (post-Pass B unification — all three are `find_witness` invocations, not separate primitives):
- Resolve's algebra is not naively bottom-up. The carrier is roughly `Node → Env → Outcome<BoundNode>` — at each Node the fold returns a function that takes inherited scope context. This is an attribute-grammar-style inherited attribute, expressed through the carrier's higher-orderness.
- Ground (infer) may produce a `ConstraintGraph` per Node, then invoke `find_witness` with the constraint-satisfaction predicate (the operation named `solve_constraints` in the derived-operations glossary). Output is the **canonical source grounding** + `StructuralPropertyWitness<canonical>`. Consumed later by translate's `find_witness` invocation.
- Translate's coercion-fold step is `find_witness` with the exact-structural-equality-zip-fold predicate. The candidate set is closed (declared in `target_lang.dag`) and the structural check is decidable by construction — this is **deterministic candidate enumeration**, not a heuristic search. Output is `(TargetNode, HomomorphismWitness<exact_equality>)`.
- Future `LawfulRewriteWitness` (P7) is `find_witness` with a lawful-rewrite-precondition predicate. Same primitive, different predicate.

The discipline still holds (everything is a substrate-primitive operation), but "fold" doesn't mean "pure synthesized bottom-up." Inherited context, effects, constraints, and fixed-point analyses are first-class — they're just carried by the algebra's signature, not by ad-hoc walkers. Each previously-distinct primitive (`solve_constraints` / coercion fold / `LawfulRewriteWitness`) is a `find_witness` invocation with a different preservation predicate — single primitive, three predicates.

### P6 — Dependencies are first-class typed relationships, projected by lens from substrate usage

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).
> **Sharpened 2026-05-20 (operator-direct):** dependencies are projections of substrate usage via a typed-dependency lens, NOT parallel-authored edge data. The substrate's usage graph (imports + field references + function calls + Edge-on-`Node`) IS the dependency graph; `std/dependency.dag` owns the **classification** of usage patterns (the `DependencyKind` enum) + the **lens fold** that emits emergent typed-relationship tuples. Authoring a parallel `DependencyEdge` / `DependencyReason` carrier alongside the substrate usage that already encodes those facts = Practice 11 parametric-duplication-of-mechanism.

The compiler does NOT treat DAG-ness as an incidental property of syntactic containment. It treats the substrate's usage graph as a **typed dependency graph**, projected by lens. Multiple kinds of dependency relationships emerge from substrate usage, and each kind has distinct semantic obligations. The lens classifies each substrate usage site into a typed relationship; the substrate IS the graph; the lens projects the typed view.

**Dependency kinds** — the classification taxonomy that the lens fold maps substrate usage patterns into (in `std/dependency.dag` — currently missing substrate):

| Kind | Meaning |
|---|---|
| `Contains` | structural / syntactic containment — what `fold_node` walks |
| `BindsTo` | Atom → Decl reference (added by resolve) |
| `TypeDependsOn` | type-inference dependency — this type's shape requires that one's resolution |
| `DataDependsOn` | dataflow — this value's computation reads that value |
| `EffectDependsOn` | effect ordering — this action must precede that action; non-commuting effects |
| `ResourceDependsOn` | resource access — alias / borrow / memory model constraints |
| `ModuleDependsOn` | module/import dependency — for incremental rebuild and visibility |
| `BarrierBefore` | synchronization / fence requirement (relevant for concurrent targets like CUDA, MapReduce) |
| `PlacementDependsOn` | host/device / partition / shard placement constraint |
| `ModelDependsOn` | artifact-tier — this artifact's regeneration requires that model to be the input authority (P8) |
| `ProjectionDependsOn` | artifact-tier — this artifact was produced by that projection producer (P3 + P8) |
| `GeneratedFrom` | artifact-tier — this artifact's content is a function of that model node's state (provenance edge) |
| `VerifiedBy` | artifact-tier — this artifact's correctness requires that lens / witness to validate it before consumption |
| `PromotedBy` | bootstrap-tier — promotion-protocol provenance edge (per P9; `Stage0[k+1]` was promoted by *this* `PromotionWitness` instance — distinct from `BootstrapDependsOn` which carries "what artifacts the promotion requires," whereas `PromotedBy` carries "which witness authorized the promotion") |
| `BootstrapDependsOn` | bootstrap-tier — promotion gate requirement (per P9; an artifact in the bootstrap candidate path that other artifacts in the path require) |

**Stage obligations** — each stage extends substrate facts that the dependency lens then classifies as the corresponding `DependencyKind`. The stages don't author `DependencyKind` labels directly; they extend substrate (containment, binding annotations, type/data facts) that the lens reads.
- `parse` produces the structural containment substrate (Node-DAG Edges); the lens classifies these as `Contains`.
- `resolve` monotonically extends substrate with `BindsTo`-grounding facts (Atom → Decl bindings, module-import facts); the lens classifies them as `BindsTo` and `ModuleDependsOn`.
- `ground` (infer) monotonically extends substrate with type-resolution facts, dataflow facts, and (where the source language declares them) effect/resource facts; the lens classifies them as `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`.
- `translate` to a target consumes the **lens-projected view** of the dependency graph (a fold over the substrate via `dependency_lens`). A translation is valid **only if** it preserves the dependency partial order projected by the lens, OR if the target model supplies a **checkable witness** that a reordering, fusion, parallelization, or placement change is semantics-preserving (e.g., a witness that `map` is independent per element + has no non-commuting effects → safe to parallelize).
- `eval` schedules only according to satisfied dependencies projected by the lens. Parallelism is the default where the lens-projected dependency view permits it.

**Cycles are not all rejected.** Mutually recursive functions, recursive types, fixed-point definitions, loops, and feedback systems are valid cyclic dependency structures. The substrate must classify each cycle:
- Valid fixed-point / SCC structure with productive witness → permitted; condensed to a strongly-connected component for topological purposes.
- Invalid cycle (module import cycle without fixpoint; non-productive type-recursion; deadlocked effect cycle) → fail-closed diagnostic.

**Implication for derived behavior:** parallel execution, MapReduce-style sharded execution, CUDA-style kernel placement, memoization, incremental rebuilds, and target-specific scheduling are **all derived from the typed dependency graph + algebra-inhabitance witnesses**. They are NOT special compiler modes and NOT target-specific emitters. The compiler carries the facts; targets and runtimes interpret them.

The distinction matters: **semantic dependency facts are compiler-core; scheduling/optimization decisions are downstream.**
- Core: "x must be bound before it's used," "write A must happen before read B," "this function depends on this imported module."
- Downstream (lens / target-policy / runtime): "fuse these two maps," "shard this reduce 32 ways," "place this on GPU."

**Edge orientation convention.** All dependency edges use one orientation: `A → B` means **A is required before B** — A must be available, valid, or scheduled before B may be resolved, checked, or executed. Examples:
- `Decl → AtomUse` (BindsTo: declaration must exist before reference resolves).
- `ProducerValue → Consumer` (DataDependsOn).
- `ImportedModule → Importer` (ModuleDependsOn).
- `PriorEffect → LaterEffect` (EffectDependsOn / BarrierBefore).
- `ParentContainer → ChildContent` (Contains, when scheduling matters).

For edges that are naturally references (Atom binding), the edge payload may carry referrer/referent role tags, but the **scheduling orientation remains `required_before → dependent_after`**.

**Conservative-by-default rule.** Absence of an edge is evidence of independence **only under a `ClosedWorldDependencyWitness`** for the relevant region. Unknown effect or resource facts conservatively introduce ordering constraints or produce fail-closed diagnostics. Parallelism is permitted only when dependency classification is complete; uncertainty defaults to dependent, not independent. This matters for CUDA / MapReduce: missing `EffectDependsOn` or `ResourceDependsOn` edges from incomplete analysis must NOT be inferred as independence.

**Source-core vs target-plan dependency tiers.** Not every dependency kind belongs on `InferredTree`:

| Tier | Kinds | When introduced |
|---|---|---|
| **Source / core** | `Contains`, `BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn` | parse / resolve / ground — semantic facts about the program |
| **TargetPlan / Runtime** | `PlacementConstraint`, `TransferDependsOn`, `BarrierBefore` (for synthetic sync points), shard / partition / device-binding edges | introduced during translate / target lowering — *not* part of source semantics |

`PlacementDependsOn` from the initial taxonomy is a **TargetPlan-tier concept** unless the source language explicitly declares placement (rare). Most placement decisions belong to target / runtime policy, not source `InferredTree`.

### P9 — Bootstrap is stratified; stage0 is a generated artifact, not self-modifying code

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).
>
> **Empirical motivation:** [sunny-otter-371's PR #3407](https://github.com/gunb-ai/gunbc/pull/3407) (the stage0-mirror investigation) surfaced the concrete "stage0 mirror is fundamentally broken under constant manual patch pressure" finding that this stratification is the structural response to.

The compiler may model and regenerate its own implementation, **including stage0**, but **no running compiler generation mutates itself in place**. The "compiler edits itself" framing from earlier sections is precise only in this stratified sense:

> The compiler can **derive and verify a replacement stage0** from `Stage0Spec`. It cannot arbitrarily rewrite itself at runtime.

This precision matters. v2's pain was largely: "compiler contract changes → generated code changes → stage0 cannot regenerate enough of the compiler → manual stage0 edit required." v4 must structurally prevent that failure mode without falling into the opposite failure mode of free runtime self-modification (which is intractable to reason about).

### Three pipelines, not one

The architecture has **three distinct pipelines** that must not be collapsed:

| Pipeline | Input | Output | Surface |
|---|---|---|---|
| **1. Semantic compile pipeline** | user/source text (or `CoreNode` directly) | target source / eval Value | Per the rest of this doc — parse→normalize→resolve→ground→translate/serialize OR eval. The "compile" surface most users see. |
| **2. Artifact projection pipeline** | `InferredGraph` (the compiler's own modeled state, OR any user program) | generated compiler code / tests / docs / schemas / glue / diagnostics tables / stage1 compiler source | Lenses + projections per P3 + P8. Everything downstream of a model that's *not* the user-program target output. |
| **3. Bootstrap promotion pipeline** | current `stage0[k]` + candidate-next-compiler artifacts | `stage0[k+1]` (or rejection diagnostic) | The protocol that **replaces the active seed**. Guarded by `PromotionWitness` (composed gate with internal bootstrap-roundtrip + fixed-point sub-checks) + promotion checks. |

The mistake is collapsing all three into "the compiler edits itself." The right model:
- The compiler **models** itself (pipeline 1 reads `.dag` source of any program, including the compiler's own source).
- The compiler **generates artifacts representing** itself (pipeline 2 produces a candidate next-compiler as a projection).
- The compiler **verifies** those artifacts (pipeline 2 also runs lenses + testgen + fixed-point checks).
- **Only the build/promote protocol replaces the active stage0** (pipeline 3).

### What `stage0` is — and isn't

**stage0 should NOT be "the whole compiler, but worse."** It should be a **minimal seed runner** whose contract is explicitly modeled.

`Stage0Contract` declares:
- Load a canonical `CorePackage`.
- Understand a stable `Node` / Core schema.
- Run the minimal `fold_node` / `traverse_node` machinery needed to build `stage1`.
- Report diagnostics fail-closed.
- Emit one bootstrap target artifact.
- Verify content hashes / witnesses.

**stage0 does NOT necessarily parse the full evolving `.dag` surface language.** This is the key decoupling. A lot of v2 pain came from "stage0 must understand the same evolving contracts as the compiler" — that creates constant manual patch pressure. In v4:

```
compiler.dag         // human-authored source of truth
compiler.corepkg     // generated canonical bootstrap package (stable schema)
stage0               // seed that reads compiler.corepkg, NOT arbitrary surface syntax
```

`compiler.corepkg` is generated and checked by the real compiler pipeline, but `stage0` only needs the **stable canonical representation**. Surface syntax, parser, normalizer, lenses, testgen levels, language-model polish all evolve without breaking stage0 — because stage0 doesn't consume them, it consumes the canonical package.

### The bootstrap epoch loop

```
Epoch k (stable):
  Stage0[k] + CorePackage[k] ──> Stage1[k]
  Stage1[k] + SourceModels[k] ──> Stage1'[k]
  verify(Stage1[k] == Stage1'[k])  // in-epoch stability invariant
  // NOTE: this is NOT a promotion-witness check — Stage1[k]'s fixed-point
  // status was already proven by PromotionWitness[k] (bound to the
  // CANDIDATE during the prior k-1 → k promotion). The verify above
  // re-confirms steady-state stability; it does NOT produce a new
  // PromotionWitness.

Upgrade k → k+1 (when an upstream model changes):
  Edit SourceModels[k+1]
  current compiler computes:
    ChangeSet[k → k+1] + AffectedSet + RecomputePlan
  Stage0[k] + Bridge/CorePackage[k → k+1] ──> Stage1[k+1]
  Stage1[k+1] ──> Stage0Candidate[k+1] + Stage1Candidate[k+1]
  verify (both checks bind to PromotionWitness[k+1], over the CANDIDATE):
    Stage0Candidate[k+1] can produce Stage1Candidate[k+1]
      ──> PromotionWitness[k+1].candidate_bootstrap_roundtrip_sub_check
    Stage1Candidate[k+1] is a fixed-point (self-emits)
      ──> PromotionWitness[k+1].candidate_fixed_point_sub_check
    witnesses / tests / lenses pass
  promote:
    Stage0[k+1] = Stage0Candidate[k+1]
    (gate requires both sub-checks pass — non-self-emitting candidate
     CANNOT be promoted; THESIS facet 2 holds by construction)
```

The **active compiler never mutates itself in place**. The only "back edge" is a **promotion edge**, and it is guarded by:
- Candidate generated.
- Candidate verified (fixed-point + witnesses + tests).
- Candidate promoted via an explicit promotion protocol.

### Three cases for compiler-contract changes

| Case | Change shape | Bootstrap impact |
|---|---|---|
| **1. Normal downstream change** | Add target language; change testgen profile; change cost lens; change grammar sugar; change a non-bootstrap-touching algebra. | No stage0 change required. Affected artifacts recomputed via `affected_set` per P0+P8; stage0 untouched. |
| **2. Bootstrap-compatible stage0 change** | `Stage0Spec` changes, but the change is expressible in the existing CorePackage schema. | Current compiler generates `Stage0Candidate`; verify; promote. **No manual edit.** |
| **3. Bootstrap-breaking change** | The new model can't be consumed by the current stage0 (e.g., Node schema fundamentally changes). | Requires a modeled `Bridge[k → k+1]` (a migration package). If the bridge can't be generated, the system **fails closed** with `BootstrapBreak: current stage0 cannot consume CorePackage[k+1]; required bridge missing; affected contracts: ...`. Much better than discovering as a random manual patch later. |

Manual edits to stage0 are architecture violations except as explicitly declared emergency seed resets (documented and time-bounded).

### Critical invariant

> **At no point does an artifact become source of truth merely because it is needed for bootstrapping.**

Even if `stage0.rs`, `compiler.corepkg`, generated compiler source, and generated tests are checked in, they are **NOT authorities**. They are generated artifacts with hashes/witnesses. The source of truth remains:
- `.dag` models.
- `Stage0Contract`.
- `CorePackageSchema`.
- `Projection` definitions.

Generated artifacts that have drifted from their model are stale, not authoritative — the regeneration mechanism re-derives them, and a stale artifact whose model has changed is invalidated by the affected-set machinery.

### The conceptual stack

```
Layer 4 — Verification / promotion
  fixed-point check, homomorphism witness, bootstrap witness,
  generated-test execution, promotion of candidate stage0/stage1

Layer 3 — Generated compiler artifacts
  stage1 compiler, target serializers, generated tests,
  diagnostics tables, runtime/eval artifacts

Layer 2 — Compiler models (.dag, source of truth)
  parse / normalize / resolve / ground / translate / serialize / eval algebras
  lens algebras, projection producers (including testgen projection family)

Layer 1 — Substrate models (.dag, source of truth)
  Node, fold_node, find_witness (unified Pass B primitive),
  Outcome/Diagnostic/Locus, grammar-as-data, typed dependency graph,
  change/artifact/bootstrap substrate.
  Derived combinators on top: traverse_node, sequence_node,
  bind_outcome, apply_diff, coercion fold, solve_constraints
  (all are fold_node / find_witness invocations, not primitives).

Layer 0 — Seed
  stage0 executable. Minimal, boring, audited.
  Consumes canonical CorePackage, NOT arbitrary surface syntax.
```

Only **Layer 0 is hand-seeded** (and even that is regenerable via the promotion protocol).
Layers 1-2 are the **source of truth**.
Layer 3 is **disposable** (regeneratable from Layers 1-2).
Layer 4 is **procedural and conservative** (the gatekeeper).

### Two distinct dependency graphs

P6 names dependency edges over user programs. Bootstrap requires the **same idea applied to compiler artifacts** — but it's a *separate graph*:

| Graph | Edges | What it tracks |
|---|---|---|
| **Program dependency graph** (P6) | `Contains`, `BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn`, `BarrierBefore`, `PlacementConstraint` | The program being compiled. |
| **Build / bootstrap dependency graph** (new) | `ModelDependsOn`, `ProjectionDependsOn`, `GeneratedFrom`, `VerifiedBy`, `PromotedBy`, `BootstrapDependsOn` | Compiler artifacts and their provenance. |

Keeping them separate prevents the confusion "does the compiler's own resolver depend on the resolver it is resolving?" — answer: at epoch k, `Stage0[k]` resolves `model[k]` enough to build `Stage1[k]`. No active stage depends on its own output. The two graphs use the same substrate ideas but model different relationships.

### Witness taxonomy

*(Pass B unification 2026-05-20: 7 witness types → 3 generic carriers parameterized by what's being witnessed.)*

| Witness | Generic parameter | What it asserts | Instances |
|---|---|---|---|
| **`StructuralPropertyWitness<P>`** | `P` = the structural property | "This graph has structural property P" | `P = being-canonical` (was CanonicalGroundingWitness); `P = having-no-invalid-cycles` (was AcyclicityWitness); `P = being-completely-classified` (was ClosedWorldDependencyWitness). |
| **`HomomorphismWitness<R>`** | `R` = the preservation rule | "Rule R holds between source and target" | `R = exact-structural-equality` (was Witness<homomorphism> produced by coercion fold); `R = lawful-rewrite-precondition` (was LawfulRewriteWitness — per algebra-laws like associativity-for-tree-reduce, per-element-independence-for-parallel-map). |
| **`PromotionWitness`** | (not generic — composes two specific checks, **both bound to the CANDIDATE at k+1**) | "This **candidate** (Stage0Candidate[k+1] + Stage1Candidate[k+1]) is promotion-ready: the candidate Stage1 is a fixed-point AND the candidate Stage0 bootstrap-roundtrips to the candidate Stage1" — the fixed-point check is over the candidate, NOT the existing Stage1[k]; otherwise the gate could promote a non-self-emitting compiler. | Was BootstrapWitness + FixedPointWitness — composed into one promotion-gate witness. |

All are **checkable substrate data**, not opaque proofs — per Ratified Q9 (next section), the derivation is untrusted; the witness check is trusted.

**Why the collapse:** the previous 7 witnesses were named by *what produces them*, not *what they assert*. CanonicalGroundingWitness / AcyclicityWitness / ClosedWorldDependencyWitness all assert a structural property of a graph — one carrier parameterized by which property. Witness<homomorphism> + LawfulRewriteWitness both assert a preservation rule holds — one carrier parameterized by which rule. BootstrapWitness + FixedPointWitness both gate promotion — one carrier composing the two specific checks. Pass B's "concept unification" thesis applies recursively to the architecture's own carrier set.

### New bootstrap substrate (needed but not yet declared)

> **Concrete shape of `Stage0Contract` / `CorePackage` / `CorePackageSchema`** is **deferred to a future dispatch** when bootstrap implementation is in scope (held per the current dispatch hold on `std/bootstrap.dag`). This section names the substrate shape needed; the field-level structural design is its own substrate brief, not part of the regeneration-substrate cluster this doc dispatches.

| Concept | Purpose | Status |
|---|---|---|
| **`Stage0Contract`** | What stage0 promises to do (consume canonical CorePackage; run minimal `fold_node` + derived `traverse_node` combinator; fail-closed diagnostics; emit verified artifact). | Not declared. Likely `std/bootstrap.dag`. |
| **`BootstrapEpoch`** | The k-indexed snapshot of (stage0, CorePackage, SourceModels). | Not declared. |
| **`CorePackage`** | The stable canonical bootstrap package consumed by stage0. | Not declared. |
| **`CorePackageSchema`** | The schema describing what CorePackage carries (changes here are bootstrap-breaking per Case 3). | Not declared. |
| **`Bridge`** | Migration package for bootstrap-breaking changes (Case 3 of bootstrap-impact taxonomy). | Not declared. |
| **`PromotionWitness`** *(composed gate; Pass B witness-taxonomy unification)* | Single promotion-gate witness composing two specific checks **both bound to the CANDIDATE at k+1, NOT the existing stage at k**: (a) the **candidate-bootstrap-roundtrip check** — `Stage0Candidate[k+1]` can produce `Stage1Candidate[k+1]` (was `BootstrapWitness`); (b) the **candidate-fixed-point check** — `Stage1Candidate[k+1]` compiles to itself (i.e., the *candidate* is a self-emitting fixed point per THESIS facet 2; was `FixedPointWitness`). **The fixed-point check is over the candidate, not the existing `Stage1[k]`** — `Stage1[k]`'s stability is already-known from the prior promotion's witness; promotion to k+1 requires proving the *new* compiler self-emits, not merely re-confirming the old one is still stable. Without binding to the candidate, the gate could promote a non-self-emitting compiler (THESIS facet 2 violation). The two checks are **internal sub-fields** of the single `PromotionWitness` carrier — NOT separate authorities. Implementers see one promotion-ready witness, not three. Matches the witness-taxonomy collapse at line 511 above (no parallel authority). | Not declared. |
| **`PromotionPlan`** | The procedural step from candidate to active stage0. | Not declared. |
| **`PromotionDiagnostic`** / **`BootstrapBreak`** | Fail-closed diagnostic shape for un-promotable candidates. | Not declared. |

These compose with the regeneration substrate from P8 (`ChangeSet`, `AffectedSet`, `Projection`, `Artifact`, `RecomputePlan`) — bootstrap substrate is a specialization for compiler artifacts. Likely implementation order: declare in `std/bootstrap.dag` alongside the P8 regeneration concepts.

### What this rules out

If implementation workers find themselves writing:
- Code that lets stage0 directly edit itself at runtime.
- A "bootstrap workaround" that's a hand-authored stage0 patch outside the candidate/promotion path.
- A "well, we know this change is safe" that skips fixed-point verification.
- Generated-artifact files treated as authorities (e.g., editing the generated stage0 source to fix a bug instead of editing the model).

…that is a **STOP condition**. Escalate. The bootstrap layer is intentionally conservative; bypassing it produces v2's pain.

### P8 — The compiler is not exempt from its own discipline

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

The v4 compiler must receive the same structural benefits it provides to user programs. **Compiler stages, language models, lenses, test generation, target models, diagnostics, and glue derivation are themselves modeled as dependency graphs over the same substrate primitives.**

A change to an upstream model — Node shape, grammar productions, `LanguageModel`, `ModelCore`, dependency kind, lens contract, coercion-fold witness shape, or runtime carrier — must produce a `ChangeSet`. The system computes the affected downstream compiler artifacts (via T-21 `affected_set`) and regenerates them where derivable. If a downstream artifact cannot be regenerated from the updated model, the compiler **fails closed** with an explicit diagnostic.

**No compiler layer may depend on another layer through:**
- Implicit convention.
- Manual synchronization (developer chases the change across layers by hand).
- Hidden emitter (executable emitter logic smuggled into a model file).
- Hidden parser (token-by-token recognition that bypasses grammar-as-data).
- Hidden test generator (special-case test-emission code outside the projection framework).
- Hidden integration updater (per-pair language adapter code that should be derived).
- Hidden layer-specific patcher (ad hoc walker fixing up stale output of an upstream stage).
- Ad hoc Node traversal outside `fold_node` / `traverse_node` / `apply_diff` / lens primitives (Practice 10 row 1 violation, applied to compiler internals).

**Every compiler-internal artifact must be one of:**
- A substrate primitive (declared in `std/`).
- A declared algebra over a substrate primitive.
- A lens (in the read/edit framework).
- A projection producer / artifact projection (P3 commitment 1; output of `project`, not `observe`).
- A target/runtime policy (declared in `extdeps/`).

**The slogan.** *The compiler is the first consumer of its own modeling discipline.* If implementation workers find themselves writing ad hoc code to keep compiler layers in sync, that is a **STOP condition** — escalate, do not patch.

**Worked example: testgen self-regeneration.** Suppose `Refinement<T>` (in `std/refinement.dag`) gains a new field. A v2-style compiler would require manual updates to every place that emits boundary-tests-for-refinements, every target-language test renderer, every fixture template. The v4 compiler:

1. `affected_set` computes the downstream artifacts whose inputs include `Refinement<T>` (TestClaimProjection, BoundaryTestProjection, every TargetTestProjection that reads boundary tests).
2. Each affected projection regenerates from the new `Refinement<T>` shape via its declared projection producer.
3. If a projection's algebra can't accommodate the new shape (e.g., needs a fact the new model doesn't carry), it surfaces as a diagnostic, NOT a silent breakage.

**Worked example: language-model self-regeneration.** Suppose `LanguageModel` is extended with a new field (e.g., effect-semantics declarations per Open Q10). The compiler:

1. `affected_set` computes which compiler stages / lenses depend on `LanguageModel`'s shape (resolve consults binding rules; ground consults inhabitance; translate consults grammar; …).
2. Each affected stage's algebra extends or **fails closed** on the new field.
3. Existing `rust.dag` / `python.dag` / etc. that lack the new field must either: (a) declare a **typed `Default<T>` witness** for the field (an explicit modeled default, NOT an implicit silent default), or (b) surface a fail-closed diagnostic until updated. There is **no silent default acceptance** — per P3 fail-closed, missing newly-required facts produce either a typed witness OR a diagnostic, never an unsignalled default.

There is no manual sync. The compiler is rebuilt from its models the same way user programs are rebuilt from theirs.

### Named regeneration substrate (needed but not yet declared)

To operationalize P0 + P8, the substrate needs explicit concepts for change-consequence machinery. **These must land as a coherent unified declaration, not piecemeal.** T-21 `affected_set` is a *specialization* of `AffectedSet` (lens-frontier only); declaring the rest piecemeal while leaving regeneration authority split across declared+undeclared concepts is itself a P2 boundary violation. The substrate listed below is one unit — `std/regeneration.dag` (or a tightly-related cluster) authoring it as a coherent declaration so the regeneration authority is single-sourced:

| Concept | Purpose | Status |
|---|---|---|
| **`ChangeSet`** | A diff between two model versions (changed Nodes, edges, facts, witnesses, grammar productions, LanguageModel declarations, lens contracts). | Not declared. Likely `std/change.dag` (or part of unified `std/regeneration.dag`). |
| **`AffectedSet`** | Downstream artifacts requiring recomputation given a `ChangeSet` + dependency graph + projection graph. The **general** substrate; T-21 `affected_set` is the lens-frontier specialization and must be hoisted into this general definition once it lands (not maintained as a parallel authority). | T-21 specialization landed; general `AffectedSet` not declared. **The two MUST unify** — split authority is a P2 violation. |
| **`Projection`** | A declared projection-as-data: name, input carrier, output artifact, dependency requirements, regeneration producer, validation policy inputs. | Not declared. |
| **`Artifact`** | An emitted artifact (generated source file, test file, schema, diagnostic table, compiler stage table) with its provenance (which projection produced it from which model state). **`ArtifactKind` reserves bootstrap-related variants up front** — `Stage0Candidate`, `CorePackage`, `WitnessBundle` — so P9 bootstrap substrate can land later without forcing an artifact/projection refactor. | Narrow substrate declared in `src/v4/std/artifact.dag` (`ArtifactKind`, `Artifact`, `NodeArtifactProvenance`). General regeneration integration with `Projection` / `AffectedSet` / `RecomputePlan` is not declared. |
| **`RecomputePlan`** | Topological schedule + SCC/fixpoint groups + cached-vs-invalidated artifacts + regeneration ordering. The "how to recompute" data given an `AffectedSet`. | Not declared. |

**Implementation order:** declare the regeneration substrate as a unified cluster (likely `std/regeneration.dag` + the bootstrap substrate in `std/bootstrap.dag`); land alongside or after the `LensAlgebra<F>` + composed-lens-algebra substrate work (Ratified Q6 — derives from `fold_node`; NOT a separate primitive). T-21's `affected_set` must be **migrated** to the unified `AffectedSet` shape, not left as a parallel concept — the unified declaration is the single source of authority.

### P7 — Lawful rewrite witnesses for structure-changing lowerings

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

The coercion fold (P6 / primitive set) checks **exact structural preservation** between two canonical groundings — it is a mechanical zip-fold, decidable by construction, NOT heuristic. But some legitimate target lowerings change structure:

- Sequential `map` → parallel `map`.
- Left fold → tree reduce.
- CPU loop → CUDA kernel launch + device transfer + barrier + copy-back.
- Sequential reduce → MapReduce shuffle + per-key reduce.
- Iterator chain → fused single-pass loop.

These are valid under the algebra's laws (map's per-element independence, reduce's associativity, etc.) but they are NOT exact structural equality.

**Resolution:** structure-changing lowerings use `find_witness` with a **lawful-rewrite-precondition predicate** (post-Pass B unification — NOT a separate primitive; same `find_witness` mechanic as `solve_constraints` and the coercion fold, with a different predicate). The pipeline becomes:

```
canonical source grounding
  ── find_witness(source, declared_rewrites, lawful_rewrite_precondition,
                  MultiplicityPolicy::RewritePolicy(...))  ──>
target-plan grounding + HomomorphismWitness<lawful_rewrite>
  ── find_witness(plan, target_inhabitants, exact_structural_equality,
                  MultiplicityPolicy::TargetSelection(target_lang.target_selection_policy)) ──>
target Node tree + HomomorphismWitness<exact_equality>
```
(The first `find_witness` invocation — lawful-rewrite — is P7-gated and NOT in MVP; this diagram documents the future pipeline shape only. The second — exact coercion — is in MVP scope.)

The `HomomorphismWitness<lawful_rewrite>` is the witness that proves a structure-changing rewrite preserves semantics under declared algebraic laws. Each rewrite is **declared by the target/runtime model** (e.g., `extdeps/runtimes/cuda.dag` declares "sequential map over Vec<T> may rewrite to CUDA kernel iff per-element function is pure + has no cross-element data deps"). Searches and rewrites remain **closed and decidable** — candidates are declared, every rewrite produces a checkable witness, none is heuristic.

This preserves the "coercion fold is not heuristic search" principle while leaving room for MapReduce / CUDA / parallel-map / fusion lowerings. Without this distinction, implementation workers would have to choose between (a) cementing CUDA/MapReduce into the compiler (Practice 10 row 3 failure) or (b) extending the coercion fold to do heuristic search (T-9 D2-reversal violation). Both are wrong; P7's `find_witness`-with-rewrite-precondition is the third path.

**Historical naming preserved:** the term `LawfulRewriteWitness` is preserved as the canonical name for **`find_witness` invoked with the lawful-rewrite-precondition predicate** — same way "coercion fold" is preserved as the canonical name for **`find_witness` invoked with exact-structural-equality predicate** (per T-9 ratification). Both produce instances of `HomomorphismWitness<R>` with different `R` parameters.

---

## The primitive set — the things the compiler IS

| Primitive | Purpose | Status |
|---|---|---|
| **`fold_node`** (catamorphism over Node) | Generic structural recursion over Node trees; anti-walker-dissolution substrate (Practice 10 row 1). Every "pass" the compiler runs is `fold_node(tree, algebra)`. **Landed `NodeFold<R>` carrier** (pure catamorphism, no Outcome at the primitive level): `init: fn(Node) -> R` (seed per-Node) + `step: fn(R, Edge, R) -> R` (combine accumulator with edge + already-folded child result `R`). **Outcome-threading is NOT in the primitive** — it lives in the derived `NodeOutcomeFold` combinator (see "Derived combinators" below), where the combinator (not the algebra) interprets `StageDiagnosticPolicy`. Separating the pure-catamorphism primitive from the Outcome-threading combinator is what keeps single-authority for failure handling at the combinator layer. | **Landed** in `src/v4/std/node.dag` line 43, with the `NodeFold<R>` `init`/`step` carrier above. Outcome-threading combinator (`NodeOutcomeFold`) is **not yet landed** — Worker C's escalated signature-collapse work delivers it; lands as part of the 43-callsite Outcome-shape migration (see Diagnostic row below). |
| **Grammar-as-bidirectional-data** | One declarative grammar model in `extdeps/languages/X.dag` serves both parsing (text → Node) and serialization (Node → text). No separate parser and printer; the grammar production data IS the relation (Practice 8). **Default law:** `parse_target(serialize_target(target_node)) == target_node` — serialization may canonicalize; full source-text round-tripping is opt-in tooling. | **Partially landed** — `Grammar / ModeledGrammar / VoidGrammar` declared in `src/v4/compiler/02_parse.dag`. Inverse-grammar walk scaffolded but not implemented. |
| **`find_witness`** *(unified primitive — Pass B unification 2026-05-20)* | THE unified **candidate-enumeration-and-check** primitive. Given `(source_facts, closed_candidate_set, preservation_predicate, multiplicity_policy)`, mechanically enumerates the **entire closed candidate set** and checks the preservation predicate against **every** candidate; **collects ALL passing candidates** (not first-match), then resolves via the **caller-supplied** `multiplicity_policy`: **exactly one passing candidate** ⇒ `Outcome::Accepted { value: (candidate, witness), diagnostics }`; **zero passing candidates** ⇒ `Outcome::Rejected { diagnostics: NonEmptyDiagnostics { head: NoCandidate { ... }, ... } }`; **multiple passing candidates** ⇒ the policy decides — `MultiplicityPolicy::UniqueOnly` ⇒ `Outcome::Rejected { diagnostics: NonEmptyDiagnostics { head: AmbiguousCandidate { candidates }, ... } }` (fail-closed, no selection); `MultiplicityPolicy::TargetSelection(target_selection_policy)` ⇒ apply the target language model's declared `TargetSelectionPolicy` per Ratified Q14 — if it deterministically resolves, `Outcome::Accepted { value: (selected_candidate, witness), diagnostics }`, else `Outcome::Rejected { diagnostics: NonEmptyDiagnostics { head: AmbiguousTargetCandidate { candidates }, ... } }` (record/struct shape per Ratified Q11 — NOT positional variant constructors). **The primitive does NOT bake in any specific selection policy.** `MultiplicityPolicy` is a typed parameter the caller supplies per use site; this keeps target-coercion-specific Q14 selection out of source-grounding (`solve_constraints`) and other non-target uses. **`solve_constraints` binds `MultiplicityPolicy::UniqueOnly` unconditionally** — canonical source grounding has no notion of "target preference"; multiple passing candidates ⇒ ambiguous grounding ⇒ fail-closed, period. The coercion fold binds `MultiplicityPolicy::TargetSelection(target_lang.target_selection_policy)` per Q14. `LawfulRewriteWitness` (P7) introduces a `RewritePolicy` variant when it lands. **NO positional-first-wins** — ambiguity detection is required; fail-closed semantics demand collecting all passing candidates before any acceptance. **NOT a search — per T-9 ratification, the candidate set is closed and the predicate check is mechanical/decidable.** The word "find" denotes the operation's output, not a search algorithm. **Unifies what were previously two distinct primitives:** `solve_constraints` (canonical-form candidates + constraint-satisfaction predicate + `UniqueOnly`; per T-9, decidable-by-construction over the closed Find set) and the coercion fold (target-language declared inhabitants + exact-structural-equality-zip-fold predicate + `TargetSelection`; per T-9, "mechanical zip-fold, decidable by construction, never a search"). Both are mechanical enumeration over closed sets — predicate, candidate source, **and multiplicity policy** differ per caller. | **Not yet declared.** Substrate likely lands in `std/find_witness.dag` (carrying the primitive operation + the `MultiplicityPolicy` carrier), with the predicate algebras + per-caller policy bindings living in their consuming files (`std/constraints.dag` for the constraint-satisfaction predicate + `UniqueOnly`; `std/coercion.dag` for the exact-zip-fold predicate + `TargetSelection`). |
| **Typed dependency graph** | Per P6, **the substrate's usage graph IS the dependency graph** (operator-direct sharpening 2026-05-20). `std/dependency.dag` owns the `DependencyKind` classification enum + the **dependency lens** fold algebra that walks substrate usage (imports, field refs, function calls, Edge-on-`Node`) and emits emergent `(source, dependent, kind, usage_site)` lens-output tuples — the `usage_site: Node` field is a **reference** to the substrate Node where the usage occurred (the import statement Node, the field-reference Node, the function-call Node — whichever is the lens's classification site). The reference is what downstream consumers (diagnostics formatter, `affected_set` provenance, T-21 frontier receipts) cite to flow the usage fact forward per Practice 3 (Facts Flow Forward) — it carries the substrate's existing fact via reference, **NOT** by duplicating it as a parallel authored payload. **No authored `DependencyEdge` / `DependencyReason` carrier** — the substrate usage site IS the reason; the kind is the lens's classification; the lens output cites the substrate (via the `usage_site` Node reference), it does not duplicate it. Authoring parallel-edge data (a `DependencyReason { kind, subject, evidence }` carrier with copies of substrate facts) would be Practice 11 parametric-duplication-of-mechanism. The distinction: **referencing the substrate via a typed Node pointer = lens output (correct); copying the substrate fact into a parallel carrier = parallel authority (forbidden).** SCC condensation + topological-order derivation are folds over the substrate (consuming the lens output), not separate primitives operating on parallel-authored data. Source/core relationships are separated from target-plan relationships (the lens's classification distinguishes them). | **Not yet declared.** Probably lands in `src/v4/std/dependency.dag` as: `DependencyKind` taxonomy + `dependency_lens` fold + `DependencyView { source, dependent, kind, usage_site }` lens-output type. T-21's `affected_set.dag` is the incremental-rebuild specialization built on the same lens-over-substrate substrate (migrates into unified `AffectedSet`; consumes `DependencyView.usage_site` for re-execution-frontier provenance). |
| **Diagnostic + Locus carrier** | Fail-closed reporting (Practice 1). Every failure path goes through a structured `Diagnostic` with source `Locus`. `Outcome<T>` shape: `Accepted { value, diagnostics: Diagnostics } \| Rejected { diagnostics: NonEmptyDiagnostics }` (two-variant collapse per Pass B correction #11). | **Landed** in `src/v4/std/diagnostic.dag` with legacy single-variant shape. **Outcome-shape migration is the single corrections-packet substrate task that lands in Worker C's corrected scope** (43 bootstrap/lens/compiler callsites use legacy `Produced{value}` / `Rejected{diagnostic}` — explicit migration sweep, not informal "corrections round" debt). Bounded scope: one PR, one substrate authority. |

That is the full compiler-primitive surface. **Five things** (was 9 before Pass B unification). Every stage, pass, translation, eval, glue derivation, and read/edit operation is structurally `fold_node + (one or more of the other four primitives) + an algebra-as-data`.

### Derived combinators (NOT primitives — Pass B unification)

These were primitives in earlier drafts; they are now **derived** from the 5-primitive set:

- **`traverse_node` / `sequence_node` / `bind_outcome`** = `fold_node` invoked with an `Outcome`-threading algebra; the combinator interprets `StageDiagnosticPolicy` (accumulate vs short-circuit). Algebras are pure success-accumulators (per Worker C's `NodeOutcomeFold.step` fix — combinator owns failure policy as single authority). `traverse_node` is `fold_node + NodeOutcomeFold`, not a parallel primitive.
- **`apply_diff`** = `sequence_outcome(diff.edits, fn(edit, candidate) -> fold_node(candidate, substitute_at(edit.at, edit.replacement)))` — sequential Outcome-bind-fold over the Diff's ordered Edits list. Each Edit applies to the *intermediate* candidate root via `fold_node` substitution; each subsequent Edit's Path resolves against the post-prior-Edit state (per the read/edit pipeline doc requirement). Fail-closed if any Path doesn't resolve in the current candidate. Composes with `fold_node` + `sequence_outcome`; doesn't extend the primitive set.
- **The coercion fold** = `find_witness(canonical_source_grounding, target_lang.declared_inhabitants, exact_structural_equality_zip_fold, MultiplicityPolicy::TargetSelection(target_lang.target_selection_policy))`. The "fold" name is preserved in TASKS.md T-9 ratification; conceptually it's an instance of `find_witness` with target-coercion-specific multiplicity resolution (Q14).
- **`solve_constraints`** = `find_witness(constraint_graph, canonical_form_candidates, constraint_satisfaction, MultiplicityPolicy::UniqueOnly)`. Same primitive, different predicate **and** different multiplicity policy — canonical source grounding requires exactly-one or fail-closed; no target-selection notion applies. Binding `UniqueOnly` at the call site (not the primitive) is what prevents target-policy leakage into source grounding (per openai-pro single-authority finding 2026-05-20).
- **`LawfulRewriteWitness`** (P7) = `find_witness(source_grounding, candidate_rewrites_declared_by_target_or_runtime, lawful_rewrite_precondition_predicate, MultiplicityPolicy::RewritePolicy(...))`. Same primitive again with a rewrite-domain `RewritePolicy` variant introduced when P7 lands.

**Why this terminology shift matters (coercion fold vs. "search"):** an earlier framing called the homomorphism step an "algebra-homomorphism search" or "inhabitance search," which suggests an open-ended search algorithm with potentially intractable complexity. The ratified position is the opposite: the candidate set is **closed and declared** (by the target language model), the source's grounding is **canonical and unique**, and the check is a **mechanical structural zip** — closer to type unification than to logic-programming search. If you find yourself reasoning about it as a search, that's a signal you're solving the wrong problem.

---

## The pipeline — stages as named algebras

```
  ┌───── INGEST LAYER (separable, multiple entry points) ─────┐  ┌──── COMPILE CORE (data → data) ────┐

  text ── parse(input_lang) ── normalize ─────┐               ┌────── resolve ── ground ──┬── translate(target_lang) ── serialize(target_lang.grammar) ── target_text
  (or)                                        ├──> CoreNode ──┤                            └── eval(runtime) ── execute(runtime)
  programmatic / API ─────────────────────────┤
  (or)                                        │
  query-driven rewrite (e.g. affected_set) ───┤
  (or)                                        │
  round-trip ingest of a prior TargetNodeTree ┘
```

The **ingest layer** is separable: text is one of several legitimate entry points. The **compile core** operates on `CoreNode` regardless of how it was produced.

Each stage is a fold/traverse over Node (potentially with a higher-order, effectful, or constraint-bearing carrier per P5). Facts flow forward (Practice 3, strong form): **each stage performs at most one disciplined fold/traverse over the current carrier and monotonically extends it. Facts are monotonic** — later stages extend or consume prior annotations rather than discarding and reconstructing semantic state. Resolve adds BindsTo edges; ground adds typeshape + inhabitance witnesses + the broader P6 dependency-graph edges. No stage re-derives facts from raw text or from a duplicated source-of-truth.

| Stage | Layer | Algebra produces | Where the algebra lives |
|---|---|---|---|
| `parse(input_lang)` | INGEST | `SurfaceNode` (Node tree in input_lang's grammar) | `input_lang.dag` (grammar productions) |
| `normalize` | INGEST | `CoreNode` (canonical substrate Node; surface sugar dissolved) | `input_lang.dag` (sugar dissolutions) — possibly fused with parse, see Ratified Q3 |
| `resolve` | COMPILE-CORE | `ResolvedCoreNode` (every Atom → Decl reference; BindsTo + ModuleDependsOn edges added per P6) | scope-resolution algebra (likely substrate, consults input_lang for binding rules per Ratified Q5) |
| `ground` | COMPILE-CORE | **`InferredTree`** — each Node carries algebra-inhabitance witness + typeshape + the P6 semantic dependency graph. **Canonical-grounding invariant:** ground must produce *exactly one* canonical grounding per Node (with witness), OR an ambiguity diagnostic. Translate never receives an ambiguous source grounding. | substrate's `solve_constraints` primitive + algebra-inhabitance machinery |
| `translate(target_lang)` | COMPILE-CORE | Target Node tree | optional `LawfulRewriteWitness` step (per P7, for structure-changing lowerings declared by `target_lang` / target runtime); then **the coercion fold** — exact zip-fold over (rewritten canonical grounding, target_lang's closed declared candidate inhabitants) |
| `serialize(grammar)` | COMPILE-CORE | target source text | target_lang.dag (grammar productions, inverse direction) |
| `eval(runtime)` | COMPILE-CORE | RuntimeValue | runtime interpretation algebra |

**Carrier shape progression:**
```
INGEST LAYER (separable, multiple entry points):
  Text → ParseTree (SurfaceNode) → NormalizedTree (CoreNode)
  (or)
  Programmatic builder       ─────→  CoreNode
  (or)
  Query-driven rewrite       ─────→  CoreNode

COMPILE CORE (data → data):
  CoreNode → ResolvedTree (ResolvedCoreNode) → InferredTree (InferredCoreNode)
                                              ├─→ TargetNodeTree → TargetSource  (TranslateTo mode)
                                              └─→ Value                          (Eval mode)
```

**Carrier taxonomy clarification.** "Node" can mean three different things depending on which carrier you're holding; the type-system distinction matters and the doc should be explicit:

| Carrier name | What "Node" means here |
|---|---|
| `SurfaceNode` (`ParseTree`) | language-specific parsed structure — shape declared by `input_lang.dag`'s grammar |
| `CoreNode` (`NormalizedTree`) | substrate-canonical Node — six connectives + five behaviors only, sugar dissolved |
| `ResolvedCoreNode` (`ResolvedTree`) | `CoreNode` + BindsTo edges |
| `InferredCoreNode` (`InferredTree`) | `ResolvedCoreNode` + typeshape + inhabitance witness + full semantic dependency graph (P6) |
| `TargetSurfaceNode` (`TargetNodeTree`) | target-language AST — shape declared by `target_lang.dag` |
| `TargetSource` | serialized text in target's concrete syntax |

The compiler core operates over `CoreNode` and its enrichments (`ResolvedCoreNode`, `InferredCoreNode`). `SurfaceNode` and `TargetSurfaceNode` are language-specific shapes; they exist at the boundaries (input parse, output emit) but the substrate-canonical Node is the only shape the homomorphism mechanic operates on.

`Node` (unqualified) generally means `CoreNode` or an enrichment of it in this doc.

The substrate-task IDs that implement each stage:
- **T-8** (in-flight) — `normalize` + `resolve`.
- **T-9** — `ground` (currently scaffolded in `04_infer.dag`).
- **T-10** — `translate` + `serialize` (currently scaffolded in `05_emit.dag` + `00_compile.dag`).
- **T-22** — `eval` (currently scaffolded in `05_eval.dag`).

See `src/v4/TASKS.md` for the full task dependency graph.

---

## Carrier shapes — what each tree carries

### `InferredTree` (the central artifact)

> **Naming note (P6 implication).** The name "InferredTree" is historical; the artifact is structurally a **tree-plus-typed-dependency-graph** — containment edges form the tree spine, semantic dependency edges (BindsTo, TypeDependsOn, DataDependsOn, EffectDependsOn, ResourceDependsOn, ModuleDependsOn) form sidecar / embedded edges per P6. A future rename to `InferredGraph` or `GroundedProgramGraph` would be more honest, but we keep "InferredTree" for now to avoid mass-renaming during the architecture phase.

The post-grounding artifact. Each Node `N` carries:
- **Locus** — source position (from parse, propagated forward).
- **Binding** — for any Atom: resolved FunctionRef / Decl reference (from resolve, propagated forward).
- **Typeshape** — the inferred type, itself a substrate Node referencing an algebra-instance.
- **Inhabitance witness** — which algebra-instance this Node inhabits + the structural proof. Consumed downstream by `translate`'s coercion fold and `eval`'s interpretation.

The artifact ALSO carries (P6):
- **Containment edges** — the syntactic tree spine (`Contains`).
- **Semantic dependency graph** — typed edges added monotonically by `resolve` and `ground`:
  - `BindsTo` — every Atom's reference resolution.
  - `TypeDependsOn` — type-inference dependency edges.
  - `DataDependsOn` — dataflow edges (this value's computation reads that value).
  - `EffectDependsOn` — where the source language declares effects, ordering edges between non-commuting actions.
  - `ResourceDependsOn` — where the source language declares aliasing/ownership/borrow facts that constrain scheduling and mutation.
  - `ModuleDependsOn` — module/import edges for incremental rebuild and visibility.

The artifact does **NOT** carry:
- Cost, complexity, ownership *policy*, parallelism *decisions*, termination *requirements* — these are **lens outputs** (P3) computed downstream as side-channel folds over `InferredTree`. The compiler-core distinction: *semantic dependency facts* (above) are core; *scheduling and optimization decisions* derived from them are downstream.
- Target-specific anything — `translate` is parametric on `target_lang`.

**Why this distinction matters.** Parallelism, MapReduce-style sharding, CUDA placement, memoization, and incremental rebuild are all **derived** from `InferredTree`'s dependency graph (plus algebra-inhabitance witnesses for the proposed transformation). They are not special compiler modes and not target-specific code. Ownership-as-a-lens lives downstream; aliasing-as-a-semantic-fact (when required for correctness — e.g., to prove a `map` is safely parallelizable) lives on `InferredTree`'s ResourceDependsOn edges.

### `ModelCore` (shared substrate, factored per ratified Q1)

`ModelCore` is the shared base consumed by both `LanguageModel` and concrete runtime bundles. Declares:
- **Primitive types** (each a Practice-8 fact-bundle — width, signedness, range, encoding, …).
- **Algebra inhabitance declarations** (which primitives inhabit which algebras).
- **Laws / proof obligations** (associativity, commutativity, etc., for use by the coercion fold's preservation checks).
- **Effect semantics** (per Open Q10 — what effects each primitive operation declares).
- **Partiality semantics** (which operations are partial; how partiality is discharged or surfaced).

### `LanguageModel` (extends `ModelCore`)

Declared in `extdeps/languages/X.dag`. In addition to `ModelCore`:
- **Grammar** (bidirectional productions — concrete syntax ↔ Node).
- **Sugar dissolutions** (surface form → substrate Node).
- **Binding / namespace / scope rules** (lexical scope, imports, module boundaries, hoisting, shadowing rules). Consumed by `resolve`.
- **Serialization rules** (the inverse-grammar direction; see Open Q12 for the bidirectional contract).
- **Version / dialect / edition metadata** (Rust editions, Python versions, C++ standards, TypeScript config; see Open Q13).

### Runtime carriers (option C)

Abstract runtime carriers live in `std/runtime.dag`; concrete runtime bundles live in `extdeps/runtimes/*.dag`. In addition to `ModelCore`, a concrete runtime bundle provides:
- **Value representation** (how a substrate primitive is represented at runtime).
- **Primitive operation interpretation** (Transform → runtime function call; Branch → runtime branch choice; Value → runtime literal allocation).
- **Execution semantics** (call/return, allocation, control transfer).
- **Resource / effect boundary** (FFI, exceptions, async/concurrency model, identity/reference semantics).

The shared `ModelCore` ensures that algebra-inhabitance and primitive-type declarations are consistent between language and runtime targets. Runtime-side concerns (allocation, runtime values, resource limits) are categorically different from emit-side concerns (grammar, serialization) and live in the runtime carrier plus concrete runtime extdep, not in `LanguageModel`.

### `TargetSource` / `Value`

Terminal output. `TargetSource` is target-grammar-serialized text + diagnostics. `RuntimeValue` is the runtime representation + diagnostics.

---

## The EDIT direction — symmetric to compile

This doc is primarily about the **compile direction** (CoreNode → target). But the substrate is read/write symmetric: `apply_diff` is the **write-side derived combinator** (post-Pass B unification — `apply_diff = sequence_outcome(diff.edits, fn(edit, candidate) -> fold_node(candidate, substitute_at(edit.at, edit.replacement)))`, a sequential Outcome-bind-fold over the Diff's ordered Edits list with per-Edit `fold_node` substitution; **sequential semantics preserve the read/edit pipeline doc requirement that each later Edit's Path resolves against the post-prior-Edit state, fail-closed if any Path doesn't resolve in the intermediate candidate**). It mutates a `CoreNode` graph. The full **EDIT direction** is documented in [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) (PR #3364, merged 2026-05-19, operator-ratified); this section summarizes the architecture that compile must compose with.

### Read/edit vocabulary (substrate types + derived combinators)

Note: `apply_diff` / `subterm_at` / `read_lens` / `affected_set` are **derived combinators** post-Pass B unification (`fold_node` instances or compositions thereof), NOT primitives. Listed here as the amended vocabulary the read/edit pipeline exposes; their primitive-set status is per the main primitive table above (5 primitives total).

| Vocabulary | Purpose | Where |
|---|---|---|
| **`Path`** *(carrier type)* | A structural address to a sub-Node — `Path { steps: List<Symbol> }`. NOT a filesystem path; the Node graph's own coordinates. | `src/v4/std/node.dag` (ratified PR #3162) |
| **`Edit`** *(carrier type)* | A structural rewrite: `Edit { at: Path, replacement: Node }`. **Replacement only** — no separate insert/delete variants. Insertions/deletions decompose into parent-replacement. | `src/v4/std/node.dag` |
| **`Diff`** *(carrier type)* | An ordered sequential rewrite program: `Diff { edits: List<Edit> }`. Sequential composition, NOT parallel. | `src/v4/std/node.dag` |
| **`apply_diff(root, Diff) → Outcome<Node>`** *(derived combinator)* | `sequence_outcome(diff.edits, fn(edit, candidate) -> fold_node(candidate, substitute_at(edit.at, edit.replacement)))` — sequential Outcome-bind-fold over the Diff's ordered Edits list. Each Edit's Path resolves against the **intermediate** candidate (post-prior-Edit state), NOT the original root; fail-closed all-or-nothing if any Path doesn't resolve in the current candidate. Sequential semantics required per read/edit pipeline doc. | `src/v4/lens/application.dag` (scaffold; T-23) |
| **`subterm_at(root, Path) → Outcome<Node>`** *(derived combinator)* | `fold_node` instance — inverse of substitution; fetch the Node at a Path. | `src/v4/lens/application.dag` |
| **`read_lens(lens, node) → Reading`** *(derived combinator target)* | `fold_node` with the lens's algebra over any `Node`. The reading is data. Caller policy may inspect it and decide whether to continue; that policy is not part of the lens read. | Target replacement for `src/v4/lens/application.dag`'s current `apply_lens` gate surface. |
| **`affected_set(dag, Diff) → Witness<ReExecFrontier>`** *(derived combinator + specialization)* | `fold_node` over the dependency graph using the change-propagation algebra. Lens-frontier specialization of the general `AffectedSet` (per Ratified Q6 + Pass B unification — T-21 is a specialization, not a parallel primitive). | `src/v4/lens/affected_set.dag` (T-21) |

### The seven-step read→edit pipeline

Amended from [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) § 4, the agent loop is:

```
1. Read       →  read_lens(lens, node)
2. Diagnose   →  reasoning over the facts (LLM or human)
3. Propose    →  produce Diff
4. Candidate  →  candidate_result = apply_diff(dag, Diff) # Outcome; reject before readings if failed
5. Observe    →  candidate_dag = unwrap(candidate_result); read_lens(lens, candidate_dag)
6. Policy     →  caller/project policy evaluates readings over candidate nodes
7. Commit     →  dag := candidate_dag                     # only if policy accepts
8. Re-project →  project per projection plan              # files are a downstream effect
```

**The candidate-state pattern is structurally important.** Policy reads run against the **post-edit candidate state**, NOT the pre-edit graph. Reasoning: reading the pre-edit graph and applying the Diff afterward would let a Diff introduce a post-edit invariant violation that was never observed. The candidate pattern keeps the fail-closed promise honest: validation happens against the state that will be committed/emitted, not against a state already known to be valid.

The root Node passed to each read binds the candidate-vs-pre-edit context structurally — a worker can't accidentally validate against the wrong root by relying on ambient scope.

**This is the same monotonic-facts invariant** as the compile direction's Practice 3 (P3 stage-by-stage facts), but applied to mutation: facts flow forward through the candidate; the commit either lands all of them or none.

### EDIT direction = the "Query-driven rewrite" ingest path

In the "Ingest paths" table earlier, **"Query-driven rewrite"** — using `affected_set + apply_diff` to produce a new `CoreNode` — IS the read/edit pipeline. The compile direction takes the resulting CoreNode and proceeds normally. From the compile direction's view, query-driven rewrite is just another entry point producing a CoreNode; from the read/edit doc's view, it is the canonical agent workflow.

### Library-first agent surface

Amended from [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) § 6.10:

1. **All substrate primitives and derived combinators surface as library functions.** `read_lens`, `apply_diff`, `subterm_at`, `affected_set`, `declarations_in` are library calls. Not RPC services. Not CLI-first.
2. **CLI wraps libraries.** `gunbc read-lens ...`, `gunbc auto-fix ...`, `gunbc refactor ...` are thin shells over library calls.
3. **The library boundary IS the agent-substrate surface.** No premature transport layer (JSON-RPC, gRPC). Network transport is a future concern that doesn't pre-date library landings.

**This applies to graph consumers too.** `ground`, `observe`, `project`, and `authorize` are library calls. The same no-premature-transport principle holds — compiler/session operations are libraries, not services.

### Lens as `(find, transform)` — the convolution view

Per the read/edit doc § 6, every L1.x dissolution lens is structurally a `(find, transform)` pair: the lens's Signature is the find half, the Clean shape is the transform half. The read/edit doc's § 6.7b "**(f) mechanical refactor**" hero case is the worked example most relevant to this compiler architecture: a single intent → uniform per-site transform → atomic apply via the candidate-state pattern. PR #3338's canonical-B refactor (across 6 languages + 7 ratchet dissolutions) is the worked example. This is also what compiler-source rewrites (e.g., self-modification, see below) decompose into.

### Artifact projections and lens readings

Per P0 + P3, **every downstream artifact is a projection of `InferredTree`**, but artifact projection is not lens ownership. The compiler core produces `InferredTree`; `observe` runs lenses and returns readings, while `project` runs projection producers and returns artifacts. Naming things by their function rather than historical accident:

| Consumer | Producer / Reading | Output | Notes |
|---|---|---|---|
| Target source code | projection producer: `translate(target_lang)` + `serialize` | `TargetSource` artifact | The compile direction uses the homomorphism (P1) inside a projection producer; it is not a lens reading. |
| Runtime evaluation | projection producer: `eval(runtime)` | `RuntimeValue` / eval artifact | Same homomorphism, terminal action = execute (P2). |
| Dimensional facts (cost / complexity / ownership / parallelism / termination / idempotency / effect) | lens family | `DimensionFact` readings | All consume `InferredTree`; produce side-channel readings (P3 commitment 3). |
| **Test cases** | projection producer fed by test-claim readings | `TestClaim` / `TestCase` artifacts | Test generation may consult lens readings, but artifact materialization belongs to `project`, not to the lens read itself. Tests are NOT the source of truth for the model; they are behavioral probes generated FROM the model. See "Testgen as a projection family" below. |
| **Dry-run / simulated execution** | projection producer: `io_boundary` reading + `LawfulRewriteWitness` substitution + `eval(mock_runtime)` | `RuntimeValue` / dry-run artifact | Dry-run is composition of graph reading, lawful rewrite, and eval projection. It is not a new compiler primitive and not a lens-owned artifact. |
| Inter-module glue (REST client/server, marshaling, schemas) | projection producer over shared transport model (P4) | per-module `TargetSource` + glue artifacts | The omni-stack story. Cross-link to P4. |
| Migration / API-compat plans | projection producer using diff-over-trees + affected-set fold | structural change report + per-site remediation | Change-consequence story; uses `affected_set` (T-21). |
| Affected-rebuild set | `affected_set(dag, Diff)` reading | `Witness<ReExecFrontier>` | Incrementality reading; consumed by projection regeneration. |
| Documentation / API docs | documentation projection producer | structured doc data + rendered text | A projection like any other; not a separate tool. |

The compiler core does NOT name any of these specific projections or lenses (P3 commitment 4). Lenses are plug-in graph readings consumed by caller policy or projection producers; artifact producers belong to the projection registry and report through `ProjectionReport`.

### Testgen as a projection family

The current name `lens/testgen.dag` is historical — predates the projection split. Conceptually, **testgen is a projection family that may consume lens readings while producing test artifacts from `InferredTree`.** Different "levels" of testgen are different projection profiles or dependent readings over the same verification substrate.

**Three-layer structure** (per reviewer 2026-05-20):

| Layer | Consumer | Input | Output |
|---|---|---|---|
| **Abstract claims** | `TestClaimProjection` | `InferredTree` | `TestClaim` Witnesses — roundtrip-law claims, refinement-boundary claims, algebra-law claims, protocol-compatibility claims, effect/idempotency claims |
| **Concrete cases** | `TestCaseProjection` | `TestClaim` Witnesses | `TestCase` Witnesses — examples, boundary cases, property-test generators, fuzz seeds, regression fixtures (lowering of abstract claims) |
| **Target-specific files** | `TargetTestProjection` | `TestCase` Witnesses + target `LanguageModel` | `TargetSource` for tests (Rust `#[test]`, Python pytest, TS Jest, integration harness, …) — uses translate's homomorphism applied to test data |

The pipeline:
```
InferredTree ── TestClaimProjection ──> TestClaims
TestClaims    ── TestCaseProjection ──> TestCases
TestCases     ── TargetTestProjection(target_lang) ──> target test source
```

Each step is a declared projection over grounded graph data or prior projection output; each step's output is data consumable by the next.

**Profile invocations** are different projection parameters / dependent readings on the same family:

| Profile | Reads | Example |
|---|---|---|
| `testgen.smoke` | typeshape + binding | "every public function has a smoke test that calls it with default-typed args" |
| `testgen.boundary` | refinement predicates declared on types | `Refined<String, uuid_format>` → "valid UUID test, invalid UUID tests" |
| `testgen.property` | algebraic laws declared on inhabitances | `OrderedRing<Int>` → "addition is commutative", "addition has identity" |
| `testgen.algebra_law` | algebra-law-subject declarations | "associativity holds for this monoid" |
| `testgen.effect` | `EffectDependsOn` edges + idempotency lens output | "idempotent service `f` called twice produces same result" |
| `testgen.integration` | protocol models + service boundaries | `service.via rest::post(path)` → "client/server contract roundtrip" |
| `testgen.roundtrip` | grammar-as-bidir-data law | serialize-then-parse equals identity for the target language |
| `testgen.equivalence` | cross-target inhabitance witnesses | "Rust + Python + TS emit produce equivalent results" |
| `testgen.fuzz` | cardinality + disjunction boundaries | "for each variant of this Disj type, generate a sample" |
| `testgen.regression` | prior failing inputs (from `TestClaim` registry) | "this past failure must still pass" |

All profiles share the **same substrate**:
- `std/verification.dag` (TestClaim carriers) — landed.
- `std/refinement.dag` (refinement substrate, T-25-core) — landed.
- `std/dependency.dag` (P6 dep edges) — not yet declared.
- `std/testgen.dag` (projection family library) — exists as `lens/testgen.dag`; pending the rename to fit the projection-family naming convention.

**Projection dependencies** are first-class: `testgen.property` may depend on `testgen.algebra_law`'s output; `testgen.integration` may depend on `testgen.roundtrip`. These compose through projection dependencies and any required lens-reading dependencies, not via ad hoc calls — Practice 10 row 1 still applies to any lens internals they consume.

**Important boundary (P0 implication):** tests are projections OF the model; the model is NOT derived from tests. Tests are **behavioral probes** generated from the source-of-truth model. The slogan: *"tests are generated from the model as behavioral probes; they are not the source of truth for the model."*

**Self-regeneration:** when a model fact changes (e.g., `Refinement<T>` shape changes per P8's worked example), `affected_set` computes which TestClaims/TestCases/target test files need regeneration; the relevant projection producers re-fire automatically on the affected Node frontier. No manual sync.

### Dry-run as a projection over graph readings

**Dry-run / simulated testing** — running a program without committing its I/O side effects — is NOT a new compiler concept. It's the composition of three primitives we already have:

1. **A lens identifies the I/O boundaries.** `io_boundary_lens(InferredTree) → Witness<List<IOBoundary>>` — fold over the tree, find every Node whose inhabitance involves an effectful primitive (network call, DB write, file I/O, time, randomness, host syscall). The lens reads `EffectDependsOn` / `ResourceDependsOn` edges from P6 to determine what counts as I/O.
2. **A `LawfulRewriteWitness` substitutes mock stubs.** The substitution is target-declared: `dry_run.dag` (a runtime/policy model) declares "for each I/O primitive in the source's effect algebra, the rewrite produces a mock-host call that records-and-returns a canned value." The rewrite is witnessed by a "this preserves observable semantics modulo committed I/O" law per P7.
3. **Eval with a mock runtime.** `eval(mock_runtime)` runs the rewritten tree. The mock runtime's interpretation of the substituted operations records and returns mock values; nothing actually hits the real I/O surface.

```
InferredTree
  ── io_boundary_lens(Introspect) ──> List<IOBoundary>
  ── LawfulRewriteWitness(dry_run substitution) ──> InferredTree' (with stubs)
  ── eval(mock_runtime) ──> RuntimeValue + recorded_io_log
```

No new compiler primitive needed. Dry-run is a **projection over graph readings**: a lens identifies, rewrite substitutes, eval runs against a different host.

**The operator's intuition was right** — the actual interception happens via codegen-style substitution (the `LawfulRewriteWitness` produces an InferredTree with stub nodes where the I/O primitives were), not "the lens itself intercepts at runtime." Lenses are pure folds; they cannot intercept execution. But they CAN identify what needs intercepting; a rewrite witness does the substitution; eval runs the result.

**Variations** (different lens invocations / different mock runtime):
- `dry_run.record` — execute, log every would-be I/O, return mock results.
- `dry_run.assert` — execute against pre-declared expected I/O sequence; diagnostic if mismatch.
- `dry_run.replay` — execute with prior I/O log as inputs (deterministic replay).
- `dry_run.fuzz` — execute with random / boundary mock values; collect outcomes.

Each is a different mock runtime + optionally different lens for what counts as I/O (e.g., only network I/O vs all I/O). The compiler core machinery is unchanged.

### How tests and dry-run compose

`testgen` produces TestClaim Witnesses; `dry_run` provides a sandboxed execution of an InferredTree. A natural composition: `testgen` generates a set of TestClaim cases; `dry_run.assert` executes each against the implementation; failures become diagnostics. This is what the THESIS Tier 3 ("verification from structure") + the v4 `verification.dag` substrate enable — but composed from existing primitives, not as a special "test runner" subsystem.

---

## Self-modification, stage0, self-edit

> **Important framing (per P9):** the phrase "compiler editing itself" is loose. The precise framing is: the compiler **models** itself; the compiler **generates a candidate next-compiler** as an artifact; the candidate is **verified**; only the **promotion protocol** replaces the active stage0. The active compiler does NOT mutate itself in place at runtime. See P9 ("Bootstrap is stratified") for the full discipline; this section walks through the mechanics.

The compiler is itself authored in `.dag`. Self-modification — the compiler updating its own source — is therefore a natural composition of the EDIT direction (read/edit pipeline), the COMPILE direction (this doc), AND the bootstrap promotion pipeline (P9). This section addresses the specific question the read/edit doc reviewer raised: "how will the compiler edit itself?"

### The compiler reading its own source

The compiler's own pipeline (`compile.dag` / `parse.dag` / `infer.dag` / `emit.dag` / `eval.dag` / `normalize.dag` / `resolve.dag`, all in `src/v4/compiler/`) is itself a set of `.dag` programs. To the substrate, they are source values that first ground into `InferredTree` graphs like any other program. The compiler reads them by observing the grounded graph. Reading the compiler's own source uses the **same graph-reading surface** as reading any user program — there is no special introspection mechanism.

### The compiler writing to its own source

When a `.dag` substrate type changes — say `std/node.dag`'s `NodeFold<R>` carrier gains a new field — that change propagates through the compiler's own `.dag` source. Two valid mechanisms, both go through the read/edit pipeline:

1. **Hand-authored Diff via the read/edit pipeline.** The operator (or an agent) declares the change as a `Diff` against the substrate-typed Node graph. `apply_diff` mutates; the seven-step candidate-state pattern validates against the candidate. Commit lands. This is exactly the read/edit doc's "(f) mechanical refactor" hero case applied to the compiler's own source.
2. **Mechanical refactor from upstream change.** When a substrate type evolves, a lens (e.g., an L1.x dissolution lens, or a custom interface-cascade lens like read/edit doc § 6.5) computes the affected sites + the per-site transform. The substrate handles N affected sites in the compiler's own code the same way it handles N user-code sites. Atomic apply via candidate-state.

There is no "self-edit special case." Self-edit is `apply_diff(self, Diff)` where `self` is the compiler's own `CoreNode` source value before grounding. The substrate's read/write-symmetric mechanism (`fold_node` reads; `apply_diff` writes via sequential `sequence_outcome` over `Diff.edits` with per-Edit `fold_node` substitution against the intermediate candidate) means there's no hidden machinery — same primitive set, applied to the compiler's source.

### stage0 regeneration as a projection of self

The hand-Rust-to-zero trajectory (per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md)) requires the compiler to **emit its own Rust bootstrap**. Structurally this is a projection over the compiler's grounded graph, expressed as the composition of `ingest_text` + `ground` + `project` per the text/data separation:

```
// Explicit composition: ingest is separate from grounding/projection.
self_corenode_result = ingest_text(
  input_text: read_file("compiler.dag"),    // boundary action: read text
  input_lang: dag_lang                       // grammar + sugar dissolutions
) -> Outcome<CoreNode>
if self_corenode_result is Rejected:
  reject with diagnostics

self_corenode = unwrap(self_corenode_result)

compiler_grounding = ground(
  plan: IngestionPlan {
    subject: self_corenode,
    producer: ingest_explicit_language,
    params: dag_language_params
  }
) -> Outcome<InferredTree>
if compiler_grounding is Rejected:
  reject with diagnostics

compiler_graph = unwrap(compiler_grounding)
bootstrap_lens_report = observe(
  inferred: compiler_graph,
  plan: bootstrap_lens_plan
) -> LensReport

stage0_projection_report = project(
  inferred: compiler_graph,
  plan: rust_stage0_projection_plan
) -> ProjectionReport

stage0_terminal = authorize(
  policy: stage0_terminal_policy,
  lenses: bootstrap_lens_report,
  projections: stage0_projection_report
) -> Outcome<Validated<ArtifactSet>>
if stage0_terminal is Rejected:
  reject with diagnostics

stage0_artifacts = unwrap(stage0_terminal)

stage0_promotion = promote_stage0(
  candidate: stage0_artifacts,
  gate: p9_promotion_gate
) -> Outcome<PromotionWitness>
```

There is **no special "regenerate stage0" mode** and **no parse-and-normalize folded back into projection**. The pipeline is the same uniform composition as any other session: `ingest_text` produces the CoreNode (at the system boundary), `ground` produces the inferred graph, and a projection producer contributes a Rust-source artifact entry to `ProjectionReport`. The homomorphism mechanic applies inside the projection producer: coercion-fold against `rust.dag`'s declared inhabitants; serialize via Rust's grammar. The Rust bootstrap is usable only after terminal authorization produces a `Validated<ArtifactSet>` and the P9 promotion gate accepts the candidate.

**Equivalent paths** for stage0 regeneration (per P9's stratified bootstrap):
- `compile.dag` already grounded as `InferredTree` from prior ingest (cached) → `project(self_graph, rust_stage0_projection_plan)` yields a `ProjectionReport` containing the candidate stage0 artifact; `authorize(...)` and the P9 promotion gate decide whether it can replace the active seed.
- Programmatic compiler-model construction (e.g., from a `Diff`-derived candidate) → `ground(candidate_ingestion_plan)` then `project(candidate_graph, rust_stage0_projection_plan)` yields the same report-shaped candidate artifact; no source is usable before terminal authorization and promotion.

All paths go through `ground` to produce the graph, then through the requested projection producer, then through terminal authorization / P9 promotion before the generated Rust can be treated as a bootstrap replacement. The ingest boundary is explicit and separable.

This is why P1 (Languages are I/O integration surfaces) and self-hosting are not independent concerns: self-hosting is the compiler grounding its own source into a graph and projecting a bootstrap artifact from that graph, with both sides using the same `LanguageModel` substrate. The trajectory of hand-Rust-to-zero is the same graph-consumer loop closing on itself.

### Candidate-state for self-modification safety

When the compiler proposes a change to itself, the candidate-state pattern from the read/edit pipeline is what makes self-modification **safe** — and per P9, this is the *only* path: candidate generation + verification + promotion. The active running compiler is never the candidate.

```
candidate_result = apply_diff(self, proposed_diff)
if candidate_result is Rejected:
  reject with diagnostics

candidate_self = unwrap(candidate_result)
candidate_ingestion_plan = IngestionPlan {
  subject: candidate_self,
  producer: self_edit_candidate,
  params: self_edit_params
}
candidate_grounding = ground(candidate_ingestion_plan)
if candidate_grounding is Rejected:
  reject with diagnostics

candidate_graph = unwrap(candidate_grounding)
readings = observe(candidate_graph, self_edit_lens_plan)
if project_policy_accepts(readings):
  self := candidate_self     // atomic commit
  // optionally: project(candidate_graph, rust_stage0_projection_plan)
  //             -> authorize(...) -> P9 promotion gate before stage0 use
else:
  reject with diagnostics
```

Self-modification can never produce a broken substrate because the candidate is grounded before readings or policy run, and structural invariants should move into types wherever possible. If a self-edit would break Practice 10 / Practice 9 / a substrate invariant, grounding rejects or candidate-graph readings expose it and project policy rejects the commit. **The compiler cannot break itself silently** — the candidate-state pattern makes the checked grounded state explicit.

### Implications

- **No new substrate is needed for self-edit.** `apply_diff` + lens readings + caller policy are the mechanism. The compiler's `.dag` source is just data; the same primitives that handle user data handle compiler data.
- **Self-hosting is one specific projection over the grounded compiler graph.** `ground(self_ingestion_plan)` produces the graph; `project(self_graph, rust_stage0_projection_plan)` produces a `ProjectionReport` with a candidate stage0 Rust artifact; `authorize(...)` yields the `Validated<ArtifactSet>`; the P9 promotion gate decides whether it replaces the active seed. `project(self_graph, host_eval_projection_plan)` evaluates the compiler on the host through the same report boundary. Other target/compiler artifacts are additional projection requests, not separate compile modes.
- **The "Rust shrinks to zero" trajectory IS the loop closing.** When stage0 Rust is reliably regenerable from `ground(self_ingestion_plan)` + `project(self_graph, rust_stage0_projection_plan)` + `authorize(...)` + P9 promotion, the hand-maintained surface shrinks; per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md), the target is zero hand-authored Rust.

---

## Concrete example — what happens when this compiler compiles

To make the architecture concrete, here's a tiny `.dag` source going through the pipeline. (Syntax is illustrative; not all surface sugar is finalized.)

**Source (`example.dag`):**
```dag
fn add(x: Int, y: Int) -> Int = x + y
```

**Stage 1 — `parse(input_text, dag_lang)`** uses `extdeps/languages/dag.dag`'s grammar productions to recognize `fn`, identifiers, types, and the `+` expression. Output: a `ParseTree` Node:
```
Arrow {
  name: add
  params: [Atom("x"): Atom("Int"), Atom("y"): Atom("Int")]
  result: Atom("Int")
  body: Apply(Atom("+"), [Atom("x"), Atom("y")])
}
```

**Stage 2 — `normalize`** dissolves `fn` sugar to substrate `Arrow` (already mostly canonical here), dissolves `+` to `Apply(FunctionRef("std::int::add"), …)` per dag-language's sugar dissolutions. Output: same shape with surface sugar gone.

**Stage 3 — `resolve`** binds `Atom("Int")` to `std/integer.dag`'s `Int` declaration; binds `FunctionRef("std::int::add")` to its declaration. Output: every Atom now points to a Decl Node.

**Stage 4 — `ground` (T-9)** walks the tree and computes each Node's algebra-inhabitance:
- `Int` inhabits `OrderedRing` (algebra-instance witness recorded).
- `x` and `y` each inhabit `Int` (via the param type).
- `std::int::add` is the `add` operation projected from `OrderedRing<Int>`.
- The whole `add` function inhabits `Arrow<Int, Int, Int>`.

Output: `InferredTree` — same Node shape as resolve's output, but each Node now carries its inhabitance witness.

**Stage 5a — `translate(rust_lang)` (T-10)** runs the **coercion fold** over (`InferredTree`'s canonical grounding, `rust.dag`'s declared inhabitants):
- Source `Int` inhabits `OrderedRing` → look up Rust's inhabitants of `OrderedRing` → finds `i32`, `i64`, …
- Picks one by the source's width fact (or fails closed if ambiguous — diagnostic).
- Source `std::int::add` → Rust's `+` operator on the chosen integer type.
- Source `Arrow` → Rust's `fn`.

Output: a `TargetNodeTree` in Rust's grammar:
```
Rust::Fn {
  name: add
  params: [x: i32, y: i32]
  return: i32
  body: Rust::BinOp(Add, Rust::Ident(x), Rust::Ident(y))
}
```

**Stage 6 — `serialize(rust.dag.grammar)`** walks the Rust Node tree using rust.dag's grammar productions in the inverse direction:
```rust
fn add(x: i32, y: i32) -> i32 { x + y }
```

**OR Stage 5b — `eval(runtime)` (T-22)** would, instead of translating to Rust, walk the InferredTree with the runtime interpretation algebra:
- `Arrow` becomes a runtime closure.
- `std::int::add` becomes the runtime add.
- Calling `add(2, 3)` produces a `RuntimeValue`.

**Failure modes (fail-closed at every stage):**
- `parse` fails → text doesn't match grammar → `Diagnostic(SyntaxError, locus)`.
- `resolve` fails → unresolved Atom → `Diagnostic(UnresolvedSymbol, locus)`.
- `ground` fails → no algebra-instance found → `Diagnostic(UngroundedType, locus)`.
- `translate` fails → coercion-fold finds no target inhabitant matching source's structure → `Diagnostic(NoTargetInhabitant, locus)` (THESIS's Tier 1 grounding completeness).

Every failure carries source position. No silent drops. No partial outputs.

---

## What we have today (relevant substrate)

> **Verifiability note (per codex review 2026-05-20).** Each row below lists the file path + the substrate carriers it owns. To verify any row independently: `ls src/v4/...` confirms file existence; `head -10 src/v4/<file>.dag` shows the file's status header (each `.dag` file declares its own status per Practice 9's four-line header rule). Bot reviews flagged this as "verify against the v4 tree" — the rows below are accurate as of the last sweep; if a row drifts from main, this paragraph is the explicit pointer for how to re-verify.

- `src/v4/std/node.dag` — `Node`, `Edge`, `fold_node`, `NodeFold<R>` algebra carrier, plus `Path` / `Edit` / `Diff` vocabulary (PR #3162). **Foundation in place.**
- `src/v4/std/algebra.dag` — algebra structures (Magma, Monoid, …) — the inhabitance authority.
- `src/v4/std/cardinality.dag` — inhabitance + bounded-natural refinement + descent evidence.
- `src/v4/std/diagnostic.dag` — `Diagnostic` + `Locus`, fail-closed reporting.
- `src/v4/std/refinement.dag` — base-type + fail-closed validation substrate (T-25-core, landed via #3354 — header status: "T-25-core modeled").
- `src/v4/std/verification.dag` — `TestClaim` substrate (eval's downstream consumer).
- `src/v4/extdeps/languages/dag.dag` + `rust.dag` + `python.dag` + `go.dag` + `cpp.dag` + `typescript.dag` — Wave-1 LanguageModels with Practice-8 fact-bundles.
- `src/v4/compiler/01_tokenize.dag`, `02_parse.dag`, `03_normalize.dag`, `03_resolve.dag` — pipeline scaffolds; T-8 implementation in flight (consolidated as PR #3436).
- `src/v4/compiler/04_infer.dag`, `05_emit.dag`, `05_eval.dag`, `00_compile.dag` — scaffolds with mixed reconciliation state — see "Scaffold-vs-design audit" below. `00_compile.dag` currently carries the now-superseded `validate_then_compile` / `Validated<T>` gate and `CompileMode`; `04_infer.dag` no longer imports a specific lens; stage bodies remain scaffolded / partial.
- `src/v4/std/artifact.dag` — narrow artifact identity substrate (`ArtifactKind`, `Artifact`, `NodeArtifactProvenance`); not yet the full P8 regeneration substrate.
- `src/v4/lens/application.dag` — `apply_lens` + `apply_diff` operations (scaffold; T-23).
- `src/v4/lens/affected_set.dag` — incremental-rebuild + lens-frontier specialization (T-21).

Falsification probes (not load-bearing for the initial homomorphism):
- `src/v4/extdeps/languages/` — verilog, llvm_ir, machine_code, ptx, lean (probes the 5-behavior thesis across heterogeneous domains).
- `src/v4/extdeps/formats/spice.dag` — physics simulation probe (no control flow).

## Scaffold-vs-design audit — implementation migration plan

The compiler scaffolds in `src/v4/compiler/` were authored across several ratification waves. Some formerly stale interfaces are now reconciled; others remain staged. Implementation workers should use the **ratified contract** column as the authority, and treat any still-staged file as a scaffold only where the table says so. A live audit receipt for the 2026-05-21 sweep is recorded in `docs/audit/v4-compile-lens-artifact-orthogonality-2026-05-21.md`.

| Scaffold | Current tree state | This doc's ratified contract | Status / migration |
|---|---|---|---|
| `00_compile.dag` `validate_then_compile` | `validate_then_compile(source, input_lang, lenses, mode) -> Outcome<Validated<CompileOutput>>`; manual TestClaims assert empty-lens bypass and rejecting-lens blocks. | Session terminal: `ground(IngestionPlan) -> observe -> project(ProjectionPlan) -> authorize`, with `authorize(...) -> Outcome<Validated<ArtifactSet>>`. Lens readings feed policy, not compile-core. | **Conflict in ownership/framing.** Keep `Validated<T>` as terminal-policy carrier, but move ownership out of compile-core and replace scalar language/mode inputs with `IngestionPlan`, `LensPlan` / `LensReport`, and `ProjectionPlan`. |
| `00_compile.dag` `CompileMode` | `CompileMode = TranslateTo { target } | Eval { runtime }`, passed to `compile` and `validate_then_compile`. | Projection registry: `ProjectionRequest { producer: ProjectionProducerRef, subject, params, requirement }`. No closed compile/production enum. | **Conflict.** Split compile-core from projection planning; replace `CompileMode` with projection requests in the session layer. |
| `00_compile.dag` legacy ingest | `Source = String` survives only for `compile_ingest_staging`. | Text is a peripheral `ingest_text` shim producing `CoreNode`; `ground(IngestionPlan)` consumes that input and returns the grounded graph consumed by lenses and projections. | **Staged.** Dissolve when `ingest_text` lands as a peripheral shim. |
| `04_infer.dag` | No `v4.lens.*` import. `InferredFacts` carries `resolved_type`, `inhabits`, `descent`, and `canonical`. | No lens import in compiler core (P3 commitment 4). Cost and other dimensions are lens outputs over `InferredTree`, not infer facts. | **Lens-import violation resolved.** Keep this invariant: compiler core must not name a specific lens. |
| `04_infer.dag` | `InferredFacts.canonical` carries a `CanonicalGroundingWitness`; the full P6 semantic dependency graph is not yet present on `InferredTree`. | Must produce *exactly one* canonical grounding per Node + the full P6 semantic dependency graph (`BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn`) | Canonical witness carrier is staged; finish producer semantics and add the P6 dependency graph facts. Surface ambiguity as a diagnostic in ground; translate never receives ambiguous source. |
| `05_emit.dag` | `emit = serialize_target ∘ translate`; `serialize_source_for_emitted` is a fail-closed equality/stub pending inverse grammar walk. | Translate by exact coercion fold, then serialize via target grammar. P7 `LawfulRewriteWitness` is the future extension point for structure-changing lowerings. | **Staged, shape aligned.** Replace the emitted-tree equality stub with grammar-as-bidirectional-data serialization when C5 inverse grammar walk lands. |
| `05_eval.dag` | Standalone "primary execution path" with TestClaim runner | `eval(runtime)` — structurally `translate-to-runtime` per P2; differs from translate only in the terminal action (execute, not serialize) | Share the homomorphism mechanic with translate. Eval-specific machinery is only the runtime interpretation algebra + the terminal `execute` action. |
| `lens/application.dag` | `apply_lens(lens, SectionRef, Introspect/Enforce)` is framed as the only advisory-to-fail-closed bridge; `SectionRef = DeclarationScope | NodeScope`. | `observe(inferred, LensPlan) -> LensReport`; lens applications are data-producing reads. Authorization owns gating. | **Conflict.** Reframe `apply_lens` as graph read; move `Enforce` semantics to terminal policy. |
| T-12/T-13 lens family | Cost/complexity headers already say `Node -> Witness<...>`; effect/parallelism/idempotency implementations currently take `InferredTree` plus dependency lists in helper functions. | Lens consumers take the grounded graph (`InferredTree` / future `InferredGraph`) and return typed readings. | **Mixed.** Preserve graph-read direction; distinguish proof witnesses from lens readings. |
| `affected_set.dag` / TASKS prose | Several carriers and comments name changed "subgraphs." | Shared low-level `RegionRef`; refined `LensScope` / `ProjectionSubject`; no universal `SubgraphScope`. | **Terminology/audit follow-up.** Avoid introducing a universal subgraph carrier; existing names are local frontier vocabulary until reshaped. |
| `std/artifact.dag` | Declares `ArtifactKind`, `Artifact`, and `NodeArtifactProvenance`. | Artifact identity exists as a narrow substrate; P8 regeneration still needs unified `Projection` / general `AffectedSet` / `RecomputePlan`. | **Declared narrow, not complete.** Do not claim full regeneration substrate until the unified cluster lands. |

**This is not a STOP on T-9 / T-10 / T-22 implementation** — workers implementing those tasks should land the design-ratified contract, not preserve any staged placeholder against this doc. Where a row is marked resolved, do not reintroduce the old stale shape.

If a worker encounters a deeper scaffold-vs-design contradiction not catalogued above, **escalate** — don't preserve the scaffold against this doc, and don't break the scaffold without recording the migration.

---

## MVP — single endpoint

**The MVP is one endpoint, not three (rescoped 2026-05-20 — strict amendment per P2 boundary discipline; multiple valid terminuses for one boundary was a soft framing):**

> Given an `IngestionPlan` for a small `.dag` program (the canonical `add(x, y) = x + y` worked example), `ground(plan)` produces the canonical `InferredTree`, and `project(inferred, rust_projection_plan)` produces an artifact containing valid Rust source code for the equivalent program via the homomorphism mechanic. The caller may then pass the projection result through `authorize(...)` before treating it as a releasable artifact set.

**Required substrate + implementation for MVP** (post-Pass B unification — all derived operations are `find_witness` or `fold_node` invocations with specific algebras-as-data):
- `std/dependency.dag` (the **lens-over-substrate** dependency substrate per sharpened P6: `DependencyKind` classification taxonomy + `dependency_lens` fold algebra + `DependencyView { source, dependent, kind, usage_site }` lens-output type. **NO** authored `DependencyEdge` / `DependencyReason` / `DependencyGraph` carriers — the substrate's usage graph IS the dependency graph; the lens classifies and emits Node references back to substrate, never duplicates substrate facts as parallel payload. — Worker A).
- `std/find_witness.dag` (unified primitive declaration — generic; carries the `find_witness` operation signature, the `Outcome<(Candidate, Witness)>` shape, AND the `MultiplicityPolicy` carrier — the substrate-level authority on multiplicity resolution). **MVP variants of `MultiplicityPolicy`: `UniqueOnly | TargetSelection(TargetSelectionPolicy)` only** — the two variants with consumers in MVP scope. The `RewritePolicy(...)` variant is **NOT in MVP** — it lands together with P7 lawful-rewrite work per the P7 deferral; adding it ahead of the consumer would create an untracked premature substrate surface (per openai-pro tracked-debt finding 2026-05-20). The two predicate algebras + their per-caller policy bindings live in their consuming concept-homes per M10: **constraint-satisfaction predicate + `MultiplicityPolicy::UniqueOnly` binding in `std/constraints.dag`**, **exact-structural-equality-zip-fold predicate + `MultiplicityPolicy::TargetSelection(target_lang.target_selection_policy)` binding in `std/coercion.dag`**. Single authority per concept: primitive + `MultiplicityPolicy` type in find_witness.dag, predicates + per-caller policy bindings in their respective consuming files — NOT all lumped into find_witness.dag. — Worker B authors all three files; `RewritePolicy` is added at P7 trigger, not MVP.
- Derived combinators: `traverse_node` / `sequence_node` / `bind_outcome` as `fold_node + Outcome-threading-algebra` (in `std/node.dag` extension; single-authority combinator-owns-policy per Worker C `NodeOutcomeFold.step` fix — Worker C).
- `W-T-9-impl` (ground stage body — invokes `find_witness` with constraint-satisfaction predicate).
- `W-T-10-impl` (translate body — invokes `find_witness` with exact-structural-equality predicate; plus serialize via grammar-as-bidir-data).
- Outcome-shape migration (43 bootstrap/lens/compiler callsites — Worker C corrected scope).
- Scaffold reconciles per the migration plan: introduce the session four-phase split, reframe `Validated<T>` as terminal authorization over `ArtifactSet`, move lens enforcement to policy, replace `CompileMode` with projection registry requests, finish canonical/dependency facts in `04_infer`, land inverse grammar serialization in `05_emit`, integrate eval-to-host as a projection producer, and dissolve legacy ingest staging.

## Migration order — orthogonality reshape

1. **Name the public graph carrier.** Treat current `InferredTree` as the grounded graph carrier; until an explicit rename lands, `InferredTree`, `InferredGraph`, and "grounded graph" references in this design name that same carrier. Decide whether the implementation rename to `InferredGraph` waits until P6 dependency facts are explicit.
2. **Introduce session reports.** Add design/source follow-up for `IngestionPlan`, `LensPlan`, `LensReport`, `ProjectionPlan`, `ProjectionReport`, `TerminalPolicy`, `ArtifactSet`, and `CompileSessionResult`.
3. **Reframe validation.** Keep `Validated<T>`, but move it to `authorize(policy, lens_report, projection_report) -> Outcome<Validated<ArtifactSet>>`. Retire `validate_then_compile` as compile-owned lens gating.
4. **Replace `CompileMode`.** Add projection requests with `ProjectionProducerRef` registry entries and typed params; do not add a larger closed enum.
5. **Refine scopes by concern.** Add/shared low-level `RegionRef`, then refine into `LensScope` and `ProjectionSubject` according to each consumer's constraints.
6. **Prefer type-level invariants.** For checks like unused-parameter validity that can become refinements on `InferredTree` or earlier carriers, model the invariant directly. Keep measurement/report concerns as lens readings.

**Out of scope for MVP** (deferred to follow-on waves; explicit defer with trigger):
- **Eval / runtime interpretation** — `std/runtime.dag` + `extdeps/runtimes/*.dag` + W-T-22-impl + W-EvalShim. Defer trigger: after MVP lands, when first end-to-end execution case (not codegen) is needed.
- **File-to-file shim** — `extdeps/file_system.dag` refresh + W-EmitShim. Defer trigger: when MVP's `ground(test_ingestion_plan) -> project(test_graph, rust_projection_plan) -> authorize(...)` round-trips clean, file-to-file convenience is a small follow-on.
- **Multi-target translate** — Python/Go/TypeScript/C++ targets. Defer trigger: after Rust MVP, language-by-language follow-on.
- **Lawful rewrites + structure-changing lowerings** — `LawfulRewriteWitness` substrate (P7). Defer trigger: first MapReduce / CUDA / parallel-map case.
- **`LensAlgebra<F>` carrier shape + composed-lens-algebra construction** (the substrate work Ratified Q6 names; this is NOT a "multi-lens primitive" — Q6 explicitly ratifies that there is no new substrate primitive, since multi-lens execution derives from `fold_node` with a composed-lens-algebra). Defer trigger: first multi-lens scheduling case beyond a single lens.
- **Bootstrap promotion + stage0 candidate generation** (P9). Defer trigger: first `project(self_graph, rust_stage0_projection_plan)` invocation when source-of-truth migration to `.dag` is in scope.

**Why one MVP and not three:** the homomorphism mechanic is the load-bearing claim. One end-to-end demonstration of it (IngestionPlan → InferredTree → projection artifact → authorized artifact set) validates the canonical-grounding and release boundary; adding shims and eval routes BEFORE the core mechanic works is premature scope. P2 boundary discipline: name the single MVP terminus, demonstrate it, then extend.

## What's NOT in scope for this design

- **Glue derivation / omni-stack** — orthogonal substrate (P4). Architecture must not preclude, but no implementation in the initial single-target compiler.
- **`extdeps/protocols/`** — missing substrate; deferred until glue is in scope.
- **Lens framework implementation** — out of initial compiler-core scope. The lens consumption contract is **ratified by P3 + Ratified Q0**; the `LensAlgebra<F>` + composed-lens-algebra substrate work is ratified by Q6 (NOT a separate primitive; derives from `fold_node`).
- **Per-target emitters** — does not exist (P1). Each new target is one new `LanguageModel`.
- **Multi-module compilation** — single-module first; module decomposition substrate (`std/system.dag`) is a follow-on.

---

## Open questions — input wanted

### Ratified Q0 — Lens architecture (2026-05-20)

**Amended.** Lenses are pure readings over `Node`, sharing the `fold_node` primitive, with multi-lens dependency-management. Six commitments per P3 above; the old B-style/C-style wrapper question is superseded because caller policy, not compile-core, owns enforcement. The remaining substrate question is **Ratified Q6 — multi-lens execution derives from `fold_node` with a composed-lens-algebra** (below).

### Ratified Q1 supersession — option C runtime split (2026-05-21)

**Resolved** (operator-direct). A shared `ModelCore` substrate carries primitive types, algebra inhabitance, laws, effect semantics, partiality. `LanguageModel` consumes it for grammar/serialization target modeling. Runtime evaluation consumes decomposed abstract carriers from `std/runtime.dag`, while concrete runtime fact-bundles live under `extdeps/runtimes/*.dag`. Runtime-side concerns (allocation, execution model, runtime values, resource limits) remain categorically different from emit-side concerns (grammar, serialization), but there is no single substrate-level runtime umbrella carrier. Q1c ("eval = translate-to-machine-code + execute") remains rejected because direct evaluation involves runtime values, process state, and failure modes.

### Ratified Q2 — `InferredTree` dimensional facts (2026-05-20)

**Resolved.** `InferredTree` carries **compiler-core semantic facts only**: locus, binding, typeshape, inhabitance witness, **and the P6 semantic dependency graph** (`BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn`). It does NOT carry dimensional lens outputs — cost, complexity, ownership policy, termination policy, parallelism decisions, optimization decisions — those are lens outputs (P3) produced as side-channel folds over `InferredTree`. The former `04_infer.dag` import of `v4.lens.cost.SymbolicCost` has been removed; keep compiler core free of specific lens imports.

### Ratified Q3 — Parse + normalize stay logically separate (2026-05-20)

**Resolved Q3a.** Parse and normalize are logically distinct stages. The reason is inspectability + correctness, not performance: while sugar dissolutions are being proven honest, the ability to inspect `(text → SurfaceNode → CoreNode)` separately is load-bearing. Diagnostics from normalize must point back to the original surface construct (Locus preservation), not synthetic substrate nodes. An implementation MAY later fuse parse + normalize for efficiency, but only if it can still expose the per-stage debug/proof artifacts.

### Ratified Q4 — Explicit `traverse_node` derived from `fold_node` (2026-05-20; updated post-Pass B unification)

**Resolved Q4a, refined under Pass B unification.** `traverse_node` is **derived** (not a separate primitive) — it is `fold_node` invoked with an Outcome-threading algebra, where the *combinator* (not the algebra) interprets `StageDiagnosticPolicy` to thread failures. Per Worker C's escalated `NodeOutcomeFold.step` signature collapse: the algebra is pure-success-accumulator `fn(R, Edge, R) -> Outcome<R>` (the third parameter is the **already-folded child result `R`**, matching the landed `NodeFold<R>.step: fn(R, Edge, R) -> R` from `src/v4/std/node.dag:43` — the Outcome wrapping is the sole difference from the pure-catamorphism primitive; this preserves "derived from `fold_node`" structurally, not just notionally). Failure policy lives at the combinator, not the algebra. Single authority for failure handling.

Required substrate (declared as derived combinators in `std/node.dag` or peer):

1. **`traverse_node` / `sequence_node` / `bind_outcome`** — derived combinators that wrap `fold_node` with `Outcome`-threading. Single declaration of each; no duplicates (per Worker C finding #3 — `map_node_outcome` ≡ `bind_outcome` is forbidden).
2. **`Outcome<T>` shape**: `Accepted { value: T, diagnostics: Diagnostics } | Rejected { diagnostics: NonEmptyDiagnostics }` per Ratified Q11. Clean success = `Accepted(value, None)`. Two-variant collapse; no `Produced` / `RejectedAccumulating` split.
3. **`StageDiagnosticPolicy`** with `accumulation: Accumulate | ShortCircuit`; fatality is `FailClosed` invariant, NOT a selectable mode. **Type declaration lives in `std/diagnostic.dag`** (single authority — extension of the existing Outcome substrate). Each stage row in `std/pipeline.dag` carries a `diagnostic_policy: StageDiagnosticPolicy` field referencing per-stage VALUES of that type. The TYPE has one home; the VALUES are per-stage data — that is not split authority.
4. **Define laws** — `traverse_node`'s derivation from `fold_node` must obey identity (`traverse(pure)` is no-op), composition (`traverse` over composed algebras = composition of traverses), failure-monotonicity (a Rejected anywhere produces Rejected overall; accumulation is per-policy).

Without the laws + single-authority discipline, Q4a degrades to Q4b convention — that's the trap. Per Worker C's `NodeOutcomeFold.step` finding: algebras MUST NOT see prior `Outcome` (would create dual authority). The combinator is the sole policy-aware site.

### Ratified Q5 — Resolve stays its own stage; `LanguageModel` expanded (2026-05-20)

**Resolved Q5a.** Resolve is its own stage. Parsing is not responsible for binding — binding has too many nonlocal concerns (lexical scope, imports, modules, namespace separation, function/type/value namespaces, shadowing, hoisting, recursive declarations, macros, visibility) that parse can't satisfy in one pass.

Per the reviewer, `LanguageModel` is expanded (see "Carrier shapes" above) to declare these explicitly:
- Binding / namespace / scope rules.
- Effect / partiality declarations.
- Version / dialect / edition metadata.

For Rust, Python, JavaScript, etc., binding behavior is not optional metadata — it is part of the language model.

### Open Q15 — Bridge-of-bridge: when does the bridge itself need a bridge?

Per P9's bootstrap-impact taxonomy, Case 3 (bootstrap-breaking change) requires a modeled `Bridge[k → k+1]` migration package. **Second-order question:** when does upgrading the bridge format itself become a bootstrap-breaking change? I.e., if `Bridge`'s schema changes between epochs, does that require a `Bridge[bridge_k → bridge_k+1]` — bridge-of-bridge?

Sub-questions:
- Is `Bridge` a special-status carrier that's never allowed to change format (so bridge-of-bridge is impossible by construction)?
- Or is `Bridge` schema-versioned with a stable canonical schema, similar to `CorePackageSchema`?
- Or is there a tiny "bootstrap-of-bootstrap" seed for bridge-schema changes (recursive but bounded)?

Probably needs operator ratification before bootstrap substrate lands.

### Ratified Q6 — Multi-lens execution derives from `fold_node` with a composed-lens-algebra (2026-05-20, Pass B unification)

**Resolved.** Multi-lens execution is `fold_node(InferredTree, composed_lens_algebra)` where `composed_lens_algebra` is a single algebra carrying multiple per-Node lens-step functions + their dependency DAG. No new substrate primitive — this is the unification's point: any pattern that LOOKS like "find / coalesce / traverse" is `fold_node` with the appropriate algebra-as-data.

- **Single-pass coalescing** is the algebra computing all independent lens outputs per Node visit.
- **Lens-to-lens dependencies** (e.g., complexity reads cost): the lens-dependency relation is **itself a typed dependency graph** (per P6 — kinds: `LensReadsLensOutput` / `LensRequiresFact`); its topological order is computed by the **same `Typed dependency graph + topological/SCC machinery` primitive** that produces program-level orderings. Single authority for topological ordering = the typed-dep-graph primitive, NOT a separate fold over the lens-dep DAG. The composed lens algebra consumes the topological order produced by that primitive at algebra-construction time, then the fold visits each Node once with the staged per-Node steps. Multi-pass over the *Node tree* is not required.
- **Lens outputs as Node-graph data:** yes — `DimensionFact` carriers are Node-shaped, so downstream lens-algebras consume them uniformly and downstream tooling (IDEs, reporters) can read them.

Substrate that needs to land: the `LensAlgebra<F>` carrier shape + the composed-lens-algebra construction (which **consumes the topological order from the Typed dependency graph primitive** — single scheduling authority — to produce the per-Node-step composition). The lens-dependency DAG is a P6 typed-dependency-graph instance (one of its consumer surfaces), not a parallel ordering authority.

### Ratified Q7 — `LanguageModel` is declarative-data-only (2026-05-20)

**Resolved.** `LanguageModel` is purely declarative `.dag` data. Grammar productions, sugar dissolutions, binding rules, version metadata, algebra inhabitance — all data, not callable functions. No executable predicates / algebras / functions on a `LanguageModel`. Any callable on a `LanguageModel` is a Practice-10 hand-rolled-derived-operation smell + risks smuggling back a hidden emitter / hidden parser / hidden test generator (the v2 `emit_rust` failure mode P1 + P8 exist to prevent).

**Macro-handling languages out of scope** for the initial single-target compiler. Languages whose scope/binding rules require macro-rewrite semantics (Lisp-family, some C++ templates) need a typed-predicate dialect modeled separately if/when added. Defer trigger: first attempt to ground a macro-rewrite language; treated as a substrate-amendment requiring explicit operator ratification.

### Ratified Q8 — Coercion failure diagnostic carries `CoercionMismatchKind` (2026-05-20, partial)

**Resolved at substrate level.** Per Pass A's TASKS.md amendment, coercion failure is `Outcome::Rejected { diagnostics: NonEmptyDiagnostics }` where each diagnostic carries a `CoercionMismatchKind` payload: `CoercionMismatchKind = NoTargetCandidate | AmbiguousTargetCandidate | StructuralMismatch | WouldLoseInformation`. Each kind carries actionable provenance (which source inhabitance, which target candidates considered, where structural inequality surfaced).

**Deferred-with-trigger: near-miss / structural-diff IDE quick-fix details.** Defer trigger: first IDE/editor integration consuming the diagnostic; UX shape lands when there's an actual consumer beyond the compile-time error.

### Ratified Q9 — Derivation untrusted; witness check trusted (2026-05-20, Pass B)

**Resolved.** The `find_witness` primitive's output (candidate + witness pair) is **independently checkable**. A separate `verify_witness(source_facts, candidate, preservation_predicate, witness) -> Bool` consumes the witness and re-proves the preservation predicate without re-running candidate enumeration.

Discipline: **the derivation is untrusted; the witness check is trusted.** The compiler doesn't have to trust its own candidate-enumeration; the witness is verifiable by inspection. Benefits:
- Downstream tooling has a cheap re-validation path (IDE, build cache, distributed verification).
- Clean abstraction boundary: "find the candidate" (production) vs "verify the candidate satisfies the predicate" (verification).
- Applies uniformly across all `find_witness` instances: solve_constraints witnesses are check-by-replaying constraint-satisfaction; coercion-fold witnesses are check-by-replaying the structural-equality zip-fold; lawful-rewrite witnesses are check-by-replaying the rewrite-law verification.

The 3 witness carriers in the witness taxonomy above (`StructuralPropertyWitness<P>` / `HomomorphismWitness<R>` / `PromotionWitness`) each carry enough data to be re-checkable without re-running their producing primitive.

Q9b (ambiguity surface for multiple valid candidates) is folded into **Ratified Q14** above — handled by target-declared `TargetSelectionPolicy` or `AmbiguousTargetCandidate` diagnostic; not a separate question.

### Open Q10 — Partiality and effects in `ModelCore` (deferred-with-trigger)

**Deferred.** Effect / partiality modeling is substantive substrate work that the dependency substrate work pulls in as a sub-question (per the P6 lens-over-substrate sharpening 2026-05-20). Resolution lands when:
- **Trigger:** first primitive operation in `std/` that declares non-trivial effect/partiality (e.g., a `divide` operation that must declare division-by-zero partiality + an effect signature). At that point, the substrate representation decision (effect-types-as-fact-bundles vs effect-rows-in-type-system vs effect-as-substrate-usage-classified-by-the-dependency-lens) must be ratified.
- **Pending: representation candidate names** are `EffectSignature` / `ResourceAccess` / `CommutationWitness` / `ConflictWitness`.

The "EffectDependsOn / ResourceDependsOn as label vs as derived classification" decision IS the answer per the P6 sharpening: they are *lens classifications* of underlying `EffectSignature` / `ResourceAccess` substrate facts, not labels authored as a parallel data structure.

### Ratified Q11 — `Outcome<T>` per-stage policy via `StageDiagnosticPolicy` (2026-05-20)

**Resolved Q11c.** Per-stage choice via `StageDiagnosticPolicy { accumulation: Accumulate | ShortCircuit; fatality: FailClosed (invariant) }`. **The TYPE declaration lives in `std/diagnostic.dag` (single authority — extension of the existing Outcome substrate).** Each stage row in `std/pipeline.dag` carries a FIELD `diagnostic_policy: StageDiagnosticPolicy` referencing the per-stage value of that type; no duplicate type declaration. **Fail-closed is the invariant, NOT a selectable policy** (per strict-review correction #7).

`Outcome<T>` shape (per strict-review correction #11):
- `Accepted { value: T, diagnostics: Diagnostics }`
- `Rejected { diagnostics: NonEmptyDiagnostics }`
- where `Diagnostics = None | Some { diagnostics: NonEmptyDiagnostics }`

Clean success is `Accepted(value, None)`. No `Produced` vs `Accepted` split; no `Rejected` vs `RejectedAccumulating` split. The canonical derived-combinator set is `traverse_node` / `sequence_node` / `bind_outcome` per Ratified Q4 — they take/thread the `StageDiagnosticPolicy` to honor accumulation vs short-circuit. `map_node_outcome` is explicitly forbidden as a duplicate (per Worker C finding #3 — `map_node_outcome ≡ bind_outcome`); use `bind_outcome` instead.

### Ratified Q12 — Grammar bidirectional law is target-parse-after-serialize (2026-05-20)

**Resolved Q12b** as the default. Grammar law: `parse_target(serialize_target(target_node)) == target_node`. Serialize is canonicalizing (picks one canonical form for whitespace/comments/optional syntax); parse recovers the substrate Node from any valid concrete representation. Already in the primitive set table (see the `Grammar-as-bidirectional-data` row under "The primitive set — the things the compiler IS").

**Q12c (source-text-faithful round-tripping)** is **opt-in** for round-trip code-mod tooling that needs trivia preservation. NOT the compiler default; lives in tooling that wants it. Q12a (full bidirectionality both ways) is rejected — parsing is many-to-one for most languages.

### Open Q13 — Language versions / dialects (deferred-with-trigger)

**Deferred.** Multiple-version-of-same-language modeling is real work (Rust editions, Python 2 vs 3, TypeScript strict mode, C++ standards). The substrate decision is whether multiple versions are *one* parameterized `LanguageModel` (e.g., `rust.dag` with `version: 2021` field) or *separate* `LanguageModel` files (`rust_2021.dag`, `rust_2024.dag`).

Defer trigger: first attempt to compile against a non-current version (e.g., Python 2 alongside Python 3, or Rust 2024 alongside 2021). At that point, both options ratify; the call depends on how different the versions are (parameterized works for editions; separate files needed for Python 2 vs 3 where the languages are fundamentally different).

### Ratified Q14 — Target selection: deterministic policy OR ambiguity diagnostic (2026-05-20)

**Resolved Q14c + Q14d:** when the coercion fold finds multiple structurally-valid target candidates:
- **Default:** target language model declares a deterministic `TargetSelectionPolicy` (e.g., `UniqueOnly | TargetDeclaredPriority { priority } | UserSelected { selection }`). The target's own model picks the canonical preference; compiler defers to that authority.
- **Fallback when no policy declared OR policy fails to disambiguate:** `Outcome::Rejected { diagnostics: NonEmptyDiagnostics { head: AmbiguousTargetCandidate { candidates }, ... } }` — fail-closed; user (or wrapper) chooses per call site.

**Scope clarification (per openai-pro single-authority finding 2026-05-20):** `TargetSelectionPolicy` is **target-coercion-specific** — it is the value that goes into the `MultiplicityPolicy::TargetSelection(...)` variant supplied to `find_witness` BY THE COERCION FOLD CALL SITE ONLY. The generic `find_witness` primitive does NOT bake `TargetSelectionPolicy` in. Non-target use sites (notably `solve_constraints` for canonical source grounding) supply `MultiplicityPolicy::UniqueOnly` and CANNOT opt into target-selection. This separates the substrate-level multiplicity-policy authority (the `MultiplicityPolicy` type, lives with the `find_witness` primitive) from the target-language-level target-selection authority (the `TargetSelectionPolicy` value, lives with each target `LanguageModel`).

**Q14b (lowest-cost via cost-lens) is rejected** — would require lens framework to be part of compile-core decision-making, violating P3 commitment 4 (no specific lens imports in compiler-core). If a project wants cost-driven selection, it lives in a wrapper that runs the cost lens and supplies a `UserSelected` policy — NOT in compile-core.

---

## What this doc does NOT decide

- Specific implementation fan-out / who-implements-what (separate doc, follows ratification here).
- The shape of `extdeps/protocols/rest.dag` (out of scope per P4).
- Whether the bootstrap seed is Rust, native code, or another language (separate discipline — see `docs/design-pure-bootstrap-zero.md`).
- The detailed shape of the coercion fold's algorithm (substrate design follow-up; this doc only commits the **shape** — zip-fold over canonical groundings, decidable-by-construction, fail-closed on empty candidate set).

---

## Process for amending this doc

- A change to a ratified premise (P0 / P1 / P2 / P3 / P4 / P5 / P6 / P7 / P8 / P9) requires explicit operator ratification + reconcile of any downstream substrate.
- Resolution of an Open Q is recorded inline (move from "Open" section to a new "Ratified" subsection with date).
- Implementation that contradicts a premise is a STOP — escalate, do not improvise.

---

## Glossary

| Term | Meaning |
|---|---|
| **`.dag`** | The language `gunbc` compiles. Source files have `.dag` extension. Programs are typed Node trees. |
| **Node** | The substrate primitive (`src/v4/std/node.dag`). Every `.dag` value, type, function, and statement is a Node. |
| **`fold_node`** | The substrate's catamorphism — generic structural recursion over a Node tree, parameterized by an algebra. |
| **algebra** | A per-Node-kind step function plugged into `fold_node`. Compiler "passes" are algebras, not new code. |
| **catamorphism** | A structure-preserving fold over an algebraic data type. In our case, `fold_node` is the catamorphism over Node. |
| **homomorphism** | A structure-preserving map between algebraic structures. In our case, the map from source Nodes to target Nodes (via the coercion fold). |
| **coercion fold** | The mechanical zip-fold (per `TASKS.md` T-9) that walks two canonical Node groundings in parallel to check structure preservation. Decidable by construction; empty candidate set ⇒ Diagnostic. |
| **grounding** | The process of attaching algebra-inhabitance witnesses to every Node (i.e., "this Node lives in this algebra-instance"). What stage `ground` produces. |
| **inhabitance** | A type/value inhabits an algebra-instance if it satisfies that algebra's laws. E.g., `Int` inhabits `OrderedRing`. |
| **`LanguageModel`** | A `.dag` model in `extdeps/languages/X.dag` declaring language X's grammar + primitives + sugar dissolutions. |
| **`InferredTree`** | The post-grounding Node tree — same shape as input, with inhabitance/typeshape/binding witnesses attached. |
| **Practice 8** | "Fact-bundle modeling" rule — every external primitive must declare its facts (width, signedness, range, ...) rather than being a bare alias. See [`docs/modeling-discipline.md`](modeling-discipline.md#8-fact-bundle-modeling). |
| **Practice 9** | "No-prose discipline" — `.dag` files carry only structured comments, never narrative prose. |
| **Practice 10** | "Don't hand-roll a derived operation" — every fold/walker/translator that's mechanically determined by a Node's shape must use a substrate primitive (`fold_node`, `find_witness`) or a derived combinator built on them (`traverse_node`, coercion fold, `solve_constraints`, etc.), not hand-rolled recursion. Per Pass B unification, `traverse` / coercion fold are *derived*, not primitive — but they remain the right thing to reach for instead of hand-rolling. |
| **T-8 / T-9 / T-10 / T-22** | Task IDs in `src/v4/TASKS.md`. T-8 = normalize+resolve; T-9 = ground (infer); T-10 = translate; T-22 = eval. |
| **D2 reversal** | A 2026-05-17 operator-ratified design course correction (recorded in `src/v4/TASKS.md`) that reshaped how language fact-bundles and the coercion fold are framed. The current "coercion fold (not search)" framing is post-D2. |
| **fail-closed** | A discipline (Practice 1): every failure path produces a structured Diagnostic; no silent `None`s or panics. |
| **`ModelCore`** | The shared substrate consumed by `LanguageModel` and concrete runtime bundles — primitive types, algebra inhabitance, laws, effect semantics, partiality. |
| **`DependencyKind` / `DependencyView`** | Per P6 (sharpened 2026-05-20): `DependencyKind` is the **classification taxonomy** the dependency lens maps substrate usage patterns into; `DependencyView` is the **lens-output tuple** `(source, dependent, kind, usage_site)` emitted by the `dependency_lens` fold over the substrate. The `usage_site: Node` field is a **reference** to the substrate Node where the usage occurred (the import statement Node, the field-ref Node, the function-call Node — whichever the lens classified) — it carries the substrate's existing fact forward via Node reference per Practice 3 (Facts Flow Forward), enabling downstream consumers (diagnostics, `affected_set` provenance, T-21 frontier receipts) to cite the substrate authority. **NOT** an authored `DependencyEdge` carrier and **NOT** a copy of substrate facts into a parallel payload — the substrate's usage graph IS the dependency graph; the lens classifies and emits references back to substrate, never duplicates. Kinds (program-tier): `Contains`, `BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn`, `BarrierBefore`, `PlacementDependsOn`. Kinds (artifact-tier): `ModelDependsOn`, `ProjectionDependsOn`, `GeneratedFrom`, `VerifiedBy`. Kinds (bootstrap-tier): `PromotedBy`, `BootstrapDependsOn`. |
| **synthesized attribute** | A fact computed bottom-up from a Node's children (e.g., typeshape, content_hash). Naturally fits `fold_node`. |
| **inherited attribute** | A fact propagated top-down from a Node's parent / context (e.g., scope environment in resolve). Requires a higher-order algebra carrier per P5. |
| **SCC condensation** | Strongly-connected-component reduction: collapsing valid cycles in a dependency graph into single nodes so the result is a DAG. Used to handle mutually-recursive definitions and other cyclic-but-valid structures. |
| **`find_witness`** (unified primitive, Pass B 2026-05-20) | `find_witness(source_facts, closed_candidate_set, preservation_predicate, multiplicity_policy: MultiplicityPolicy) -> Outcome<(Candidate, Witness)>`. THE unified **candidate-enumeration-and-check** primitive (NOT a search per T-9 ratification — closed candidate set + mechanical predicate check + caller-supplied multiplicity resolution). Unifies what were previously `solve_constraints` + coercion fold + `LawfulRewriteWitness` — all three are `find_witness` invocations with different preservation predicates **AND different multiplicity policies** (`UniqueOnly` for source grounding; `TargetSelection(...)` per Q14 for target coercion; `RewritePolicy(...)` for P7). The `MultiplicityPolicy` parameter is what keeps target-coercion-specific selection out of source-grounding contexts (per openai-pro single-authority finding 2026-05-20). |
| **`solve_constraints`** (DERIVED, not primitive) | `find_witness(constraint_graph, canonical-form-candidates, constraint_satisfaction_predicate, MultiplicityPolicy::UniqueOnly)`. Produces canonical source grounding via the unified primitive. **The `UniqueOnly` policy is required** — canonical source grounding has no notion of "target preference"; multiple passing candidates ⇒ ambiguous grounding ⇒ fail-closed. NOT optional, NOT defaulted; pass it explicitly at the call site. |
| **coercion fold** (DERIVED, not primitive) | `find_witness(canonical_source_grounding, target_lang.declared_inhabitants, exact_structural_equality_zip_fold, MultiplicityPolicy::TargetSelection(target_lang.target_selection_policy))`. Verifies homomorphism via the unified primitive. The `TargetSelection` policy is sourced from the target language model (per Ratified Q14) — sole legitimate site for target-selection-policy leakage; non-target use sites cannot opt in. T-9 ratification preserves the "coercion fold" name; conceptually it's `find_witness`. |
| **`LawfulRewriteWitness`** (DERIVED, not primitive) | `find_witness(source_grounding, candidate_rewrites_declared_by_target_or_runtime, lawful_rewrite_precondition_predicate, MultiplicityPolicy::RewritePolicy(...))`. Structure-changing lowerings (parallel-map, tree-reduce, CUDA, MapReduce). Different rewrites = different predicates over `find_witness`. **The `RewritePolicy` variant + this call site are gated on P7** — neither exists in MVP scope; this row documents the future shape only. |
| **`ConstraintGraph`** | Intermediate carrier produced by the grounding fold, consumed by `solve_constraints` to produce the canonical `InferredTree`. |
| **`ClosedWorldDependencyWitness`** *(collapsed into `StructuralPropertyWitness<P>` per Pass B witness taxonomy — instance with `P = being-completely-classified`)* | The structural-property witness asserting "this region's dependency classification is complete." Required before absence-of-edge can be interpreted as independence (e.g., for parallelism). |
| **canonical (source) grounding** | The unique, witnessed normalization of a Node's algebra-inhabitance facts. The coercion fold operates only on canonical groundings; ambiguity surfaces as a diagnostic in ground, never as multiple downstream candidates. |
| **ingest** | The (separable) layer that produces a `CoreNode` from a source (text, programmatic builder, query-driven rewrite, round-trip). The compile-core takes `CoreNode` and does not know how it was produced. |
| **`IngestionPlan`** | The input-side peer of `ProjectionPlan`: a `CoreNode` subject plus an ingestion producer reference and typed `params`. Explicit language, inferred language, and richer ingestion/injection conventions live here instead of as scalar arguments on `compile_session`. |
| **ingest_text** | The text-to-`CoreNode` ingest path: `ingest_text(text, input_lang) → Outcome<CoreNode>`. Parse + normalize composed. Uses the grammar-as-bidirectional-data primitive (forward direction). |
| **`Path`** | A structural address to a sub-Node — `Path { steps: List<Symbol> }`. NOT a filesystem path; the Node graph's own coordinates. Ratified in `src/v4/std/node.dag` (PR #3162). |
| **`Edit`** | A structural rewrite: `Edit { at: Path, replacement: Node }`. Replacement only — no insert/delete variants; those decompose into parent-replacement. |
| **`Diff`** | An ordered sequential rewrite program: `Diff { edits: List<Edit> }`. Sequential composition, NOT parallel. Fail-closed all-or-nothing on `apply_diff`. |
| **`apply_diff`** (DERIVED, not primitive) | The structural-edit **derived combinator** = `sequence_outcome(diff.edits, fn(edit, candidate) -> fold_node(candidate, substitute_at(edit.at, edit.replacement)))`. **Sequential** Outcome-bind-fold over the Diff's ordered Edits list — each Edit's Path resolves against the intermediate candidate (post-prior-Edit state), NOT the original root; fail-closed on any unresolved Path in the intermediate state. The agent-side mutation operation; composes with `fold_node` + `sequence_outcome`. |
| **`apply_lens`** (current scaffold; target is `observe`) | Current `lens/application.dag` surface: `apply_lens(lens, scope, mode) → Witness<finding> \| Outcome<()>`. Audit disposition: reframe as graph-in lens reading that contributes to `LensReport`; `Enforce` semantics move to `authorize`. |
| **`RegionRef`** | Low-level region reference shared by graph consumers. It is not a universal `SubgraphScope`; lenses and projections refine it into their own scope/subject shapes. |
| **candidate-state pattern** | The read/edit invariant: readings and policy evaluate the accepted candidate from `apply_diff(dag, Diff)`, NOT the pre-edit graph. `apply_diff` is an `Outcome`; if it rejects, the candidate flow fails closed before readings run. |
| **`affected_set`** | T-21 derived combinator + lens-frontier specialization: `affected_set(dag, Diff) → Witness<ReExecFrontier>`. Incremental re-execution frontier derived from `fold_node` over dependency facts; consumed by caller/session policy and projection regeneration. |
| **self-edit / stage0** | The compiler editing its own source via `apply_diff(self, Diff)`. Self-hosting (stage0 regeneration) is `project(self_graph, rust_stage0_projection_plan)` yielding a `ProjectionReport`, then `authorize(...)` + the P9 promotion gate before the generated artifact can replace the active seed — no special compile mode. See "Self-modification" section. |
| **projection** | Per P0 + P3, every downstream artifact (tests, glue, schemas, migration plans, docs, dry-run results) is a projection of `InferredTree` computed by a projection producer and reported through `ProjectionReport`. Lens readings may feed projection producers, but they do not own artifact materialization. |
| **testgen** | A projection family that produces `TestClaim` / `TestCase` artifacts from `InferredTree`, optionally consuming lens readings. Historical location/name predates the projection split. Tests are projections of the model, NOT the source of truth for the model. |
| **dry-run** | A projection over graph readings: `io_boundary_lens` identifies the I/O sites; a `LawfulRewriteWitness` substitutes mock stubs; `eval(mock_runtime)` runs the substituted tree. No new compiler primitive; composes lens reading + rewrite + eval projection. Variations: record, assert, replay, fuzz. |
| **peripheral shim** | A user-facing convenience that composes modeled effects with session phases. Examples: `ingest_text` (file_read + parse + normalize), `compile_file_to_file` (file_read + ingest_text + ground + observe + project + authorize + file_write). Shims are NOT substrate primitives — they live outside the architecture's load-bearing surface. |
| **modeled effect** | A real-world I/O operation declared as substrate data in `extdeps/` (file_read, file_write, network call, process spawn, …) — carries `EffectDependsOn` / `ResourceDependsOn` edges per P6; visible to lenses; substitutable by `LawfulRewriteWitness` for dry-run. There are no implicit side effects in the architecture. |
| **`TestClaimProjection`** | Projection layer 1 of testgen — produces abstract behavioral claims (roundtrip / refinement-boundary / algebra-law / protocol-compatibility / effect-idempotency) as `TestClaim` Witnesses. |
| **`TestCaseProjection`** | Projection layer 2 of testgen — lowers `TestClaim` Witnesses into concrete `TestCase` Witnesses (examples, boundary cases, property-test generators, fuzz seeds, regression fixtures). |
| **`TargetTestProjection`** | Projection layer 3 of testgen — translates `TestCase` Witnesses into target-language test source via translate's homomorphism. |
| **`ChangeSet`** | A diff between two model versions (changed Nodes / edges / facts / witnesses / grammar productions / LanguageModel fields / lens contracts). Substrate for P0 + P8 consequence-recomputation. Not yet declared (`std/change.dag`). |
| **`AffectedSet`** | Downstream artifacts requiring recomputation given a `ChangeSet`. T-21 `affected_set` is the lens-frontier specialization; full `AffectedSet` over arbitrary projection graphs not yet declared. |
| **`Projection`** | Declared projection-as-data: name + input carrier + output artifact + dependency requirements + regeneration producer + validation policy inputs. Not yet declared (`std/projection.dag`). |
| **`Artifact`** | An emitted artifact (source file, test file, schema, diagnostic table, compiler stage table) with its provenance (which projection produced it from which model state). Narrow identity/provenance substrate is declared in `src/v4/std/artifact.dag`; full P8 regeneration integration with `Projection`, general `AffectedSet`, and `RecomputePlan` is not yet declared. |
| **`RecomputePlan`** | Topological schedule + SCC/fixpoint groups + cached-vs-invalidated artifacts. The "how to recompute" data given an `AffectedSet`. Not yet declared. |
| **`Stage0Contract`** | What stage0 promises (consume canonical CorePackage; minimal `fold_node` + derived `traverse_node` combinator; fail-closed; emit verified artifact). Per P9. Not yet declared. |
| **`CorePackage` / `CorePackageSchema`** | The stable canonical bootstrap package stage0 consumes (per P9). NOT the evolving surface language — decoupled from it. Not yet declared. |
| **`BootstrapEpoch`** | k-indexed snapshot of (stage0, CorePackage, SourceModels). Per P9. Not yet declared. |
| **`Bridge`** | Migration package for bootstrap-breaking changes (Case 3 of P9's bootstrap-impact taxonomy). Not yet declared. |
| **`PromotionWitness`** | Verification artifact for the promotion step (P9), with internal sub-fields for the bootstrap-roundtrip and fixed-point checks (formerly `BootstrapWitness` + `FixedPointWitness`). Not yet declared. |
| **promotion protocol** | The guarded step that replaces `Stage0[k]` with `Stage0[k+1]`. Per P9 — the *only* "back edge" in the system. Not yet implemented. |
