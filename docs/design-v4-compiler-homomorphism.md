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
2. **Plus four substrate operations** that aren't `fold_node` but are needed:
   - A diagnostic carrier (`Diagnostic + Locus`, fail-closed reporting).
   - **Grammar-as-bidirectional-data** — one declarative grammar model that drives both parsing (text → Node) and serialization (Node → text). No separate parser and printer.
   - **`traverse`** over the diagnostic-effect carrier (Node fold that short-circuits on failures without re-derived match ladders).
   - **The coercion fold** — a mechanical zip-fold over two canonical-Node groundings that checks structure preservation. Per `src/v4/TASKS.md` T-9 (ratified 2026-05-17), this is **the** homomorphism-derivation primitive — decidable by construction, never a search.
3. **Plus three irreducible boundary actions** — read input text, write output text (or execute, or project a value), report diagnostics.

Everything else — every "pass," every "stage," every "language-specific" anything — is an **algebra** plugged into `fold_node`. The compiler knows nothing about any specific language (Rust, Python, even `.dag` itself, beyond an irreducible bootstrap-seed needed to read `dag.dag` once).

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

That is the full external surface of the compiler. Two parameters; one fail-closed `Outcome`.

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

---

## The primitive set — the things the compiler IS

| Primitive | Purpose | Status |
|---|---|---|
| **`fold_node`** (catamorphism over Node) | Generic structural recursion over Node trees; anti-walker-dissolution substrate (Practice 10 row 1 of the modeling-discipline rubric). Every "pass" the compiler runs is `fold_node(tree, algebra)`. | **Landed** in `src/v4/std/node.dag` line 47, with the `NodeFold<R>` algebra carrier type. |
| **`traverse`** over `Outcome<T>` | Effect threading. When a fold's algebra produces `Outcome<T>` (success-or-diagnostic), `traverse` propagates failures and short-circuits without inline `match` ladders (Practice 10 row 2 — "traverse dissolution"). | **Uncertain.** Not explicitly visible in `std/`; may be implicit in `fold_node` with short-circuiting algebras. See Open Q4. |
| **Grammar-as-bidirectional-data** | One declarative grammar model in `extdeps/languages/X.dag` serves both parsing (text → Node) and serialization (Node → text). No separate parser and printer; the grammar production data IS the relation, used in both directions (Practice 8 — "No string-templating"). | **Partially landed** — `Grammar / ModeledGrammar / VoidGrammar` declared in `src/v4/compiler/02_parse.dag`; consumed by `extdeps/languages/dag.dag`. Inverse-grammar walk (the serialize direction) is scaffolded but not implemented. |
| **The coercion fold** | THE homomorphism-derivation primitive. Given two **canonical Node groundings** (a source Node's grounding into its algebra-instance, and the target language's declared inhabitants), a **mechanical zip-fold** (catamorphism) walks both in parallel and checks structure preservation. Decidable by construction over the closed declared candidate set; empty candidate ⇒ Diagnostic (fail-closed), never a fabricated coercion. | **Not yet implemented.** Substrate underlying it (canonical-form fold via `node.dag`'s B1-CANON contract, `content_hash = merkle_fold ∘ canonical`) exists. Ratified per `src/v4/TASKS.md` T-9 (line 418, D2-reversal 2026-05-17). Earlier project text (THESIS line 181, "structural algebra-homomorphism search") was superseded by this ratification; the doc-level reconcile is pending. |
| **Diagnostic + Locus carrier** | Fail-closed reporting (Practice 1). Every failure path goes through a structured `Diagnostic` with source `Locus`. | **Landed** in `src/v4/std/diagnostic.dag`. |

That is the full compiler-primitive surface. Five things. Every stage, every pass, every translation, every eval, every glue derivation is structurally `fold_node + traverse + (one of the other three) + an algebra-as-data`.

**Why this terminology shift matters (coercion fold vs. "search"):** an earlier framing called the homomorphism step an "algebra-homomorphism search" or "inhabitance search," which suggests an open-ended search algorithm with potentially intractable complexity. The ratified position is the opposite: the candidate set is **closed and declared** (by the target language model), the source's grounding is **canonical and unique**, and the check is a **mechanical structural zip** — closer to type unification than to logic-programming search. If you find yourself reasoning about it as a search, that's a signal you're solving the wrong problem.

---

## The pipeline — stages as named algebras

```
                                  ┌─── pure middle (folds over Node) ───┐

  text ── parse(input_lang) ── normalize ── resolve ── ground ──┬── translate(target_lang) ── serialize(target_lang.grammar) ── target_text
                                                                 └── eval(host_model) ── execute(host)
```

Each stage is `fold_node(upstream_carrier, stage_algebra)`. Facts flow forward (Practice 3, strong form): each stage extends the **same** tree with its added facts. Nothing re-walks. The tree is monotonic — resolve adds bind-edges (Atom-to-declaration references); ground adds inhabitance witnesses (proof that each Node lives in some algebra-instance).

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

The substrate-task IDs that implement each stage:
- **T-8** (in-flight) — `normalize` + `resolve`.
- **T-9** — `ground` (currently scaffolded in `04_infer.dag`).
- **T-10** — `translate` + `serialize` (currently scaffolded in `05_emit.dag` + `00_compile.dag`).
- **T-22** — `eval` (currently scaffolded in `05_eval.dag`).

See `src/v4/TASKS.md` for the full task dependency graph.

---

## Carrier shapes — what each tree carries

### `InferredTree` (the central artifact)

The post-grounding tree. Each Node `N` carries:
- **Locus** — source position (from parse, propagated forward).
- **Binding** — for any Atom: resolved FunctionRef / Decl reference (from resolve, propagated forward).
- **Typeshape** — the inferred type, itself a substrate Node referencing an algebra-instance.
- **Inhabitance witness** — which algebra-instance this Node inhabits + the structural proof. Consumed downstream by `translate`'s coercion fold and `eval`'s interpretation.

`InferredTree` does **NOT** carry:
- Cost, complexity, ownership, parallelism, termination — these are **lens facts** (see "Open Q0" + "Open Q2" below) and their location is part of the unresolved lens-architecture question.
- Target-specific anything — `translate` is parametric on `target_lang`.

### `LanguageModel`

Declared in `extdeps/languages/X.dag`. Contains:
- **Grammar** (bidirectional productions — concrete syntax ↔ Node).
- **Primitive types** (each a Practice-8 fact-bundle — width, signedness, range, encoding, …).
- **Sugar dissolutions** (surface form → substrate Node).
- **Algebra inhabitance declarations** (which primitives inhabit which algebras, e.g., Rust's `i32` inhabits the signed-integer algebra with width=32).

### `HostModel`

Structurally a `LanguageModel` with no grammar (no serialization step). Declares:
- Algebra inhabitance for substrate primitives (same shape as `LanguageModel`).
- The host's runtime representation for each substrate primitive (Transform → host function call; Branch → host if-expression; Value → host literal allocation).

See Open Q1 for whether it's a distinct type or a `LanguageModel` variant.

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

### Open Q1 — `HostModel`: distinct type, or `LanguageModel` variant?

Eval needs a model of "what does the host runtime do for each substrate primitive." Options:

- **Q1a.** `HostModel` is a distinct substrate type in `std/host.dag` (or similar) — explicitly no grammar field, declares only algebra-inhabitance + per-primitive host-operation mapping.
- **Q1b.** `HostModel` is a `LanguageModel` with `grammar = VoidGrammar` (the existing empty-grammar variant). Same machinery, just with grammar omitted.
- **Q1c.** "Host" is just `extdeps/languages/machine_code.dag` (or some other LanguageModel) — eval is then literally `translate-to-machine-code + execute`, with no new type at all.

Q1c is the most-economical (no new substrate); Q1a is the most-explicit. Q1b is in between. Recommendation: probably Q1a — host concerns (allocation, execution model, runtime values) are categorically different from emit concerns (grammar, serialization). But open to argument.

### Ratified Q2 — `InferredTree` dimensional facts (2026-05-20)

**Resolved by P3.** `InferredTree` carries **only grounding facts** (locus, binding, typeshape, inhabitance witness). Dimensional facts (cost, complexity, termination, ownership) are **lens outputs** produced as side-channel from folds over `InferredTree` (P3 commitment 3). The current `04_infer.dag` import of `v4.lens.cost.SymbolicCost` violates P3 commitment 4 and is a fix-needed.

### Open Q3 — Stage fusion: separate `parse` + `normalize`, or one stage?

`parse` produces a Node tree in input_lang's grammar; `normalize` dissolves sugar to substrate primitives. The sugar IS declared in input_lang's grammar. Could `parse` produce canonical substrate Nodes directly?

- **Q3a.** Keep `parse` and `normalize` separate (current scaffold).
- **Q3b.** Fuse — `parse` consults the language model's sugar dissolutions and produces canonical Nodes directly.

Q3b saves a stage and an intermediate carrier; Q3a is more debuggable (you can inspect a pre-normalized parse tree). Probably Q3a for now; Q3b is a future optimization.

### Open Q4 — `traverse` primitive: explicit or implicit?

Practice 10 row 2 requires an explicit `traverse` over the diagnostic-effect carrier. Currently the substrate has `fold_node`; a `fold_node` whose algebra short-circuits on `Rejected` IS effectively a traverse. But if the substrate primitive is implicit, every fold-author writes the short-circuit ladder — which is exactly the traverse-dissolution finding that Practice 10 forbids.

- **Q4a.** Declare an explicit `traverse_node` in `std/node.dag` that takes a fail-closed algebra.
- **Q4b.** Document the convention that fold algebras must short-circuit, without a separate primitive.

Recommendation: Q4a — explicit primitive matches the discipline doc.

### Open Q5 — Resolve's substrate placement

`resolve` is binding (Atom → Decl). Options:

- **Q5a.** Resolve is its own stage with its own algebra; binding rules consulted from input_lang.
- **Q5b.** Resolve is part of parse (consults language scope rules; produces bound Nodes directly).

Current scaffold is Q5a. Probably keep — binding has scope-discipline beyond what parse can satisfy in one pass.

### Open Q6 — Multi-lens dependency management substrate primitive

P3 commitment 5 requires that N lenses consuming the same `InferredTree` share traversal — no re-walks per lens. This is a substrate-design question, of the same flavor as the coercion fold's substrate primitive.

Three sub-questions:

- **Q6a.** Is there a substrate primitive `multi_lens_fold(tree, List<Lens>) -> Map<Lens, DimensionFact>` that coalesces shared traversals — single pass over the tree, each Node visited once, each lens's algebra applied at the right step?
- **Q6b.** How are lens-to-lens dependencies expressed? Example: the complexity lens reads the cost lens's output. Does the multi-lens fold compute a topological order over lens dependencies and stage the folds (cost first, complexity second)? Or do lenses produce intermediate values consumable by other lenses in the SAME pass (more complex but single-traversal)?
- **Q6c.** Are lens outputs themselves Node-graph data, so they are consumable by other lenses uniformly (and by downstream tooling like IDEs, build reporters)?

This is substrate-design work. Probably lands in a new `std/lens.dag` or extension of `std/node.dag`. Required before multi-lens execution is correct under Practice 3 (facts flow forward, no re-derivation).

---

## What this doc does NOT decide

- Specific implementation fan-out / who-implements-what (separate doc, follows ratification here).
- The shape of `extdeps/protocols/rest.dag` (out of scope per P4).
- Whether the bootstrap seed is Rust, native code, or another language (separate discipline — see `docs/design-pure-bootstrap-zero.md`).
- The detailed shape of the coercion fold's algorithm (substrate design follow-up; this doc only commits the **shape** — zip-fold over canonical groundings, decidable-by-construction, fail-closed on empty candidate set).

---

## Process for amending this doc

- A change to a ratified premise (P1 / P2 / P4) requires explicit operator ratification + reconcile of any downstream substrate.
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
