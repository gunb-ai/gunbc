# v4 Compiler Architecture — Homomorphism-First Design

> **Status: DRAFT, under discussion.** Forum for review and amendment — comment, push back, propose alternatives. Decisions ratified here flow into implementation briefs.

## What this document is

A design reference for the **v4 compiler** in the `gunbc` project, written before implementation of the compiler's middle and back stages (the ones that turn parsed source into typed source, then into target output or evaluated values). It captures the architectural decisions we want to commit to before code is written, so implementation does not cement the wrong shape.

This doc supplements but does not replace [`THESIS.md`](../THESIS.md) (the project's goals) or [`docs/modeling-discipline.md`](modeling-discipline.md) (the modeling rules). It exists to make one specific question concrete: **what is the compiler, mechanically, if we take "The derived homomorphism" claim in THESIS literally?**

Intended audience:
- Project insiders deciding T-9 / T-10 / T-22 implementation shape.
- Newcomers trying to understand what the compiler is supposed to do.
- Reviewers checking architectural commitments against THESIS, MODELING, and INVARIANTS.

You can read it cold without prior context — the next two sections give that context. If you already know the project, skim to "TL;DR".

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
2. **Plus five substrate operations** that aren't `fold_node` but are needed:
   - A **diagnostic carrier** (`Diagnostic + Locus`, fail-closed reporting).
   - **Grammar-as-bidirectional-data** — one declarative grammar model that drives both parsing and serialization. No separate parser and printer.
   - **`traverse`** over the diagnostic-effect carrier (a Node fold/traverse that short-circuits or accumulates over `Outcome<T>` without re-derived match ladders).
   - **The coercion fold** — a mechanical zip-fold over two canonical-Node groundings that checks structure preservation. Per `src/v4/TASKS.md` T-9, this is **the** homomorphism-derivation primitive — decidable by construction, never a search.
   - **Typed dependency graph + topological/SCC machinery** — per P6, dependencies are first-class typed edges carried alongside containment.
3. **Plus three irreducible boundary actions** — read input text, write output text (or execute), report diagnostics. (The earlier "project" boundary-action mention from the original draft is removed — it was an artifact of the rescinded lens-as-compile-mode framing.)

Everything else — every "pass," every "stage," every "language-specific" anything — is an **algebra** plugged into `fold_node` or another declared substrate primitive. The compiler knows no specific language code (Rust, Python, even `.dag` itself, beyond an irreducible bootstrap-seed for `dag.dag`); language-specific *data* lives in `LanguageModel` declarations under `extdeps/languages/`.

**Algebras can be higher-order, effectful, constraint-bearing, or fixpoint-seeking (P5).** "Fold" doesn't mean "pure bottom-up." Inherited context, effects, constraints, and fixed-point analyses are first-class — carried by the algebra's signature, never as ad-hoc walkers.

**Dependency management is inherent to the substrate but only under enforced discipline (P6).** Synthesized (child→parent) facts ride `fold_node`'s natural order. Inherited (parent→child) facts ride higher-order carriers (P5). Memoization and incrementality ride content-addressed Node identity. The inherent property breaks the moment any pass walks Nodes outside the declared primitives — Practice 10 enforces this.

The doc that follows defines what `compile()` takes and returns, why each piece is shaped that way, what's already in the codebase, and what's still open for design.

---

## End-to-end I/O — the compile function

```
compile(
  input_text: Text,
  input_lang: LanguageModel,           // the parse-side language model
  mode: CompileMode
) -> Outcome<Output>

where CompileMode =
  | TranslateTo(target_lang: LanguageModel)    // produce target source
  | Eval(host_interp: HostModel)               // execute on host runtime
```

That is the full external surface of the compiler. **Three arguments** (`input_text` + two semantic parameters: `input_lang` and `mode`); one fail-closed `Outcome`. The `input_text` boundary is text-in (a side-effect at the system boundary, not part of the semantic configuration).

**`LanguageModel`** is a `.dag` model in `extdeps/languages/X.dag` — a fact-bundle declaring X's grammar (bidirectional), primitive types with their facts (width, signedness, range, …), algebra inhabitance (this type inhabits this algebra), and sugar dissolutions (how the language's surface forms reduce to substrate primitives).

**`HostModel`** is structurally the same shape as `LanguageModel` minus the grammar — execution is the terminal action, not text emission. Whether it's a distinct substrate type or a LanguageModel variant is **Open Q1** below.

**`Output`** is mode-dependent: `TargetSource` (bytes/text) for `TranslateTo`, `Value` (host representation) for `Eval`.

> **Note on Lens mode:** the original draft of this doc proposed a third `CompileMode` for running lenses (the analysis framework for complexity / cost / ownership / effects / user-defined dimensions). This was flagged as contradicting THESIS by an automated review. **Resolved 2026-05-20 — see P3 below.** Lenses are a side-channel over `InferredTree`, orthogonal to the emission/eval homomorphism. They're invoked via a project-mandatory wrapper (`validate_then_compile`-style) that gates emit/eval on lens pass; whether the lens fold runs inside or outside the bare `compile()` is a non-load-bearing surface choice.

---

## Architectural premises (ratified — these inform implementation briefs)

### P1 — Languages are I/O integration surfaces, not compiler concerns

Both source and target languages are `LanguageModel` data passed in as parameters. The same `rust.dag` model is consumed when emitting Rust AND when parsing Rust as input (bidirectional grammar-as-data). There is no per-target emitter; `emit_rust` does not exist as compiler code. The Rust knowledge lives entirely in `extdeps/languages/rust.dag`.

This is a direct consequence of THESIS's "The derived homomorphism" claim plus [`docs/modeling-discipline.md`](modeling-discipline.md) Practice 10 Row 3, which classifies a hand-written cross-target emitter as "you re-wrote the compiler" — a whole-architecture failure, not a localized smell.

### P2 — Eval is translate-to-host

Evaluation is structurally the same operation as translation, with the host runtime as the target. The compiler-side machinery (parse, normalize, resolve, ground, then the coercion fold) is shared. Eval differs from translate **only** in the terminal action: execute the resulting structure via the host's interpretation algebra (eval) vs. serialize it via the target's grammar (translate).

Open question on `HostModel` shape — see Open Q1.

### P3 — Lenses are a side-channel over `InferredTree`

> **Ratified 2026-05-20** (operator-direct, resolving the codex-flagged conflict between the prior draft's lens stance and THESIS lines 105 + 342). The earlier "lenses entirely external to compile" framing was too strong; THESIS's "by construction" guarantee must hold. The reconciling insight: lens enforcement is **orthogonal to the homomorphism** — lenses observe the program; they don't participate in translate/eval. The right architectural question is the consumption *contract*, not the surface invocation.

Lens enforcement is **orthogonal to the emission/eval homomorphism.** The compiler's pipeline produces `InferredTree` as the **raw primitive layer where the program's full structure is visible.** Lenses are **folds over that tree** that produce dimension facts as side-channel output — they do NOT modify the tree, do NOT block translate/eval, and do NOT feed downstream stages of the homomorphism. THESIS's "by construction" guarantee is preserved by project-mandatory invocation of the lens pass, regardless of where the fold physically runs.

**Six commitments:**
1. **`InferredTree` is the lens consumption point** — a stable public contract. The lens framework binds to it.
2. **Lenses are folds over `InferredTree`** sharing the `fold_node` primitive (Practice 10 row 1).
3. **Lens outputs are side-channel** — `DimensionFact` / diagnostics. Translate/eval do NOT depend on lens output. Lens output does NOT feed downstream stages of the homomorphism.
4. **Built-in and user-defined lenses share one algebra contract.** The compiler core does NOT import or name any specific lens. `04_infer.dag`'s current `import v4.lens.cost { SymbolicCost }` violates this and needs to be fixed.
5. **Multi-lens execution is dependency-managed.** Multiple lenses over the same `InferredTree` share traversal — no re-walks. Lens-to-lens dependencies (e.g., complexity reading cost) are expressed in the substrate, not re-derived per call. See Open Q6 for the substrate primitive.
6. **"By construction" guarantee preserved at the contract layer.** Some project-mandatory wrapper (`validate_then_compile` or equivalent) runs lens validation BEFORE emit/eval. Bare `compile(text, input_lang, mode)` is the narrow advanced surface; the wrapper is the everyday entry point.

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

### P5 — Fold discipline does not imply purely local bottom-up computation

> **Ratified 2026-05-20** (reviewer-proposed, operator-direct).

Compiler stages are expressed as folds/traversals over Node, but their carriers may be **higher-order**, **effectful**, **constraint-bearing**, or **fixpoint-seeking**. Non-local mechanisms — scope resolution, type constraint solving, the coercion fold's structural-equality check, fixed-point analyses — must be **declared substrate machinery** (named primitives) and must produce **checkable witnesses** or **fail-closed diagnostics**. They must **never** appear as ad hoc compiler logic.

Practical implications:
- Resolve's algebra is not naively bottom-up. The carrier is roughly `Node → Env → Outcome<BoundNode>` — at each Node the fold returns a function that takes inherited scope context. This is an attribute-grammar-style inherited attribute, expressed through the carrier's higher-orderness.
- Ground (infer) may produce a constraint set per Node, then a separate `solve : ConstraintSet → Outcome<InferredTree>` step. The solver is substrate machinery (coercion fold) — not ad hoc compiler code.
- Translate's coercion-fold step is a fold, but its algebra performs structural-equality search against the target language model's declared inhabitants. The search is bounded (closed candidate set) and decidable by construction — not heuristic.

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

---

## The primitive set — the things the compiler IS

| Primitive | Purpose | Status |
|---|---|---|
| **`fold_node`** (catamorphism over Node) | Generic structural recursion over Node trees; anti-walker-dissolution substrate (Practice 10 row 1 of the modeling-discipline rubric). Every "pass" the compiler runs is `fold_node(tree, algebra)`. | **Landed** in `src/v4/std/node.dag` line 47, with the `NodeFold<R>` algebra carrier type. |
| **`traverse`** over `Outcome<T>` | Effect threading. When a fold's algebra produces `Outcome<T>` (success-or-diagnostic), `traverse` propagates failures and short-circuits without inline `match` ladders (Practice 10 row 2 — "traverse dissolution"). | **Uncertain.** Not explicitly visible in `std/`; may be implicit in `fold_node` with short-circuiting algebras. See Open Q4. |
| **Grammar-as-bidirectional-data** | One declarative grammar model in `extdeps/languages/X.dag` serves both parsing (text → Node) and serialization (Node → text). No separate parser and printer; the grammar production data IS the relation, used in both directions (Practice 8 — "No string-templating"). | **Partially landed** — `Grammar / ModeledGrammar / VoidGrammar` declared in `src/v4/compiler/02_parse.dag`; consumed by `extdeps/languages/dag.dag`. Inverse-grammar walk (the serialize direction) is scaffolded but not implemented. |
| **The coercion fold** | THE homomorphism-derivation primitive. Given two **canonical Node groundings** (a source Node's grounding into its algebra-instance, and the target language's declared inhabitants), a **mechanical zip-fold** (catamorphism) walks both in parallel and checks structure preservation. Decidable by construction over the closed declared candidate set; empty candidate ⇒ Diagnostic (fail-closed), never a fabricated coercion. | **Not yet implemented.** Substrate underlying it (canonical-form fold via `node.dag`'s B1-CANON contract, `content_hash = merkle_fold ∘ canonical`) exists. Ratified per `src/v4/TASKS.md` T-9 (line 418, D2-reversal 2026-05-17). Earlier project text (THESIS line 181, "structural algebra-homomorphism search") was superseded by this ratification; the doc-level reconcile is pending. |
| **Diagnostic + Locus carrier** | Fail-closed reporting (Practice 1). Every failure path goes through a structured `Diagnostic` with source `Locus`. | **Landed** in `src/v4/std/diagnostic.dag`. |
| **Typed dependency graph + topological/SCC machinery** | Per P6, dependencies are first-class typed edges (not incidental containment). The substrate declares `DependencyEdge`, `DependencyKind`, `DependencyGraph`, `AcyclicityWitness`, SCC condensation, topological-order derivation. Carriers (post-resolve onward) carry the graph alongside the containment tree. | **Not yet declared.** Probably lands in `src/v4/std/dependency.dag`. T-21's `affected_set.dag` is the incremental-rebuild specialization — built on the same substrate. |

That is the full compiler-primitive surface. Six things. Every stage, every pass, every translation, every eval, every glue derivation is structurally `fold_node + traverse + (one or more of the other four) + an algebra-as-data`.

**Why this terminology shift matters (coercion fold vs. "search"):** an earlier framing called the homomorphism step an "algebra-homomorphism search" or "inhabitance search," which suggests an open-ended search algorithm with potentially intractable complexity. The ratified position is the opposite: the candidate set is **closed and declared** (by the target language model), the source's grounding is **canonical and unique**, and the check is a **mechanical structural zip** — closer to type unification than to logic-programming search. If you find yourself reasoning about it as a search, that's a signal you're solving the wrong problem.

---

## The pipeline — stages as named algebras

```
                                  ┌─── pure middle (folds over Node) ───┐

  text ── parse(input_lang) ── normalize ── resolve ── ground ──┬── translate(target_lang) ── serialize(target_lang.grammar) ── target_text
                                                                 └── eval(host_model) ── execute(host)
```

Each stage is a fold/traverse over Node (potentially with a higher-order, effectful, or constraint-bearing carrier per P5). Facts flow forward (Practice 3, strong form): **each stage performs at most one disciplined fold/traverse over the current carrier and monotonically extends it. Facts are monotonic** — later stages extend or consume prior annotations rather than discarding and reconstructing semantic state. Resolve adds BindsTo edges; ground adds typeshape + inhabitance witnesses + the broader P6 dependency-graph edges. No stage re-derives facts from raw text or from a duplicated source-of-truth.

| Stage | Algebra produces | Where the algebra lives |
|---|---|---|
| `parse(input_lang)` | Node tree in input_lang's grammar | `input_lang.dag` (grammar productions) |
| `normalize` | canonical Node (surface sugar dissolved to substrate primitives) | `input_lang.dag` (sugar dissolutions) — possibly fused with parse, see Open Q3 |
| `resolve` | bound Node (every Atom → Decl reference) | scope-resolution algebra (likely substrate, may consult input_lang for binding rules) |
| `ground` | **`InferredTree`** — each Node carries algebra-inhabitance witness + typeshape | substrate's algebra-inhabitance machinery |
| `translate(target_lang)` | Target Node tree | **derived via the coercion fold** over (`InferredTree`, target_lang's declared inhabitants) |
| `serialize(grammar)` | target source text | target_lang.dag (grammar productions, inverse direction) |
| `eval(host_model)` | host Value | host_model's interpretation algebra |

**Carrier shape progression:**
```
Text → ParseTree → NormalizedTree → ResolvedTree → InferredTree
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

- `src/v4/std/node.dag` — `Node`, `Edge`, `fold_node`, `NodeFold<R>` algebra carrier. **Foundation in place.**
- `src/v4/std/algebra.dag` — algebra structures (Magma, Monoid, …) — the inhabitance authority.
- `src/v4/std/cardinality.dag` — inhabitance + bounded-natural refinement + descent evidence.
- `src/v4/std/diagnostic.dag` — `Diagnostic` + `Locus`, fail-closed reporting.
- `src/v4/std/refinement.dag` — base-type + fail-closed validation substrate (T-25-core, recently landed).
- `src/v4/std/verification.dag` — `TestClaim` substrate (eval's downstream consumer).
- `src/v4/extdeps/languages/dag.dag` + `rust.dag` + `python.dag` + `go.dag` + `cpp.dag` + `typescript.dag` — Wave-1 LanguageModels with Practice-8 fact-bundles.
- `src/v4/compiler/01_tokenize.dag`, `02_parse.dag`, `03_normalize.dag`, `03_resolve.dag` — pipeline scaffolds; T-8 implementation in flight.
- `src/v4/compiler/04_infer.dag`, `05_emit.dag`, `05_eval.dag`, `00_compile.dag` — scaffolds with correct interface shape, **bodies not implemented**.

Falsification probes (not load-bearing for the initial homomorphism):
- `src/v4/extdeps/languages/` — verilog, llvm_ir, machine_code, ptx, lean (probes the 5-behavior thesis across heterogeneous domains).
- `src/v4/extdeps/formats/spice.dag` — physics simulation probe (no control flow).

---

## What's NOT in scope for this design

- **Glue derivation / omni-stack** — orthogonal substrate (P4). Architecture must not preclude, but no implementation in the initial single-target compiler.
- **`extdeps/protocols/`** — missing substrate; deferred until glue is in scope.
- **Lens framework integration** — pending Open Q0 resolution.
- **Per-target emitters** — does not exist (P1). Each new target is one new `LanguageModel`.
- **Multi-module compilation** — single-module first; module decomposition substrate (`std/system.dag`) is a follow-on.

---

## Open questions — input wanted

### Ratified Q0 — Lens architecture (2026-05-20)

**Resolved.** Lenses are a side-channel over `InferredTree`, sharing the `fold_node` primitive, with multi-lens dependency-management and a project-mandatory wrapper preserving the THESIS "by construction" guarantee. Six commitments per P3 above; B-style vs C-style surface is a non-load-bearing implementation choice. The new substrate question this raises is **Open Q6 — multi-lens dependency management** (below).

### Ratified Q1 — `HostModel` is `Q1a + factored ModelCore` (2026-05-20)

**Resolved** (reviewer-recommended, operator-direct). A shared `ModelCore` substrate carries primitive types, algebra inhabitance, laws, effect semantics, partiality. `LanguageModel` and `HostModel` both extend it (see "Carrier shapes — `ModelCore` / `LanguageModel` / `HostModel`" above). Host-side concerns (allocation, execution model, runtime values, resource limits) are categorically different from emit-side concerns (grammar, serialization) and live only on `HostModel`; the `VoidGrammar` variant (Q1b) conflated them. Q1c ("eval = translate-to-machine-code + execute") hides too much (host execution involves real process state, runtime values, failure modes) and was rejected.

### Ratified Q2 — `InferredTree` dimensional facts (2026-05-20)

**Resolved by P3.** `InferredTree` carries **only grounding facts** (locus, binding, typeshape, inhabitance witness). Dimensional facts (cost, complexity, termination, ownership) are **lens outputs** produced as side-channel from folds over `InferredTree` (P3 commitment 3). The current `04_infer.dag` import of `v4.lens.cost.SymbolicCost` violates P3 commitment 4 and is a fix-needed.

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

### Open Q8 — Coercion-fold completeness vs fail-closed-incomplete

When the coercion fold cannot find a target inhabitant for a source structure, is the failure:
- **Q8a.** "No homomorphism exists" (complete: the search proved exhaustively that no inhabitant works), OR
- **Q8b.** "Search could not find one" (incomplete: the search bounded by reasonable depth, may have missed something).

Diagnostics must distinguish these. Q8b is acceptable IF the diagnostic surface tells the user "the compiler could not establish a homomorphism; this is not proof of impossibility — consider X / Y / Z workarounds." Q8a is stronger but may not always be achievable.

### Open Q9 — Independent witness checking

Is the coercion fold's output (a target Node + inhabitance witness) **independently checkable**? I.e., can a separate `verify_homomorphism(source, target, witness) -> Bool` function consume the witness and re-prove structure preservation without re-running the search?

Strongly preferred (per reviewer): **search is untrusted; checker is trusted.** This matches fail-closed philosophy — the compiler doesn't have to trust its own search; the witness is verifiable by inspection.

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

- A change to a ratified premise (P1 / P2 / P3 / P4 / P5 / P6) requires explicit operator ratification + reconcile of any downstream substrate.
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
