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
- [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) — **the EDIT-direction companion to this doc.** Defines the agent-side read/edit surface: `apply_lens(lens, scope, mode)`, `apply_diff(root, Diff)`, the seven-step read→edit pipeline, candidate-state gating, library-first agent surface, Layer 0–6 build order. **PR #3364 merged 2026-05-19.** This doc's "ingest paths" table refers to it for the query-driven rewrite mechanism; this doc's EDIT-direction section (below) is a summary; the read-edit-pipeline doc is the authority.
- [`docs/design-emit-stage-l25-model.md`](design-emit-stage-l25-model.md) — emit-stage L2.5 model, the substrate-side specification for what this doc names as `translate` + `serialize` (T-10).
- [`docs/design-infer-stage-l25-model.md`](design-infer-stage-l25-model.md) — infer-stage L2.5 model, the substrate-side specification for what this doc names as `ground` (T-9).
- [`docs/design-lens-application-surface.md`](design-lens-application-surface.md) — the substrate-level lens-application surface; defines the `Lens / SectionRef / Witness / Outcome` shapes this doc's P3 commits to.
- [`docs/design-lens-framework.md`](design-lens-framework.md) — the lens framework as a whole.
- [`docs/design-affected-set-lens.md`](design-affected-set-lens.md) — `affected_set` (T-21), the incremental-rebuild + lens-frontier primitive consumed by both this doc's pipeline diagram and the read/edit pipeline's gating step.
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
2. Either **emit** that same graph as source code in a target language (Rust, Python, Go, TypeScript, C++, …), or **evaluate** it directly on a host runtime.
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
2. **Plus eight substrate operations** that aren't `fold_node` but are needed:
   - A **diagnostic carrier** (`Diagnostic + Locus`, fail-closed reporting).
   - **Grammar-as-bidirectional-data** — one declarative grammar model that drives both parsing and serialization. Default law: `parse_target(serialize_target(node)) == node`.
   - **`traverse`** over the diagnostic-effect carrier (a Node fold/traverse that short-circuits or accumulates over `Outcome<T>` without re-derived match ladders).
   - **`solve_constraints`** — declared constraint-solving substrate; produces the *canonical* source grounding consumed by the coercion fold. Distinct from the coercion fold.
   - **The coercion fold** — a mechanical zip-fold over two canonical-Node groundings that checks **exact** structure preservation. Per `src/v4/TASKS.md` T-9, decidable by construction, never a search.
   - **`LawfulRewriteWitness`** (P7) — witness primitive for structure-changing lowerings (parallel-map, tree-reduce, CUDA, MapReduce). Rewrites are declared by target/runtime models with precondition algebra laws; rewritten plan groundings then go through the coercion fold's exact check.
   - **Typed dependency graph + topological/SCC machinery** — per P6, dependencies are first-class typed edges carried alongside containment.
   - **`apply_diff`** (write-side primitive, symmetric to `fold_node`) — structural mutation of a Node graph via a `Diff = List<Edit { at: Path, replacement: Node }>`. Fail-closed all-or-nothing. The agent-side mutation primitive. The substrate is **read/write-symmetric** at the primitive layer: reads are folds, writes are `apply_diff`. See [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) for the full surface.
3. **Compile-core has no boundary actions.** The compile core is **purely data-in / data-out**: `CoreNode + LanguageModel + CompileMode → Outcome<Output>`. No implicit text-reading, no implicit text-writing, no implicit "execute." All real-world I/O — reading source from disk, writing target source to disk, executing on a host, reporting diagnostics to stderr — is **modeled as effects against modeled resources** (`extdeps/file_system.dag` for files, `extdeps/process.dag` for shell/OS, `extdeps/network.dag` for network, etc.), composed by *peripheral shims* outside the compile-core surface. The architecture itself doesn't think about text/files/processes; those are orthogonal concerns modeled independently.

Everything else — every "pass," every "stage," every "language-specific" anything — is an **algebra** plugged into `fold_node` or another declared substrate primitive. The compiler knows no specific language code (Rust, Python, even `.dag` itself, beyond an irreducible bootstrap-seed for `dag.dag`); language-specific *data* lives in `LanguageModel` declarations under `extdeps/languages/`.

**Algebras can be higher-order, effectful, constraint-bearing, or fixpoint-seeking (P5).** "Fold" doesn't mean "pure bottom-up." Inherited context, effects, constraints, and fixed-point analyses are first-class — carried by the algebra's signature, never as ad-hoc walkers.

**Dependency management is inherent to the substrate but only under enforced discipline (P6).** Synthesized (child→parent) facts ride `fold_node`'s natural order. Inherited (parent→child) facts ride higher-order carriers (P5). Memoization and incrementality ride content-addressed Node identity. The inherent property breaks the moment any pass walks Nodes outside the declared primitives — Practice 10 enforces this.

The doc that follows defines what `compile()` takes and returns, why each piece is shaped that way, what's already in the codebase, and what's still open for design.

---

## End-to-end I/O — the compile function

> **Text/data separation + I/O via modeled effects (ratified 2026-05-20, operator-direct).** Compilation operates on **data** (`CoreNode`), not text. Text, files, and shell/OS are **orthogonal concepts modeled independently** in `extdeps/` — there's a small shim to ingest text into `CoreNode`, but the compiler core never sees text or files. **The architecturally pure approach:** all real-world I/O is a modeled effect against a modeled resource (file_system.dag, process.dag, network.dag, …), composed by peripheral shims outside the compile-core surface.

```
// === Compile core — pure data-in / data-out, no I/O ===
compile(
  source: CoreNode,                    // canonical substrate Node graph (data, not text)
  input_lang: LanguageModel,           // for resolve's scope rules + ground's inhabitance
  mode: CompileMode
) -> Outcome<Output>

where CompileMode =
  | TranslateTo(target_lang: LanguageModel)    // produce TargetSource (data)
  | Eval(host_interp: HostModel)               // produce Value (data)
```

That is the full external surface of compile-core. **Three arguments; one fail-closed Outcome. No I/O.** Text, files, processes, network are NOT compile-core concerns.

### Peripheral shims for text and file I/O

If a user wants to compile a `.dag` source file from disk to a Rust source file on disk, they compose modeled effects with compile-core. Each step is its own substrate-modeled operation:

```
// Each line is a modeled effect / pure transform — NOT a compile-core boundary action.

source_text   = file_read(path)              // effect against extdeps/file_system.dag
core_node     = ingest_text(source_text,     // shim — composes parse + normalize
                            input_lang)
result        = compile(core_node,           // pure data-in/data-out — compile-core
                        input_lang,
                        TranslateTo(rust_lang))
target_text   = unwrap(result)               // TargetSource is just bytes
file_write(out_path, target_text)            // effect against extdeps/file_system.dag
```

- **`file_read` / `file_write`** are modeled effects in `extdeps/file_system.dag` — operations against the modeled file-system resource (carrying `ResourceDependsOn` edges per P6, fail-closed-or-explicit-modeled-default per P3).
- **`ingest_text`** is a peripheral shim — NOT a substrate primitive. Composes parse (using input_lang's grammar) + normalize (using input_lang's sugar dissolutions). The shim exists because text-on-disk is a common user-facing input format, but the compile architecture itself stops at CoreNode.
- **`compile`** is pure. No file handle. No text. No process. Just `CoreNode + LanguageModel + CompileMode → Outcome<Output>`.

**Other ingest paths** (no text involved at all):
- Programmatic: a client builds a `CoreNode` via substrate constructors (IDE plugins, code generators).
- Query-driven: `affected_set + apply_diff` produces a new `CoreNode` from an existing one (read/edit pipeline).
- Round-trip: a `TargetNodeTree` from a prior translate is parsed back via the grammar's inverse direction.

All paths produce `CoreNode`; compile takes it from there. **The compile contract doesn't know how the CoreNode was produced.**

### Why this matters architecturally

P0 + P8 (consequence recomputation + the compiler obeys its own discipline) plus the operator's I/O-as-modeled-effect rule together imply: **there are NO hidden side effects anywhere in the architecture**. Every read, every write, every execute is a modeled effect against a modeled resource, carrying typed dependency edges (P6) and visible to lenses.

This is what makes dry-run work as composition (lens identifies I/O → LawfulRewriteWitness substitutes → eval against mock_host_model). It's also what makes incremental rebuild work (every artifact's provenance is modeled; affected_set walks the typed effect dependencies). It's what makes self-modification safe (the compiler editing its own source goes through modeled `apply_diff`, not implicit file writes).

**The slogan.** *No implicit I/O. Files are not the architecture. Effects are modeled, not assumed.*

### Why text/data are separated

There are multiple legitimate entry points to a `CoreNode` graph:

| Entry point | How `CoreNode` is produced |
|---|---|
| **Text ingest** | `ingest_text(text, input_lang)` — text → parse → normalize → `CoreNode`. The standard `.dag` source-file path. |
| **Programmatic / API** | A client builds the `CoreNode` directly via substrate constructors. IDE plugins, code generators, build systems. |
| **Query-driven rewrite** | `affected_set` + a node-graph transformation produces a new `CoreNode` from an existing one. Future query languages (e.g., write-via-query against an existing program graph) operate here. |
| **Round-trip from translate** | A `TargetNodeTree` from a prior translate can be parsed back via the grammar's inverse direction and ingested as `CoreNode` (for cross-language refactoring / round-trip workflows). |

All paths produce `CoreNode`; `compile` takes it from there. This matters because:
- The lens / affected-set / query story (much of the v4 wishlist) operates on Node data, not text. Forcing every entry through text would either require lossless code-mod tooling or constrain the substrate's expressiveness.
- The compile pipeline stays pure data-in / data-out. Text I/O is at the ingest/serialize boundaries only.
- The substrate model is uniform: a `CoreNode` is a `CoreNode` regardless of how it was authored. The compile contract doesn't care which entry point produced it.

### Carrier types in this signature

**`CoreNode`** is the substrate-canonical Node — six connectives + five behaviors only, sugar dissolved. This is what compile's `source` parameter consumes. (See "Carrier taxonomy" below — `CoreNode` is the post-normalize shape, distinct from `SurfaceNode` and `TargetSurfaceNode`.)

**`LanguageModel`** is a `.dag` model in `extdeps/languages/X.dag` — a fact-bundle declaring X's grammar (bidirectional), primitive types with their facts (width, signedness, range, …), algebra inhabitance (this type inhabits this algebra), sugar dissolutions, binding/scope rules, and version metadata. `compile` consults it for resolve's scope rules and ground's inhabitance declarations; `ingest_text` consults it for grammar + sugar.

**`HostModel`** is a **distinct substrate type, peer of `LanguageModel`** — both extend a shared `ModelCore` (primitive types, algebra inhabitance, laws, effect / partiality semantics). `HostModel` declares host-side concerns that have no emit-side analog: runtime value representation, primitive operation interpretation, execution semantics, resource / effect boundary. **Ratified Q1** below; full carrier breakdown in "Carrier shapes — `ModelCore` / `LanguageModel` / `HostModel`."

**`Output`** is mode-dependent: `TargetSource` (bytes/text) for `TranslateTo`, `Value` (host representation) for `Eval`.

> **Note on Lens mode:** the original draft of this doc proposed a third `CompileMode` for running lenses (the analysis framework for complexity / cost / ownership / effects / user-defined dimensions). This was flagged as contradicting THESIS by an automated review. **Resolved 2026-05-20 — see P3 below.** Lenses are a side-channel over `InferredTree`, orthogonal to the emission/eval homomorphism. They're invoked via a project-mandatory wrapper (`validate_then_compile`-style) that gates emit/eval on lens pass; whether the lens fold runs inside or outside the bare `compile()` is a non-load-bearing surface choice.

---

## Architectural premises (ratified — these inform implementation briefs)

### P0 — The compiler exists to make change consequences computable

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

**The problem this compiler solves is not text-to-text code generation.** It's that in ordinary codebases, interface / integration / effect / resource / protocol dependencies are scattered, implicit, and manually synchronized. A small upstream change can break distant downstream behavior without a structural explanation; developers chase fallout via runtime failures, broken tests, and grep.

The v4 architecture's answer:

1. **Model the system as a typed dependency graph.** Interfaces, integrations, protocols, resources, effects, and transformations are first-class typed nodes/edges (P6), not implicit in arbitrary code.
2. **Ground it canonically.** `InferredTree` is the canonical post-grounding graph; ambiguity is a diagnostic, never a downstream surprise (canonical-grounding invariant).
3. **Code, tests, glue, schemas, runtime plans, diagnostics — all are projections of the grounded graph.** Not the source of truth. When a modeled fact changes, the compiler computes the affected dependency subgraph (T-21 `affected_set`) and **recomputes** all downstream projections whose inputs changed.
4. **If a projection can no longer be derived** by homomorphism or a declared `LawfulRewriteWitness`, compile fails-closed with a diagnostic.

**Source-of-truth inversion.** This compiler inverts the traditional relationship:

| Traditional codebase | This architecture |
|---|---|
| Code is source of truth | Model is source of truth |
| Interfaces are scattered | Interfaces are typed Nodes |
| Dependencies are implicit | Dependencies are typed edges (P6) |
| Tests detect drift after the fact | Tests are projections; drift is structural |
| Build failures reveal dependencies | Compile-time analysis enumerates affected sites |
| Manually patched cascade after a change | Compiler computes affected subgraph + regenerates projections; non-derivable consequences surface as diagnostics |

**Closed-world questions become decidable.** The compiler does NOT claim "all programming becomes decidable." It claims a **specific bounded set of system-level questions** becomes decidable *because the input structure is constrained*: the coercion fold is decidable-by-construction over a closed candidate set; parallelism is provable when dependency/effect/resource facts are complete; interface fidelity is structural because both sides derive from the same Node. See P5 (constraint solving), P6 (dependency taxonomy), P7 (lawful rewrites) for the mechanics; the resulting decidability is bounded and named, not universal.

**The slogan.** *Code is not manually synchronized; consequences are recomputed.*

### P1 — Languages are I/O integration surfaces, not compiler concerns

Both source and target languages are `LanguageModel` data passed in as parameters. The same `rust.dag` model is consumed when emitting Rust AND when parsing Rust as input (bidirectional grammar-as-data). There is no per-target emitter; `emit_rust` does not exist as compiler code. The Rust knowledge lives entirely in `extdeps/languages/rust.dag`.

This is a direct consequence of THESIS's "The derived homomorphism" claim plus [`docs/modeling-discipline.md`](modeling-discipline.md) Practice 10 Row 3, which classifies a hand-written cross-target emitter as "you re-wrote the compiler" — a whole-architecture failure, not a localized smell.

### P2 — Eval is translate-to-host

Evaluation is structurally the same operation as translation, with the host runtime as the target. The compiler-side machinery (parse, normalize, resolve, ground, then the coercion fold) is shared. Eval differs from translate **only** in the terminal action: execute the resulting structure via the host's interpretation algebra (eval) vs. serialize it via the target's grammar (translate).

`HostModel` shape is **ratified per Q1**: distinct `HostModel` as a peer of `LanguageModel`, both over shared `ModelCore`.

### P3 — Lenses are a side-channel over `InferredTree`

> **Ratified 2026-05-20** (operator-direct, resolving the codex-flagged conflict between the prior draft's lens stance and THESIS lines 105 + 342). The earlier "lenses entirely external to compile" framing was too strong; THESIS's "by construction" guarantee must hold. The reconciling insight: lens enforcement is **orthogonal to the homomorphism** — lenses observe the program; they don't participate in translate/eval. The right architectural question is the consumption *contract*, not the surface invocation.

Lens enforcement is **orthogonal to the emission/eval homomorphism.** The compiler's pipeline produces `InferredTree` as the **raw primitive layer where the program's full structure is visible.** Lenses are **folds over that tree** that produce dimension facts as side-channel output — they do NOT modify the tree, do NOT block translate/eval, and do NOT feed downstream stages of the homomorphism. THESIS's "by construction" guarantee is preserved by project-mandatory invocation of the lens pass, regardless of where the fold physically runs.

**Six commitments:**
1. **`InferredTree` is the lens consumption point** — a stable public contract. The lens framework binds to it.
2. **Lenses are folds over `InferredTree`** sharing the `fold_node` primitive (Practice 10 row 1).
3. **Lens outputs are side-channel** — `DimensionFact` / diagnostics. Translate/eval do NOT depend on lens output. Lens output does NOT feed downstream stages of the homomorphism.
4. **Built-in and user-defined lenses share one algebra contract.** The compiler core does NOT import or name any specific lens. `04_infer.dag`'s current `import v4.lens.cost { SymbolicCost }` violates this and needs to be fixed.
5. **Multi-lens execution is dependency-managed.** Lenses with no interdependencies are coalesced into a shared traversal. Lenses with dependencies (e.g., complexity reads cost) are scheduled by a **lens-dependency DAG**; each stage coalesces all lenses whose inputs are available. No lens may trigger an ad hoc unmanaged re-walk. See Open Q6 for the substrate primitive.
6. **"By construction" guarantee — type-enforced at the public terminal.** Lens output does not feed the homomorphism and bare compile-core does not consume lens output. **`validate_then_compile` (or equivalent) is the project's public terminal API**; bare `compile(source, input_lang, mode)` is **explicitly marked as internal / non-terminal** — accessible to advanced consumers (lens framework, build tooling, the compiler itself for self-edit), but NOT the everyday user surface. The distinction is **type-level**, not convention-level: a `Validated<Output>` carrier (or similar) discharged only by the wrapper enforces the gate at the type system, so a bare `compile()` invocation cannot accidentally bypass lens enforcement at terminal emit/eval. Project policy decides the wrapper's lens set; the wrapper-as-terminal pattern is invariant.

**Surface choice (B vs C) is not architecturally load-bearing.** Given commitments 1–6, two implementation surfaces are equivalent:
- **B-style:** lenses fold inside `compile(text, input_lang, lenses, mode)`.
- **C-style:** lenses fold outside compile, in a thin wrapper that calls `ground` + the lens fold + `compile_terminal`.

Both honor 1–6 identically. The `InferredTree` contract is the same; the lens algebra contract is the same; the dependency-management discipline is the same. **Refactoring B↔C is a mechanical change**, not an architectural one. Initial implementation will land one of them; the choice can be revisited without architectural cost.

**The hard substrate work** that 1–6 imply:
- A `LensAlgebra<F>` carrier shape that both built-ins and user-defined lenses satisfy.
- The multi-lens dependency-management primitive (Open Q6).
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

Practical implications:
- Resolve's algebra is not naively bottom-up. The carrier is roughly `Node → Env → Outcome<BoundNode>` — at each Node the fold returns a function that takes inherited scope context. This is an attribute-grammar-style inherited attribute, expressed through the carrier's higher-orderness.
- Ground (infer) may produce a `ConstraintGraph` per Node, then invoke a declared constraint-solving substrate primitive: `solve_constraints : ConstraintGraph → Outcome<InferredTree>`. The solver is **named substrate machinery — distinct from the coercion fold** — and must produce checkable witnesses or fail-closed diagnostics. It is not ad hoc compiler code. Its output is the **canonical source grounding** consumed later by the coercion fold during translate.
- Translate's coercion-fold step is a fold, but its algebra **enumerates the target language model's declared candidate inhabitants and performs a structural-equality zip-fold** against the source's canonical grounding. The candidate set is closed (declared in `target_lang.dag`) and the structural check is decidable by construction — this is **deterministic candidate enumeration**, not a heuristic search.

The discipline still holds (everything is a substrate-primitive operation), but "fold" doesn't mean "pure synthesized bottom-up." Inherited context, effects, constraints, and fixed-point analyses are first-class — they're just carried by the algebra's signature, not by ad-hoc walkers.

### P6 — Dependencies are first-class typed edges, not incidental graph topology

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

The compiler does NOT treat DAG-ness as an incidental property of syntactic containment. It maintains a **typed dependency graph** as part of the program carrier. Multiple kinds of dependency edges exist alongside the containment tree, and each kind has distinct semantic obligations.

**Dependency kinds** (initial taxonomy, in `std/dependency.dag` — currently missing substrate):

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

**Stage obligations:**
- `parse` produces `Contains` edges (the syntactic tree).
- `resolve` monotonically adds `BindsTo` and `ModuleDependsOn` edges.
- `ground` (infer) monotonically adds `TypeDependsOn`, `DataDependsOn`, and (where the source language declares them) `EffectDependsOn` / `ResourceDependsOn` edges.
- `translate` to a target consumes the dependency graph. A translation is valid **only if** it preserves the dependency partial order, OR if the target model supplies a **checkable witness** that a reordering, fusion, parallelization, or placement change is semantics-preserving (e.g., a witness that `map` is independent per element + has no non-commuting effects → safe to parallelize).
- `eval` schedules only according to satisfied dependencies. Parallelism is the default where the dependency graph permits it.

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

The compiler may model and regenerate its own implementation, **including stage0**, but **no running compiler generation mutates itself in place**. The "compiler edits itself" framing from earlier sections is precise only in this stratified sense:

> The compiler can **derive and verify a replacement stage0** from `Stage0Spec`. It cannot arbitrarily rewrite itself at runtime.

This precision matters. v2's pain was largely: "compiler contract changes → generated code changes → stage0 cannot regenerate enough of the compiler → manual stage0 edit required." v4 must structurally prevent that failure mode without falling into the opposite failure mode of free runtime self-modification (which is intractable to reason about).

### Three pipelines, not one

The architecture has **three distinct pipelines** that must not be collapsed:

| Pipeline | Input | Output | Surface |
|---|---|---|---|
| **1. Semantic compile pipeline** | user/source text (or `CoreNode` directly) | target source / eval Value | Per the rest of this doc — parse→normalize→resolve→ground→translate/serialize OR eval. The "compile" surface most users see. |
| **2. Artifact projection pipeline** | `InferredGraph` (the compiler's own modeled state, OR any user program) | generated compiler code / tests / docs / schemas / glue / diagnostics tables / stage1 compiler source | Lenses + projections per P3 + P8. Everything downstream of a model that's *not* the user-program target output. |
| **3. Bootstrap promotion pipeline** | current `stage0[k]` + candidate-next-compiler artifacts | `stage0[k+1]` (or rejection diagnostic) | The protocol that **replaces the active seed**. Guarded by `BootstrapWitness` + `FixedPointWitness` + promotion checks. |

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
Epoch k:
  Stage0[k] + CorePackage[k] ──> Stage1[k]
  Stage1[k] + SourceModels[k] ──> Stage1'[k]
  verify(Stage1[k] == Stage1'[k]) ──> FixedPointWitness[k]

Upgrade k → k+1 (when an upstream model changes):
  Edit SourceModels[k+1]
  current compiler computes:
    ChangeSet[k → k+1] + AffectedSet + RecomputePlan
  Stage0[k] + Bridge/CorePackage[k → k+1] ──> Stage1[k+1]
  Stage1[k+1] ──> Stage0Candidate[k+1] + Stage1Candidate[k+1]
  verify:
    Stage0Candidate[k+1] can produce Stage1Candidate[k+1]
    Stage1Candidate[k+1] is equivalent / fixed-point
    witnesses / tests / lenses pass
  promote:
    Stage0[k+1] = Stage0Candidate[k+1]
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
  lens algebras (including testgen lens family)

Layer 1 — Substrate models (.dag, source of truth)
  Node, fold_node, traverse, Outcome/Diagnostic/Locus,
  grammar-as-data, dependency graph, coercion fold,
  change/artifact/bootstrap substrate

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

### New bootstrap substrate (needed but not yet declared)

| Concept | Purpose | Status |
|---|---|---|
| **`Stage0Contract`** | What stage0 promises to do (consume canonical CorePackage; run minimal fold_node + traverse; fail-closed diagnostics; emit verified artifact). | Not declared. Likely `std/bootstrap.dag`. |
| **`BootstrapEpoch`** | The k-indexed snapshot of (stage0, CorePackage, SourceModels). | Not declared. |
| **`CorePackage`** | The stable canonical bootstrap package consumed by stage0. | Not declared. |
| **`CorePackageSchema`** | The schema describing what CorePackage carries (changes here are bootstrap-breaking per Case 3). | Not declared. |
| **`Bridge`** | Migration package for bootstrap-breaking changes (Case 3 of bootstrap-impact taxonomy). | Not declared. |
| **`BootstrapWitness`** | Proof that Stage0Candidate[k+1] can produce Stage1Candidate[k+1]. | Not declared. |
| **`FixedPointWitness`** | Proof that Stage1[k] compiles to itself (compiler self-emits, THESIS facet 2). | Not declared. |
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

A change to an upstream model — Node shape, grammar productions, `LanguageModel`, `ModelCore`, dependency kind, lens contract, coercion-fold witness shape, or `HostModel` — must produce a `ChangeSet`. The system computes the affected downstream compiler artifacts (via T-21 `affected_set`) and regenerates them where derivable. If a downstream artifact cannot be regenerated from the updated model, the compiler **fails closed** with an explicit diagnostic.

**No compiler layer may depend on another layer through:**
- Implicit convention.
- Manual synchronization (developer chases the change across layers by hand).
- Hidden emitter (executable emitter logic smuggled into a model file).
- Hidden parser (token-by-token recognition that bypasses grammar-as-data).
- Hidden test generator (special-case test-emission code outside the lens framework).
- Hidden integration updater (per-pair language adapter code that should be derived).
- Hidden layer-specific patcher (ad hoc walker fixing up stale output of an upstream stage).
- Ad hoc Node traversal outside `fold_node` / `traverse_node` / `apply_diff` / lens primitives (Practice 10 row 1 violation, applied to compiler internals).

**Every compiler-internal artifact must be one of:**
- A substrate primitive (declared in `std/`).
- A declared algebra over a substrate primitive.
- A lens (in the read/edit framework).
- A projection (P3 commitment 1; output of a lens).
- A target/runtime policy (declared in `extdeps/`).

**The slogan.** *The compiler is the first consumer of its own modeling discipline.* If implementation workers find themselves writing ad hoc code to keep compiler layers in sync, that is a **STOP condition** — escalate, do not patch.

**Worked example: testgen self-regeneration.** Suppose `Refinement<T>` (in `std/refinement.dag`) gains a new field. A v2-style compiler would require manual updates to every place that emits boundary-tests-for-refinements, every target-language test renderer, every fixture template. The v4 compiler:

1. `affected_set` computes the downstream artifacts whose inputs include `Refinement<T>` (TestClaimLens, BoundaryTestLens, every TargetTestProjection that reads boundary tests).
2. Each affected projection regenerates from the new `Refinement<T>` shape via its declared lens algebra.
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
| **`Projection`** | A declared projection-as-data: name, input carrier, output artifact, dependency requirements, regeneration algebra, validation lens set. | Not declared. |
| **`Artifact`** | An emitted artifact (generated source file, test file, schema, diagnostic table, compiler stage table) with its provenance (which projection produced it from which model state). **`ArtifactKind` reserves bootstrap-related variants up front** — `Stage0Candidate`, `CorePackage`, `WitnessBundle` — so P9 bootstrap substrate can land later without forcing an artifact/projection refactor. | Not declared. |
| **`RecomputePlan`** | Topological schedule + SCC/fixpoint groups + cached-vs-invalidated artifacts + regeneration ordering. The "how to recompute" data given an `AffectedSet`. | Not declared. |

**Implementation order:** declare the regeneration substrate as a unified cluster (likely `std/regeneration.dag` + the bootstrap substrate in `std/bootstrap.dag`); land alongside or after the multi-lens dependency-management primitive (Open Q6). T-21's `affected_set` must be **migrated** to the unified `AffectedSet` shape, not left as a parallel concept — the unified declaration is the single source of authority.

### P7 — Lawful rewrite witnesses for structure-changing lowerings

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

The coercion fold (P6 / primitive set) checks **exact structural preservation** between two canonical groundings — it is a mechanical zip-fold, decidable by construction, NOT heuristic. But some legitimate target lowerings change structure:

- Sequential `map` → parallel `map`.
- Left fold → tree reduce.
- CPU loop → CUDA kernel launch + device transfer + barrier + copy-back.
- Sequential reduce → MapReduce shuffle + per-key reduce.
- Iterator chain → fused single-pass loop.

These are valid under the algebra's laws (map's per-element independence, reduce's associativity, etc.) but they are NOT exact structural equality.

**Resolution:** structure-changing lowerings require a separate **`LawfulRewriteWitness`** primitive. The pipeline becomes:

```
canonical source grounding
  ── LawfulRewriteWitness (declared by target/runtime model) ──>
target-plan grounding
  ── coercion fold (exact structural zip-fold) ──>
target Node tree
```

The `LawfulRewriteWitness` is the primitive that proves a structure-changing rewrite preserves semantics under declared algebraic laws. Each rewrite is **declared by the target/runtime model** (e.g., `extdeps/runtimes/cuda.dag` declares "sequential map over Vec<T> may rewrite to CUDA kernel iff per-element function is pure + has no cross-element data deps"). Searches and rewrites remain **closed and decidable** — candidates are declared, every rewrite produces a checkable witness, none is heuristic.

This preserves the "coercion fold is not heuristic search" principle while leaving room for MapReduce / CUDA / parallel-map / fusion lowerings. Without this distinction, implementation workers would have to choose between (a) cementing CUDA/MapReduce into the compiler (Practice 10 row 3 failure) or (b) extending the coercion fold to do heuristic search (T-9 D2-reversal violation). Both are wrong; P7 names the third path.

---

## The primitive set — the things the compiler IS

| Primitive | Purpose | Status |
|---|---|---|
| **`fold_node`** (catamorphism over Node) | Generic structural recursion over Node trees; anti-walker-dissolution substrate (Practice 10 row 1 of the modeling-discipline rubric). Every "pass" the compiler runs is `fold_node(tree, algebra)`. | **Landed** in `src/v4/std/node.dag` line 47, with the `NodeFold<R>` algebra carrier type. |
| **`traverse`** over `Outcome<T>` | Effect threading. When a fold's algebra produces `Outcome<T>` (success-or-diagnostic), `traverse` propagates failures and short-circuits without inline `match` ladders (Practice 10 row 2 — "traverse dissolution"). | **Uncertain.** Not explicitly visible in `std/`; may be implicit in `fold_node` with short-circuiting algebras. See Open Q4. |
| **Grammar-as-bidirectional-data** | One declarative grammar model in `extdeps/languages/X.dag` serves both parsing (text → Node) and serialization (Node → text). No separate parser and printer; the grammar production data IS the relation (Practice 8 — "No string-templating"). **Default law:** `parse_target(serialize_target(target_node)) == target_node` — serialization may canonicalize whitespace, comments, optional delimiters, and equivalent syntactic forms; full source-text round-tripping (Q12c) is an opt-in trivia-preserving tool mode, NOT the compiler default. | **Partially landed** — `Grammar / ModeledGrammar / VoidGrammar` declared in `src/v4/compiler/02_parse.dag`; consumed by `extdeps/languages/dag.dag`. Inverse-grammar walk (the serialize direction) is scaffolded but not implemented. |
| **`solve_constraints`** | Declared constraint-solving substrate primitive. Consumed by ground / infer. Takes a `ConstraintGraph` produced by the grounding fold; produces a `Outcome<InferredTree>` containing the **canonical source grounding** (unique by ratified Q-canonical invariant — see Ground stage). Must produce checkable witnesses or fail-closed diagnostics. **Distinct from the coercion fold** — solver is the upstream machinery; coercion fold is the downstream exact-preservation check. | **Not yet declared.** Likely lands in `std/inference.dag` or extension of `std/cardinality.dag`. |
| **The coercion fold** | THE homomorphism-verification primitive. Given two **canonical Node groundings** (the canonical source grounding produced by `solve_constraints`, and the target language's closed declared candidate inhabitants), a **mechanical zip-fold** (catamorphism) walks both in parallel and checks structure preservation. Decidable by construction over the closed declared candidate set; empty candidate ⇒ Diagnostic (fail-closed), never a fabricated coercion. **Strictly exact**: structure-changing rewrites (parallel-map / tree-reduce / CUDA / MapReduce lowerings) are NOT performed by the coercion fold. Output carrier carries an **extension slot for future LawfulRewriteWitness**: conceptually `TranslatePlan = ExactCoercion(source_grounding, target_node) \| RewrittenThenCoerced(rewrite_witness, target_plan_grounding, target_node)`. MVP implements only `ExactCoercion`; the `RewrittenThenCoerced` variant lands with P7's LawfulRewriteWitness substrate without requiring refactor. | **Not yet implemented.** Substrate underlying it (canonical-form fold via `node.dag`'s B1-CANON contract, `content_hash = merkle_fold ∘ canonical`) exists. Ratified per `src/v4/TASKS.md` T-9 (line 418, D2-reversal 2026-05-17). Earlier project text (THESIS line 181, "structural algebra-homomorphism search") was superseded by this ratification; the doc-level reconcile is pending. |
| **`LawfulRewriteWitness`** | Per P7. Witness primitive for structure-changing target lowerings — sequential→parallel map, left-fold→tree-reduce, CPU loop→CUDA kernel, sequential reduce→MapReduce. Each rewrite is declared by the target/runtime model (e.g., `extdeps/runtimes/cuda.dag`) with its precondition algebra laws (per-element independence, associativity, etc.). The witness composes with the coercion fold: rewrite first, then exact-zip-fold check on the rewritten plan grounding. Keeps "not heuristic search" while supporting structure-changing lowerings. | **Not yet declared.** Substrate-design work; likely lands in `std/rewrite.dag` or similar. Out of scope for the initial single-target compiler but architecturally enabled. |
| **Diagnostic + Locus carrier** | Fail-closed reporting (Practice 1). Every failure path goes through a structured `Diagnostic` with source `Locus`. | **Landed** in `src/v4/std/diagnostic.dag`. |
| **Typed dependency graph + topological/SCC machinery** | Per P6, dependencies are first-class typed edges (not incidental containment). The substrate declares `DependencyEdge`, `DependencyKind`, `DependencyGraph`, `AcyclicityWitness`, SCC condensation, topological-order derivation. Carriers (post-resolve onward) carry the graph alongside the containment tree. | **Not yet declared.** Probably lands in `src/v4/std/dependency.dag`. T-21's `affected_set.dag` is the incremental-rebuild specialization — built on the same substrate. |
| **`apply_diff` (write-side primitive)** | The structural-edit primitive symmetric to `fold_node`. Takes a `Node` and a `Diff` (a `List<Edit>` where `Edit = { at: Path, replacement: Node }` and `Path` is a structural address into the Node graph). Folds the edits sequentially into a candidate Node; **fail-closed all-or-nothing** if any Path fails to resolve. The agent-side mutation primitive. Used by query-driven rewrite, mechanical refactor, self-modification, and the candidate-state gate pattern. See [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) for the full read/edit pipeline. | **Vocabulary ratified** (`Path / Edit / Diff` in `src/v4/std/node.dag` per PR #3162). **Operations scaffold** in `src/v4/lens/application.dag`; T-23 fills them. |

That is the full compiler-primitive surface. **Nine things.** Every stage, every pass, every translation, every eval, every glue derivation, every read/edit operation is structurally `fold_node + traverse + (one or more of the other seven) + an algebra-as-data`. **Reads** are folds (via `fold_node` / lens algebras); **writes** are `apply_diff`; the substrate is read/write-symmetric at the primitive layer.

**Why this terminology shift matters (coercion fold vs. "search"):** an earlier framing called the homomorphism step an "algebra-homomorphism search" or "inhabitance search," which suggests an open-ended search algorithm with potentially intractable complexity. The ratified position is the opposite: the candidate set is **closed and declared** (by the target language model), the source's grounding is **canonical and unique**, and the check is a **mechanical structural zip** — closer to type unification than to logic-programming search. If you find yourself reasoning about it as a search, that's a signal you're solving the wrong problem.

---

## The pipeline — stages as named algebras

```
  ┌───── INGEST LAYER (separable, multiple entry points) ─────┐  ┌──── COMPILE CORE (data → data) ────┐

  text ── parse(input_lang) ── normalize ─────┐               ┌────── resolve ── ground ──┬── translate(target_lang) ── serialize(target_lang.grammar) ── target_text
  (or)                                        ├──> CoreNode ──┤                            └── eval(host_model) ── execute(host)
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
| `eval(host_model)` | COMPILE-CORE | host Value | host_model's interpretation algebra |

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

`ModelCore` is the shared base for both `LanguageModel` and `HostModel`. Declares:
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

### `HostModel` (extends `ModelCore`, ratified Q1a)

Structurally a peer of `LanguageModel` over the same `ModelCore`. In addition to `ModelCore`:
- **Host value representation** (how a substrate primitive is represented at runtime).
- **Primitive operation interpretation** (Transform → host function call; Branch → host if-expression; Value → host literal allocation).
- **Execution semantics** (call/return, allocation, control transfer).
- **Resource / effect boundary** (FFI, host exceptions, async/concurrency model, identity/reference semantics).

The shared `ModelCore` ensures that algebra-inhabitance and primitive-type declarations are consistent between language and host targets. Host-side concerns (allocation, runtime values, resource limits) are categorically different from emit-side concerns (grammar, serialization) and live only on `HostModel`.

### `TargetSource` / `Value`

Terminal output. `TargetSource` is target-grammar-serialized text + diagnostics. `Value` is the host's runtime representation + diagnostics.

---

## The EDIT direction — symmetric to compile

This doc is primarily about the **compile direction** (CoreNode → target). But the substrate is read/write symmetric: `apply_diff` is the write-side primitive that mutates a `CoreNode` graph. The full **EDIT direction** is documented in [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) (PR #3364, merged 2026-05-19, operator-ratified); this section summarizes the architecture that compile must compose with.

### Read/edit primitives

| Primitive | Purpose | Where |
|---|---|---|
| **`Path`** | A structural address to a sub-Node — `Path { steps: List<Symbol> }`. NOT a filesystem path; the Node graph's own coordinates. | `src/v4/std/node.dag` (ratified PR #3162) |
| **`Edit`** | A structural rewrite: `Edit { at: Path, replacement: Node }`. **Replacement only** — no separate insert/delete variants. Insertions/deletions decompose into parent-replacement. | `src/v4/std/node.dag` |
| **`Diff`** | An ordered sequential rewrite program: `Diff { edits: List<Edit> }`. Sequential composition, NOT parallel. | `src/v4/std/node.dag` |
| **`apply_diff(root, Diff) → Result<Node, Diagnostic>`** | Folds Edits in order; fail-closed all-or-nothing. Any unresolved Path ⇒ whole Diff fails with one Diagnostic. | `src/v4/lens/application.dag` (scaffold; T-23) |
| **`subterm_at(root, Path) → Result<Node, Diagnostic>`** | Inverse of substitution — fetch the Node at a Path. | `src/v4/lens/application.dag` |
| **`apply_lens(lens, scope, mode) → Witness<finding> \| Outcome<()>`** | The read-side lens application surface — Introspect or Enforce. `scope: SectionRef = DeclarationScope \| NodeScope`. | `src/v4/lens/application.dag` |
| **`affected_set(dag, Diff) → Witness<ReExecFrontier>`** | Incremental re-execution frontier — what needs re-validation given a Diff. The substrate's lens-frontier primitive. | `src/v4/lens/affected_set.dag` (T-21) |

### The seven-step read→edit pipeline

Per [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) § 4, the agent loop is:

```
1. Read       →  apply_lens(lens, scope_in(dag, ref), Introspect)
2. Diagnose   →  reasoning over the facts (LLM or human)
3. Propose    →  produce Diff
4. Candidate  →  candidate_dag = apply_diff(dag, Diff)   # uncommitted candidate
5. Gate       →  affected_set(dag, Diff).frontier.for_each(ref =>
                   apply_lens(_, scope_in(candidate_dag, ref), Enforce)
                 )                                       # gates against CANDIDATE
6. Commit     →  dag := candidate_dag                    # only if all gates pass
7. Re-emit    →  emit per target language                # files are a downstream effect
```

**The candidate-state pattern is structurally important.** Gates run against the **post-edit candidate state**, NOT the pre-edit graph. Reasoning: gating the pre-edit graph and applying the Diff afterward would let a Diff introduce a post-edit invariant violation that never gets enforced. The candidate pattern keeps the fail-closed promise honest: validation happens against the state that will be committed/emitted, not against a state already known to be valid.

The `scope_in(root, ref)` helper binds a frontier ref to a specific root, making the candidate-vs-pre-edit context structurally explicit in every gate call — a worker can't accidentally validate against the wrong root.

**This is the same monotonic-facts invariant** as the compile direction's Practice 3 (P3 stage-by-stage facts), but applied to mutation: facts flow forward through the candidate; the commit either lands all of them or none.

### EDIT direction = the "Query-driven rewrite" ingest path

In the "Ingest paths" table earlier, **"Query-driven rewrite"** — using `affected_set + apply_diff` to produce a new `CoreNode` — IS the read/edit pipeline. The compile direction takes the resulting CoreNode and proceeds normally. From the compile direction's view, query-driven rewrite is just another entry point producing a CoreNode; from the read/edit doc's view, it is the canonical agent workflow.

### Library-first agent surface

Per [`docs/design-read-edit-pipeline.md`](design-read-edit-pipeline.md) § 6.10:

1. **All substrate primitives surface as library functions.** `apply_lens`, `apply_diff`, `subterm_at`, `affected_set`, `declarations_in` are library calls. Not RPC services. Not CLI-first.
2. **CLI wraps libraries.** `gunbc apply-lens ...`, `gunbc auto-fix ...`, `gunbc refactor ...` are thin shells over library calls.
3. **The library boundary IS the agent-substrate surface.** No premature transport layer (JSON-RPC, gRPC). Network transport is a future concern that doesn't pre-date library landings.

**This applies to `compile()` too.** `compile(source, input_lang, mode)` is a library call. The same no-premature-transport principle holds — compile is a library, not a service.

### Lens as `(find, transform)` — the convolution view

Per the read/edit doc § 6, every L1.x dissolution lens is structurally a `(find, transform)` pair: the lens's Signature is the find half, the Clean shape is the transform half. The read/edit doc's § 6.7b "**(f) mechanical refactor**" hero case is the worked example most relevant to this compiler architecture: a single intent → uniform per-site transform → atomic apply via the candidate-state pattern. PR #3338's canonical-B refactor (across 6 languages + 7 ratchet dissolutions) is the worked example. This is also what `compile`-level rewrites (e.g., self-modification, see below) decompose into.

### Projections — what lenses produce

Per P0 + P3, **every downstream artifact is a projection of `InferredTree`** computed by a lens (a fold over the tree). The compiler core produces `InferredTree`; lenses fan out into the projection family. Naming things by their function rather than historical accident:

| Projection | Lens (kind) | Output | Notes |
|---|---|---|---|
| Target source code | `translate(target_lang)` + `serialize` | `TargetSource` | The compile direction; NOT a lens — it's the homomorphism (P1). Listed here because conceptually it's "one projection among many." |
| Host evaluation | `eval(host_model)` | `Value` | Same — homomorphism with terminal = execute (P2). |
| Dimensional facts (cost / complexity / ownership / parallelism / termination / idempotency / effect) | dimensional lens family | `DimensionFact` Witnesses | All consume InferredTree; produce side-channel facts (P3 commitment 3). |
| **Test cases** | **testgen lens(es)** | `TestClaim` Witnesses | **Reframed:** testgen is a lens, not a separate compiler concept. Historical naming. The various flavors of test generation (boundary tests from refinements, property tests from algebraic laws, integration tests from protocol models, serialization roundtrip tests, target-equivalence tests, fuzz cases from cardinality boundaries, …) are **different invocations of the same lens family** parameterized by which structural facts to project. Tests are NOT the source of truth for the model; they are **behavioral probes** generated FROM the model. See "Testgen as a lens" below. |
| **Dry-run / simulated execution** | **lens-identifies + `LawfulRewriteWitness`-substitutes + Eval(mock_host_model)** | `Value` (in a sandboxed eval) | **Reframed:** dry-run is composition of three primitives we already have, not a new compiler concept. See "Dry-run as a lens-composed projection" below. |
| Inter-module glue (REST client/server, marshaling, schemas) | composed homomorphism via shared transport model (P4) | per-module `TargetSource` + glue artifacts | The omni-stack story. Cross-link to P4. |
| Migration / API-compat plans | diff-over-Trees + affected-set fold | structural change report + per-site remediation | Change-consequence story; uses `affected_set` (T-21). |
| Affected-rebuild set | `affected_set(dag, Diff)` | `Witness<ReExecFrontier>` | Incrementality primitive; consumed by ALL projection regeneration. |
| Documentation / API docs | a doc-projection lens | structured doc data + rendered text | A projection like any other; not a separate tool. |

The compiler core does NOT name any of these specific projections (P3 commitment 4). Each lens is plug-in data; the compiler's job is to enable the fold mechanism and the candidate-state gate.

### Testgen as a lens family

The current name `lens/testgen.dag` is historical — predates the lens reframe. Conceptually, **testgen is a lens family that produces test artifacts as projections of `InferredTree`.** Different "levels" of testgen are different invocations, profiles, or dependent lenses over the same verification substrate.

**Three-layer structure** (per reviewer 2026-05-20):

| Layer | Lens | Input | Output |
|---|---|---|---|
| **Abstract claims** | `TestClaimLens` | `InferredTree` | `TestClaim` Witnesses — roundtrip-law claims, refinement-boundary claims, algebra-law claims, protocol-compatibility claims, effect/idempotency claims |
| **Concrete cases** | `TestCaseLens` | `TestClaim` Witnesses | `TestCase` Witnesses — examples, boundary cases, property-test generators, fuzz seeds, regression fixtures (lowering of abstract claims) |
| **Target-specific files** | `TargetTestProjection` | `TestCase` Witnesses + target `LanguageModel` | `TargetSource` for tests (Rust `#[test]`, Python pytest, TS Jest, integration harness, …) — uses translate's homomorphism applied to test data |

The pipeline:
```
InferredTree ── TestClaimLens ──> TestClaims
TestClaims    ── TestCaseLens ──> TestCases
TestCases     ── TargetTestProjection(target_lang) ──> target test source
```

Each step is a fold (P3 + lens framework); each step's output is data consumable by the next.

**Profile invocations** are different lens parameters / dependent lenses on the same family:

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
- `std/testgen.dag` (lens family library) — exists as `lens/testgen.dag`; pending the rename to fit the lens-family naming convention.

**Lens-to-lens dependencies** are first-class (Open Q6 multi-lens primitive): `testgen.property` may depend on `testgen.algebra_law`'s output; `testgen.integration` may depend on `testgen.roundtrip`. These compose via the lens-dependency DAG (P3 commitment 5), not via ad hoc calls — Practice 10 row 1 applies to lens internals too.

**Important boundary (P0 implication):** tests are projections OF the model; the model is NOT derived from tests. Tests are **behavioral probes** generated from the source-of-truth model. The slogan: *"tests are generated from the model as behavioral probes; they are not the source of truth for the model."*

**Self-regeneration:** when a model fact changes (e.g., `Refinement<T>` shape changes per P8's worked example), `affected_set` computes which TestClaims/TestCases/target test files need regeneration; the relevant lens family invocations re-fire automatically on the affected subgraph. No manual sync.

### Dry-run as a lens-composed projection

**Dry-run / simulated testing** — running a program without committing its I/O side effects — is NOT a new compiler concept. It's the composition of three primitives we already have:

1. **A lens identifies the I/O boundaries.** `io_boundary_lens(InferredTree) → Witness<List<IOBoundary>>` — fold over the tree, find every Node whose inhabitance involves an effectful primitive (network call, DB write, file I/O, time, randomness, host syscall). The lens reads `EffectDependsOn` / `ResourceDependsOn` edges from P6 to determine what counts as I/O.
2. **A `LawfulRewriteWitness` substitutes mock stubs.** The substitution is target-declared: `dry_run.dag` (a runtime/policy model) declares "for each I/O primitive in the source's effect algebra, the rewrite produces a mock-host call that records-and-returns a canned value." The rewrite is witnessed by a "this preserves observable semantics modulo committed I/O" law per P7.
3. **Eval with a mock host model.** `eval(mock_host_model)` runs the rewritten tree. The mock host's interpretation of the substituted operations records and returns mock values; nothing actually hits the real I/O surface.

```
InferredTree
  ── io_boundary_lens(Introspect) ──> List<IOBoundary>
  ── LawfulRewriteWitness(dry_run substitution) ──> InferredTree' (with stubs)
  ── eval(mock_host_model) ──> Value + recorded_io_log
```

No new compiler primitive needed. Dry-run is a **lens-composed projection**: lens identifies, rewrite substitutes, eval runs against a different host.

**The operator's intuition was right** — the actual interception happens via codegen-style substitution (the `LawfulRewriteWitness` produces an InferredTree with stub nodes where the I/O primitives were), not "the lens itself intercepts at runtime." Lenses are pure folds; they cannot intercept execution. But they CAN identify what needs intercepting; a rewrite witness does the substitution; eval runs the result.

**Variations** (different lens invocations / different mock_host_model):
- `dry_run.record` — execute, log every would-be I/O, return mock results.
- `dry_run.assert` — execute against pre-declared expected I/O sequence; diagnostic if mismatch.
- `dry_run.replay` — execute with prior I/O log as inputs (deterministic replay).
- `dry_run.fuzz` — execute with random / boundary mock values; collect outcomes.

Each is a different mock_host_model + optionally different lens for what counts as I/O (e.g., only network I/O vs all I/O). The compiler core machinery is unchanged.

### How tests and dry-run compose

`testgen` produces TestClaim Witnesses; `dry_run` provides a sandboxed execution of an InferredTree. A natural composition: `testgen` generates a set of TestClaim cases; `dry_run.assert` executes each against the implementation; failures become diagnostics. This is what the THESIS Tier 3 ("verification from structure") + the v4 `verification.dag` substrate enable — but composed from existing primitives, not as a special "test runner" subsystem.

---

## Self-modification, stage0, self-edit

> **Important framing (per P9):** the phrase "compiler editing itself" is loose. The precise framing is: the compiler **models** itself; the compiler **generates a candidate next-compiler** as an artifact; the candidate is **verified**; only the **promotion protocol** replaces the active stage0. The active compiler does NOT mutate itself in place at runtime. See P9 ("Bootstrap is stratified") for the full discipline; this section walks through the mechanics.

The compiler is itself authored in `.dag`. Self-modification — the compiler updating its own source — is therefore a natural composition of the EDIT direction (read/edit pipeline), the COMPILE direction (this doc), AND the bootstrap promotion pipeline (P9). This section addresses the specific question the read/edit doc reviewer raised: "how will the compiler edit itself?"

### The compiler reading its own source

The compiler's own pipeline (`compile.dag` / `parse.dag` / `infer.dag` / `emit.dag` / `eval.dag` / `normalize.dag` / `resolve.dag`, all in `src/v4/compiler/`) is itself a set of `.dag` programs. To the substrate, they are `CoreNode` graphs like any other. The compiler reads them by `apply_lens(lens, scope, mode)` on the relevant `DeclarationScope` or `NodeScope`. Reading the compiler's own source uses the **same lens surface** as reading any user program — there is no special introspection mechanism.

### The compiler writing to its own source

When a `.dag` substrate type changes — say `std/node.dag`'s `NodeFold<R>` carrier gains a new field — that change propagates through the compiler's own `.dag` source. Two valid mechanisms, both go through the read/edit pipeline:

1. **Hand-authored Diff via the read/edit pipeline.** The operator (or an agent) declares the change as a `Diff` against the substrate-typed Node graph. `apply_diff` mutates; the seven-step candidate-state pattern validates against the candidate. Commit lands. This is exactly the read/edit doc's "(f) mechanical refactor" hero case applied to the compiler's own source.
2. **Mechanical refactor from upstream change.** When a substrate type evolves, a lens (e.g., an L1.x dissolution lens, or a custom interface-cascade lens like read/edit doc § 6.5) computes the affected sites + the per-site transform. The substrate handles N affected sites in the compiler's own code the same way it handles N user-code sites. Atomic apply via candidate-state.

There is no "self-edit special case." Self-edit is `apply_diff(self, Diff)` where `self` is the compiler's own CoreNode graph. The substrate's read/write-symmetric primitive set means there's no hidden machinery — same surface, applied to the compiler's source.

### stage0 regeneration as a compile of self

The hand-Rust-to-zero trajectory (per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md)) requires the compiler to **emit its own Rust bootstrap**. Structurally this is a self-targeting compile, expressed as the composition of `ingest_text` + `compile` per the text/data separation — **compile-core consumes `CoreNode`, not text** (per Open Q-text-data ratification):

```
// Explicit composition: ingest is separate from compile-core.
self_corenode = ingest_text(
  input_text: read_file("compiler.dag"),    // boundary action: read text
  input_lang: dag_lang                       // grammar + sugar dissolutions
) -> Outcome<CoreNode>

stage0_source = compile(
  source: self_corenode,                     // already CoreNode — never text
  input_lang: dag_lang,                       // for resolve + ground
  mode: TranslateTo(target_lang: rust_lang)   // Rust is the target
) -> Outcome<TargetSource>                    // the emitted Rust bootstrap source
```

There is **no special "regenerate stage0" mode** and **no parse-and-normalize folded back into compile**. The pipeline is the same uniform composition as any other compile: `ingest_text` produces the CoreNode (at the system boundary, separable per the text/data ratification); `compile` consumes the CoreNode (pure data-in/data-out). The homomorphism mechanic applies identically: resolve + ground the `.dag`-substrate Node graph; coercion-fold against `rust.dag`'s declared inhabitants; serialize via Rust's grammar. The output is the Rust bootstrap that THESIS facet 2 ("compiler self-emits, fixed-point") requires.

**Equivalent paths** for stage0 regeneration (per P9's stratified bootstrap):
- `compile.dag` already loaded as `CoreNode` from prior ingest (cached) → `compile(self_corenode, dag_lang, TranslateTo(rust_lang))`.
- Programmatic compiler-model construction (e.g., from a `Diff`-derived candidate) → `compile(candidate_corenode, dag_lang, TranslateTo(rust_lang))`.

All paths go through `compile()` taking a `CoreNode`. The ingest boundary is explicit and separable.

This is why P1 (Languages are I/O integration surfaces) and self-hosting are not orthogonal: self-hosting IS the compiler taking itself as input and emitting itself as output, with both sides using the same `LanguageModel` substrate. The trajectory of hand-Rust-to-zero IS the same compile loop closing on itself.

### Candidate-state for self-modification safety

When the compiler proposes a change to itself, the candidate-state pattern from the read/edit pipeline is what makes self-modification **safe** — and per P9, this is the *only* path: candidate generation + verification + promotion. The active running compiler is never the candidate.

```
candidate_self = apply_diff(self, proposed_diff)
// Validate the candidate against all relevant lenses:
gates = [
  apply_lens(L_practice_10, scope_in(candidate_self, ...), Enforce),  // no walker dissolutions
  apply_lens(L_practice_9,  scope_in(candidate_self, ...), Enforce),  // no-prose discipline
  apply_lens(L_correctness, scope_in(candidate_self, ...), Enforce),  // substrate invariants
  // ... whatever lenses the project policy requires
]
if all_passed(gates):
  self := candidate_self     // atomic commit
  // optionally: regenerate stage0 via compile(self, dag_lang, TranslateTo(rust_lang))
else:
  reject with diagnostics
```

Self-modification can never produce a broken substrate because the candidate is validated against the same lens framework that validates user code. If a self-edit would break Practice 10 / Practice 9 / a substrate invariant, the lenses fire on the candidate and the commit fails. **The compiler cannot break itself silently** — the candidate-state + lens-gate pattern makes that fail-closed.

### Implications

- **No new substrate is needed for self-edit.** `apply_diff` + lens-gate is the mechanism. The compiler's `.dag` source is just data; the same primitives that handle user data handle compiler data.
- **Self-hosting is one specific compile invocation.** `compile(self, dag, TranslateTo(rust))` produces stage0 Rust. `compile(self, dag, Eval(host))` evaluates the compiler on the host. `compile(self, dag, TranslateTo(other_lang))` would emit the compiler in some other language — the substrate makes this trivial in principle.
- **The "Rust shrinks to zero" trajectory IS the loop closing.** When stage0 Rust is reliably regenerable from `compile(self, dag, TranslateTo(rust))`, the hand-maintained surface shrinks; per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md), the target is zero hand-authored Rust.

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

**OR Stage 5b — `eval(host_model)` (T-22)** would, instead of translating to Rust, walk the InferredTree with the host's interpretation algebra:
- `Arrow` becomes a host closure.
- `std::int::add` becomes the host's runtime add.
- Calling `add(2, 3)` produces `Value::Int(5)`.

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
- `src/v4/compiler/04_infer.dag`, `05_emit.dag`, `05_eval.dag`, `00_compile.dag` — scaffolds with **divergent interface shape** — see "Scaffold-vs-design divergence" below — and bodies not implemented.
- `src/v4/lens/application.dag` — `apply_lens` + `apply_diff` operations (scaffold; T-23).
- `src/v4/lens/affected_set.dag` — incremental-rebuild + lens-frontier primitive (T-21).

Falsification probes (not load-bearing for the initial homomorphism):
- `src/v4/extdeps/languages/` — verilog, llvm_ir, machine_code, ptx, lean (probes the 5-behavior thesis across heterogeneous domains).
- `src/v4/extdeps/formats/spice.dag` — physics simulation probe (no control flow).

## Scaffold-vs-design divergence — implementation migration plan

The current compiler scaffolds in `src/v4/compiler/` were authored before this design doc; some of their declared interfaces **predate the ratifications in this doc and are stale-pending-reconcile**. Implementation workers should treat these scaffolds as **shape placeholders**, not as the contracts to implement against. The migration is named below; landing it is part of T-9 / T-10 / T-22 implementation work.

| Scaffold | Current declared signature | This doc's ratified signature | Migration |
|---|---|---|---|
| `00_compile.dag` | `fn compile(source: Source, target: TargetModel) -> Outcome<TargetSource>` | `compile(source: CoreNode, input_lang: LanguageModel, mode: CompileMode) -> Outcome<Output>` where `CompileMode = TranslateTo(target_lang) \| Eval(host_interp)` | (a) Rename `Source` → use `CoreNode` from `std/node.dag` directly. (b) Add `input_lang` param. (c) Replace `target: TargetModel` with `mode: CompileMode` (sum type). (d) `Output` is mode-dependent. (e) Public terminal is `validate_then_compile` (lens gate); bare `compile` is internal/advanced surface per P3 commitment 6. |
| `04_infer.dag` | `import v4.lens.cost { SymbolicCost }` (line 13) | No lens import in compiler core (P3 commitment 4) | Remove the `v4.lens.cost` import. Cost is a lens that runs over `InferredTree` downstream — NOT a fact baked into infer's carrier. Practice 10 enforcement: a specific lens name imported in compiler core is a Row-3-direction violation. |
| `04_infer.dag` | Produces `InferredTree { typeshape, inhabitance witness, ... }` without explicit canonical-grounding invariant | Must produce *exactly one* canonical grounding per Node + the full P6 semantic dependency graph (`BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn`) | Add the canonical-grounding invariant to ground's contract. Surface ambiguity as a diagnostic in ground; translate never receives ambiguous source. |
| `05_emit.dag` | Single-step "emit" | Two steps under P7: optional `LawfulRewriteWitness` (target-declared) → exact coercion fold → serialize via grammar | When implementing emit, factor the two steps. Direct exact-preservation coercion is the default; `LawfulRewriteWitness` is the named extension point for structure-changing lowerings. |
| `05_eval.dag` | Standalone "primary execution path" with TestClaim runner | `eval(host_model)` — structurally `translate-to-host` per P2; differs from translate only in the terminal action (execute, not serialize) | Share the homomorphism mechanic with translate. Eval-specific machinery is only the host-interpretation algebra + the terminal `execute` action. |
| `00_compile.dag` `Source` carrier | Implicit text-typed source | `CoreNode` (data) — text is one of several ingest paths per the text/data separation | Replace `Source` carrier with explicit `CoreNode`. Text→CoreNode is the separable `ingest_text` operation, not part of compile-core's signature. |

**This is not a STOP on T-9 / T-10 / T-22 implementation** — workers implementing those tasks should land the design-ratified interface, not preserve the scaffold's stale signature. The scaffolds' purpose was to lock down stage existence + dependency order; the *exact signatures* in those scaffolds predate this doc's ratifications and are expected to be migrated.

If a worker encounters a deeper scaffold-vs-design contradiction not catalogued above, **escalate** — don't preserve the scaffold against this doc, and don't break the scaffold without recording the migration.

---

## MVP routes — named explicitly

Three named MVP endpoints, increasing in scope:

| MVP | What it proves | Required substrate + impl |
|---|---|---|
| **Core MVP** | The compile-core homomorphism produces a TargetSource from a CoreNode. Pure data-in/data-out; no shims. | `std/dependency.dag` + `std/constraints.dag` (solve_constraints) + W-CoercionFold + W-TraverseOutcome + W-T-9-impl + W-T-10-impl (translate + serialize) + scaffold reconciles. |
| **MVP-A (translate-only)** | Core MVP + the peripheral shim composes to end-to-end "file on disk → file on disk." Validates the I/O-as-modeled-effects framing. | Core MVP + `extdeps/file_system.dag` refresh + W-EmitShim (file_read → ingest_text → compile → file_write). |
| **MVP-B (eval-only)** | The compile-core homomorphism produces a Value via host interpretation. Validates P2 (eval = translate-to-host). | `std/dependency.dag` + `std/constraints.dag` + W-CoercionFold + W-TraverseOutcome + W-T-9-impl + `std/host.dag` (Ratified Q1) + W-T-22-impl + W-EvalShim. |

**Either MVP-A or MVP-B is sufficient as the first proof point**; both together is the strongest exercise of the architecture. MVP-A is the more direct demonstration of the homomorphism story (translate explicitly walks the coercion fold); MVP-B is smaller if Rust serialization proves harder than expected.

## What's NOT in scope for this design

- **Glue derivation / omni-stack** — orthogonal substrate (P4). Architecture must not preclude, but no implementation in the initial single-target compiler.
- **`extdeps/protocols/`** — missing substrate; deferred until glue is in scope.
- **Lens framework implementation** — out of initial compiler-core scope. The lens consumption contract is **ratified by P3 + Ratified Q0**; the multi-lens dependency-management substrate primitive remains Open Q6.
- **Per-target emitters** — does not exist (P1). Each new target is one new `LanguageModel`.
- **Multi-module compilation** — single-module first; module decomposition substrate (`std/system.dag`) is a follow-on.

---

## Open questions — input wanted

### Ratified Q0 — Lens architecture (2026-05-20)

**Resolved.** Lenses are a side-channel over `InferredTree`, sharing the `fold_node` primitive, with multi-lens dependency-management and a project-mandatory wrapper preserving the THESIS "by construction" guarantee. Six commitments per P3 above; B-style vs C-style surface is a non-load-bearing implementation choice. The new substrate question this raises is **Open Q6 — multi-lens dependency management** (below).

### Ratified Q1 — `HostModel` is `Q1a + factored ModelCore` (2026-05-20)

**Resolved** (reviewer-recommended, operator-direct). A shared `ModelCore` substrate carries primitive types, algebra inhabitance, laws, effect semantics, partiality. `LanguageModel` and `HostModel` both extend it (see "Carrier shapes — `ModelCore` / `LanguageModel` / `HostModel`" above). Host-side concerns (allocation, execution model, runtime values, resource limits) are categorically different from emit-side concerns (grammar, serialization) and live only on `HostModel`; the `VoidGrammar` variant (Q1b) conflated them. Q1c ("eval = translate-to-machine-code + execute") hides too much (host execution involves real process state, runtime values, failure modes) and was rejected.

### Ratified Q2 — `InferredTree` dimensional facts (2026-05-20)

**Resolved.** `InferredTree` carries **compiler-core semantic facts only**: locus, binding, typeshape, inhabitance witness, **and the P6 semantic dependency graph** (`BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn`). It does NOT carry dimensional lens outputs — cost, complexity, ownership policy, termination policy, parallelism decisions, optimization decisions — those are lens outputs (P3) produced as side-channel folds over `InferredTree`. The current `04_infer.dag` import of `v4.lens.cost.SymbolicCost` violates P3 commitment 4 and is a fix-needed.

### Ratified Q3 — Parse + normalize stay logically separate (2026-05-20)

**Resolved Q3a.** Parse and normalize are logically distinct stages. The reason is inspectability + correctness, not performance: while sugar dissolutions are being proven honest, the ability to inspect `(text → SurfaceNode → CoreNode)` separately is load-bearing. Diagnostics from normalize must point back to the original surface construct (Locus preservation), not synthetic substrate nodes. An implementation MAY later fuse parse + normalize for efficiency, but only if it can still expose the per-stage debug/proof artifacts.

### Ratified Q4 — Explicit `traverse_node` substrate primitive (2026-05-20)

**Resolved Q4a, with required additions.** Declare an explicit `traverse_node` in `std/node.dag` (or a peer file) that takes a fail-closed algebra. Per reviewer's required additions:

1. **Distinguish fatal rejection from diagnostic accumulation.** `Outcome<T>` shape should support both — likely `Accepted(value: T, diagnostics: List<Diagnostic>) | Rejected(diagnostics: NonEmptyList<Diagnostic>)` (see Open Q11 for the final accumulate-vs-short-circuit policy decision).
2. **Define laws.** `traverse_node` is a substrate primitive; it must obey predictable identity / composition behavior. Otherwise stage authors will create incompatible traversal semantics over time. Laws to specify: identity (`traverse(pure)` is no-op), composition (`traverse` over composed algebras = composition of traverses), failure-monotonicity (a Rejected anywhere produces Rejected overall, possibly with accumulated diagnostics depending on policy).
3. **Substrate companions.** Likely also `sequence_node`, `map_node_outcome` for common patterns. Avoid forcing each fold-author to recreate them.

Without the laws, Q4a degrades to Q4b convention — that's the trap.

### Ratified Q5 — Resolve stays its own stage; `LanguageModel` expanded (2026-05-20)

**Resolved Q5a.** Resolve is its own stage. Parsing is not responsible for binding — binding has too many nonlocal concerns (lexical scope, imports, modules, namespace separation, function/type/value namespaces, shadowing, hoisting, recursive declarations, macros, visibility) that parse can't satisfy in one pass.

Per the reviewer, `LanguageModel` is expanded (see "Carrier shapes" above) to declare these explicitly:
- Binding / namespace / scope rules.
- Effect / partiality declarations.
- Version / dialect / edition metadata.

For Rust, Python, JavaScript, etc., binding behavior is not optional metadata — it is part of the language model.

### Open Q6 — Multi-lens dependency management substrate primitive

P3 commitment 5 requires that N lenses consuming the same `InferredTree` share traversal — no re-walks per lens. This is a substrate-design question, of the same flavor as the coercion fold's substrate primitive.

Three sub-questions:

- **Q6a.** Is there a substrate primitive `multi_lens_fold(tree, List<Lens>) -> Map<Lens, DimensionFact>` that coalesces shared traversals — single pass over the tree, each Node visited once, each lens's algebra applied at the right step?
- **Q6b.** How are lens-to-lens dependencies expressed? Example: the complexity lens reads the cost lens's output. Does the multi-lens fold compute a topological order over lens dependencies and stage the folds (cost first, complexity second)? Or do lenses produce intermediate values consumable by other lenses in the SAME pass (more complex but single-traversal)?
- **Q6c.** Are lens outputs themselves Node-graph data, so they are consumable by other lenses uniformly (and by downstream tooling like IDEs, build reporters)?

This is substrate-design work. Probably lands in a new `std/lens.dag` or extension of `std/node.dag`. Required before multi-lens execution is correct under Practice 3 (facts flow forward, no re-derivation).

### Open Q7 — `LanguageModel` contract: declarative data only, or executable predicates?

Is a `LanguageModel` purely declarative `.dag` data, or can it contain executable predicates / algebras / functions?

If executable behavior is allowed, what prevents a "hidden emitter" (e.g., a function on `rust.dag` named `emit_function_decl(node) -> Text` that's effectively the v2 `emit_rust` smuggled into the model) from sneaking back in? Or a "hidden parser" (executable token-by-token recognition that bypasses grammar-as-data)?

Preferred: declarative-data-only. Every part of a `LanguageModel` should be inspectable as data — including grammar productions, sugar dissolutions, binding rules. Any callable function on a `LanguageModel` is a Practice-10 hand-rolled-derived-operation smell.

But declarative-only is hard for some concerns (e.g., scoping rules in languages with macros that rewrite scope). Need to decide: are macro-handling languages out of scope, or does `LanguageModel` permit a typed-predicate dialect for them?

### Open Q8 — Diagnostic shape on coercion-fold failure

> **Note:** the prior draft of this Q proposed a "Q8a complete vs Q8b incomplete-bounded" fork. This was wrong and was struck — per `src/v4/TASKS.md` T-9 (D2 reversal ratification), the coercion fold is **decidable by construction over the closed declared candidate set**; there is no "search may have missed something" mode. Empty candidate ⇒ Diagnostic, always. The legitimate open question is the *shape* of that diagnostic, not whether incompleteness is allowed.

When the coercion fold fails (no target inhabitant exists for a source structure in the target's closed candidate set), the diagnostic must carry actionable provenance. The open sub-questions:

- **Q8a.** What structural mismatch caused the failure? The diagnostic should name WHICH algebra inhabitance the source required, WHICH target inhabitants were considered, and WHERE in the zip-fold the structural inequality first surfaced — not a generic "no inhabitant found."
- **Q8b.** Does the diagnostic include a *near-miss* — the target inhabitant that came closest, with a structural-diff explanation? Useful for IDE quick-fix suggestions; possibly costly to compute.
- **Q8c.** Is the diagnostic differentiated by *reason*: structural-shape-mismatch vs missing-algebra-inhabitance vs effect/partiality-mismatch vs version-dialect-mismatch? Each is a different user-facing remediation.

These are diagnostic-UX questions, not correctness questions. The coercion fold's correctness is settled (decidable-by-construction per T-9).

### Open Q9 — Independent witness checking

Is the coercion fold's output (a target Node + inhabitance witness) **independently checkable**? I.e., can a separate `verify_homomorphism(source, target, witness) -> Bool` function consume the witness and re-prove structure preservation **without re-running the coercion fold's candidate enumeration**?

Strongly preferred (per reviewer): the **derivation is untrusted; the witness check is trusted.** This matches fail-closed philosophy — the compiler doesn't have to trust its own derivation; the witness is verifiable by inspection. Useful even though the coercion fold is decidable by construction, because (a) it gives downstream tooling a cheap re-validation path, and (b) it provides a clean abstraction boundary between "find the inhabitant" and "verify the inhabitant satisfies structure preservation."

### Open Q9b — Ambiguity surface when multiple candidates pass structure preservation

(Related to Q14 — target selection policy.) The coercion fold over a closed candidate set may find **zero, one, or many** inhabitants whose structural check passes. T-9's contract handles "zero ⇒ Diagnostic" cleanly. The **many** case is the ambiguity question: which target inhabitant gets picked, and is the selection part of the coercion fold's contract or a separate target-policy layer?

This isn't a question about completeness or search — both candidates are *valid* under structure preservation. It's about deterministic selection in the legitimately-ambiguous case.

### Open Q10 — Partiality and effects in `ModelCore`

How are these declared on `ModelCore`?
- **Partiality** — division by zero, integer overflow, out-of-bounds, force-unwrap.
- **Effects** — exceptions, mutation, async, host I/O, allocation failure, nondeterminism, nontermination.

These affect whether a homomorphism is honest — a source that's pure can't be translated to a target where the corresponding operation is effectful, without surfacing the effect. Practice 8 fact-bundle modeling for languages must include these. Need to decide the substrate representation:
- Effect types as fact-bundles (`{ pure: Bool, mutates: List<Resource>, ... }`)?
- Effect rows in the type system?
- Effect graph as P6 dependency edges (`EffectDependsOn`)?

### Open Q11 — `Outcome<T>` accumulate vs short-circuit policy

Per Q4's ratification, `Outcome<T>` likely accumulates diagnostics. What's the per-stage policy?

- **Q11a.** Every stage accumulates all local failures; reports them all at the end of the stage.
- **Q11b.** Every stage short-circuits at the first failure for performance.
- **Q11c.** Per-stage choice — some accumulate (parse, ground — UX wants all errors), some short-circuit (translate — once one Node fails, downstream is meaningless).

Reviewer recommends Q11c. The per-stage policy is part of each stage's algebra contract.

### Open Q12 — Bidirectional grammar law

What law does grammar-as-bidirectional-data satisfy?

- **Q12a.** Full bidirectionality: `parse(serialize(node)) == node` AND `serialize(parse(text)) == text`. The strongest law; usually impossible because parsing is many-to-one (whitespace, comments, optional syntax).
- **Q12b.** Target stability only: `parse_target(serialize_target(target_node)) == target_node`. The serialize direction is canonicalizing (picks one canonical form); the parse direction recovers structure.
- **Q12c.** Source-text-faithful: `serialize_source(parse_source(original_text)) == original_text` — requires preserving comments, whitespace, trivia. Opt-in for round-trip code-mod tooling.

Reviewer recommends Q12b as the default (this is what translation needs); Q12c is opt-in for tools that need fidelity.

### Open Q13 — Language versions / dialects

Rust editions, Python 2 vs 3, TypeScript strict mode, C++ standards (98, 11, 14, 17, 20, 23) — where do these live?

Probably as fields on `LanguageModel`. But the question is whether multiple versions are *one* `LanguageModel` parameterized by version (e.g., `rust.dag` with a `version: 2021` field) or *separate* `LanguageModel`s (`rust_2021.dag`, `rust_2024.dag`, …). The former is more parsimonious but may not be honest if the language is fundamentally different across versions (Python 2 vs 3 is the canonical example).

### Open Q14 — Target selection policy under multiple valid homomorphisms

Per the reviewer: a source `Map` or `Transform` may be representable in a target as a loop, an iterator combinator, recursion, a library call, a macro, or a SIMD/vectorized primitive — multiple structure-preserving choices. The compiler needs a deterministic selection policy:

- **Q14a.** First-declared in the target model (positional priority).
- **Q14b.** Lowest-cost per cost-lens (requires lens-as-input to compile — collides with P3 unless via target-model policy).
- **Q14c.** Canonical / project-policy declared on the target model (the target language declares its own preferences).
- **Q14d.** Surface ambiguity to the user — multiple valid translations, user picks per call site.

Recommendation: Q14c default + Q14d opt-in for advanced surface. Q14b is the "let optimizer choose" version but requires the lens framework to be part of the compile decision, which violates P3 — it can live in a wrapper, but not in compile-core.

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
| **Practice 10** | "Don't hand-roll a derived operation" — every fold/walker/translator that's mechanically determined by a Node's shape must use a substrate primitive (`fold_node`, `traverse`, coercion fold), not hand-rolled recursion. |
| **T-8 / T-9 / T-10 / T-22** | Task IDs in `src/v4/TASKS.md`. T-8 = normalize+resolve; T-9 = ground (infer); T-10 = translate; T-22 = eval. |
| **D2 reversal** | A 2026-05-17 operator-ratified design course correction (recorded in `src/v4/TASKS.md`) that reshaped how language fact-bundles and the coercion fold are framed. The current "coercion fold (not search)" framing is post-D2. |
| **fail-closed** | A discipline (Practice 1): every failure path produces a structured Diagnostic; no silent `None`s or panics. |
| **`ModelCore`** | The shared substrate of `LanguageModel` and `HostModel` (per Ratified Q1) — primitive types, algebra inhabitance, laws, effect semantics, partiality. |
| **DependencyEdge / DependencyKind** | Typed edge in the P6 dependency graph; kinds include `Contains`, `BindsTo`, `TypeDependsOn`, `DataDependsOn`, `EffectDependsOn`, `ResourceDependsOn`, `ModuleDependsOn`, `BarrierBefore`, `PlacementDependsOn`. |
| **synthesized attribute** | A fact computed bottom-up from a Node's children (e.g., typeshape, content_hash). Naturally fits `fold_node`. |
| **inherited attribute** | A fact propagated top-down from a Node's parent / context (e.g., scope environment in resolve). Requires a higher-order algebra carrier per P5. |
| **SCC condensation** | Strongly-connected-component reduction: collapsing valid cycles in a dependency graph into single nodes so the result is a DAG. Used to handle mutually-recursive definitions and other cyclic-but-valid structures. |
| **`solve_constraints`** | Declared constraint-solving substrate primitive consumed by ground / infer. Takes a `ConstraintGraph`, produces a unique canonical source grounding (or an ambiguity diagnostic). Distinct from the coercion fold. |
| **`LawfulRewriteWitness`** | Witness primitive (P7) for structure-changing target lowerings — parallel-map, tree-reduce, CUDA, MapReduce. Each rewrite is declared by the target/runtime model with its precondition algebra laws; the rewritten plan grounding is then checked by the coercion fold's exact zip-fold. |
| **`ConstraintGraph`** | Intermediate carrier produced by the grounding fold, consumed by `solve_constraints` to produce the canonical `InferredTree`. |
| **`ClosedWorldDependencyWitness`** | Substrate witness that a region's dependency classification is complete; required before absence-of-edge can be interpreted as independence (e.g., for parallelism). |
| **canonical (source) grounding** | The unique, witnessed normalization of a Node's algebra-inhabitance facts. The coercion fold operates only on canonical groundings; ambiguity surfaces as a diagnostic in ground, never as multiple downstream candidates. |
| **ingest** | The (separable) layer that produces a `CoreNode` from a source (text, programmatic builder, query-driven rewrite, round-trip). The compile-core takes `CoreNode` and does not know how it was produced. |
| **ingest_text** | The text-to-`CoreNode` ingest path: `ingest_text(text, input_lang) → Outcome<CoreNode>`. Parse + normalize composed. Uses the grammar-as-bidirectional-data primitive (forward direction). |
| **`Path`** | A structural address to a sub-Node — `Path { steps: List<Symbol> }`. NOT a filesystem path; the Node graph's own coordinates. Ratified in `src/v4/std/node.dag` (PR #3162). |
| **`Edit`** | A structural rewrite: `Edit { at: Path, replacement: Node }`. Replacement only — no insert/delete variants; those decompose into parent-replacement. |
| **`Diff`** | An ordered sequential rewrite program: `Diff { edits: List<Edit> }`. Sequential composition, NOT parallel. Fail-closed all-or-nothing on `apply_diff`. |
| **`apply_diff`** | The structural-edit primitive symmetric to `fold_node`. Takes a Node and a Diff; folds edits sequentially; fail-closed on unresolved Path. The agent-side mutation primitive. |
| **`apply_lens`** | The lens application surface: `apply_lens(lens, scope, mode) → Witness<finding> \| Outcome<()>`. The substrate-level read-side primitive lenses use. |
| **`SectionRef`** | The scope shape for lens application: `DeclarationScope(decl_ref) \| NodeScope(node_ref)`. Two variants only — corpus-wide application is composition over the declaration set, not a separate scope. |
| **`scope_in`** | Helper `scope_in(root: Node, ref: NodeRef) → SectionRef` that binds a frontier ref to a specific dag root. Makes the candidate-vs-pre-edit root structurally explicit in every gate call (read/edit doc § 4). |
| **candidate-state pattern** | The read/edit pipeline's invariant: gates run against `candidate_dag = apply_diff(dag, Diff)`, NOT against the pre-edit graph. Validates against the state that will be committed, not against a state already known to be valid. |
| **`affected_set`** | T-21 substrate primitive: `affected_set(dag, Diff) → Witness<ReExecFrontier>`. Incremental re-execution frontier; consumed by the read/edit pipeline's gate step + by incremental rebuild. |
| **self-edit / stage0** | The compiler editing its own source via `apply_diff(self, Diff)`. Self-hosting (stage0 regeneration) is `compile(self, dag_lang, TranslateTo(rust_lang))` — no special mode. See "Self-modification" section. |
| **projection (lens output)** | Per P0 + P3, every downstream artifact (tests, dimensional facts, glue, schemas, migration plans, docs, dry-run results) is a *projection* of `InferredTree` computed by a lens. The compiler core produces InferredTree; lenses fan out into the projection family. |
| **testgen** | A lens family that produces `TestClaim` Witnesses as projections of `InferredTree`. Historical name predating the lens reframe; different "levels" (boundary / property / integration / roundtrip / equivalence / fuzz / regression) are different invocations of the same family. Tests are projections of the model, NOT the source of truth for the model. |
| **dry-run** | A lens-composed projection: `io_boundary_lens` identifies the I/O sites; a `LawfulRewriteWitness` substitutes mock stubs; `eval(mock_host_model)` runs the substituted tree. No new compiler primitive; composes lens + rewrite + eval-with-host-variant. Variations: record, assert, replay, fuzz. |
| **peripheral shim** | A user-facing convenience that composes modeled effects with compile-core. Examples: `ingest_text` (file_read + parse + normalize), `compile_file_to_file` (file_read + ingest_text + compile + file_write). Shims are NOT substrate primitives — they live outside the architecture's load-bearing surface. |
| **modeled effect** | A real-world I/O operation declared as substrate data in `extdeps/` (file_read, file_write, network call, process spawn, …) — carries `EffectDependsOn` / `ResourceDependsOn` edges per P6; visible to lenses; substitutable by `LawfulRewriteWitness` for dry-run. There are no implicit side effects in the architecture. |
| **`TestClaimLens`** | Lens layer 1 of testgen — produces abstract behavioral claims (roundtrip / refinement-boundary / algebra-law / protocol-compatibility / effect-idempotency) as `TestClaim` Witnesses. |
| **`TestCaseLens`** | Lens layer 2 of testgen — lowers `TestClaim` Witnesses into concrete `TestCase` Witnesses (examples, boundary cases, property-test generators, fuzz seeds, regression fixtures). |
| **`TargetTestProjection`** | Lens layer 3 of testgen — translates `TestCase` Witnesses into target-language test source via translate's homomorphism. |
| **`ChangeSet`** | A diff between two model versions (changed Nodes / edges / facts / witnesses / grammar productions / LanguageModel fields / lens contracts). Substrate for P0 + P8 consequence-recomputation. Not yet declared (`std/change.dag`). |
| **`AffectedSet`** | Downstream artifacts requiring recomputation given a `ChangeSet`. T-21 `affected_set` is the lens-frontier specialization; full `AffectedSet` over arbitrary projection graphs not yet declared. |
| **`Projection`** | Declared projection-as-data: name + input carrier + output artifact + dependency requirements + regeneration algebra + validation lens set. Not yet declared (`std/projection.dag`). |
| **`Artifact`** | An emitted artifact (source file, test file, schema, diagnostic table, compiler stage table) with its provenance (which projection produced it from which model state). Not yet declared (`std/artifact.dag`). |
| **`RecomputePlan`** | Topological schedule + SCC/fixpoint groups + cached-vs-invalidated artifacts. The "how to recompute" data given an `AffectedSet`. Not yet declared. |
| **`Stage0Contract`** | What stage0 promises (consume canonical CorePackage; minimal fold_node + traverse; fail-closed; emit verified artifact). Per P9. Not yet declared. |
| **`CorePackage` / `CorePackageSchema`** | The stable canonical bootstrap package stage0 consumes (per P9). NOT the evolving surface language — decoupled from it. Not yet declared. |
| **`BootstrapEpoch`** | k-indexed snapshot of (stage0, CorePackage, SourceModels). Per P9. Not yet declared. |
| **`Bridge`** | Migration package for bootstrap-breaking changes (Case 3 of P9's bootstrap-impact taxonomy). Not yet declared. |
| **`BootstrapWitness` / `FixedPointWitness`** | Verification artifacts for the promotion step (P9). Not yet declared. |
| **promotion protocol** | The guarded step that replaces `Stage0[k]` with `Stage0[k+1]`. Per P9 — the *only* "back edge" in the system. Not yet implemented. |
