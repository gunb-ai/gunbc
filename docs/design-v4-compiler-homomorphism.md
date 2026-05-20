# v4 Compiler Architecture — Homomorphism-First Design

> **Status: DRAFT, under discussion.** This doc reconciles what the v4 compiler should be, ahead of T-9 (infer), T-10 (translate), T-22 (eval) implementation. It is intended as a forum for review and amendment — comment, push back, propose alternatives. Decisions ratified here flow into worker briefs and substrate PRs.
>
> Purpose: ensure the compiler architecture is consistent with [THESIS.md](../THESIS.md) — particularly "The derived homomorphism" — before implementation cements the wrong shape. v2's `emit_rust` is the canonical example of the shape we are NOT building.

## TL;DR — the compiler we want

The v4 compiler is **one structural primitive** (`fold_node`) plus **four irreducible side-effects at the boundaries** (read text, write text / execute / project) plus **five things that aren't `fold_node` but are substrate-shared** (the diagnostic carrier, grammar-as-bidirectional-data, `traverse` over the effect carrier, algebra-inhabitance search, and the diagnostic+locus carrier).

Everything else — every "pass," every "stage," every "language-specific" anything — is an **algebra** plugged into `fold_node`. The compiler knows nothing about Rust, Python, JavaScript, or even `.dag` beyond an irreducible bootstrap-seed grammar for `dag.dag` itself.

## End-to-end I/O — the compile function

```
compile(
  input_text: Text,
  input_lang: LanguageModel,           // the parse-side language model
  mode: CompileMode
) -> Outcome<Output>

where CompileMode =
  | TranslateTo(target_lang: LanguageModel)        // produce target source
  | Eval(host_interp: HostModel)                   // execute on host runtime
```

That is the full external surface of the compiler. Two parameters; one fail-closed `Outcome`.

**`LanguageModel`** is a `.dag` model in `extdeps/languages/X.dag` — a fact-bundle declaring X's grammar (bidirectional), primitive types, algebra inhabitance, and sugar dissolutions. Practice 8 governs its honesty.

**`HostModel`** is structurally the same shape as `LanguageModel` minus the grammar (no serialization needed — execution is the terminal action, not text emission). See "Open Q1" below — whether HostModel is a distinct substrate type or just a TargetModel variant is unresolved.

**`Output`** is mode-dependent: `TargetSource` (bytes/text) for TranslateTo, `Value` (host representation) for Eval.

**No Lens mode.** See "Lens stance" below — lenses are NOT a compiler concern.

## Architectural premises (ratified — these inform worker briefs)

### P1 — Languages are I/O integration surfaces, not compiler concerns

Both source and target languages are LanguageModels passed in as parameters. The same `rust.dag` model is consumed when emitting Rust AND when parsing Rust as input (bidirectional grammar-as-data). There is no per-target emitter; `emit_rust` does not exist as compiler code. The Rust knowledge lives entirely in `extdeps/languages/rust.dag`.

This is a direct consequence of the derived-homomorphism thesis ([THESIS](../THESIS.md) "The derived homomorphism") + Practice 10 Row 3 (hand-written emitter is a whole-architecture failure).

### P2 — Eval is translate-to-host

Eval is structurally the same operation as translate, with the host runtime as the target. The homomorphism-derivation step is shared. Eval differs from translate ONLY in the terminal action: execute via host's algebra vs. serialize via target's grammar.

Open question on HostModel shape — see Open Q1.

### P3 — Lens stance: NOT a compile mode

Lenses are user-facing enforcement over arbitrary sections of code. They are NOT consumed by the compiler core and the compiler does NOT know about any specific lens by name (`complexity`, `cost`, `ownership`, etc.).

The compiler produces `InferredTree` (the grounded program with algebra-inhabitance facts attached). Whatever runs lenses consumes `InferredTree` AS A DOWNSTREAM LAYER — it is not part of the compile pipeline. A lens framework MAY exist in `lens/` and emit diagnostics through the shared `std/diagnostic.dag` channel, but invocation is external to `compile()`.

**Concretely:**
- `compile()` has no `lenses: List<Lens>` parameter.
- The compiler does not import any `lens/*.dag` module.
- The smell to fix: `04_infer.dag` currently imports `v4.lens.cost { SymbolicCost }` — this entangles infer with a specific lens. The right shape is infer producing the grounding facts; cost-as-a-lens runs separately over the InferredTree.

This stance is **conservative** — it does not preclude a future where "this lens applies to all code" is a project policy. It DOES preclude that policy being implemented inside the compiler. Such a policy would be a build-system convention or a wrapper, not a compiler feature.

### P4 — Glue derivation is composed homomorphism, orthogonal to the compiler

For full-stack omni-emission (one `.dag` source → backend Rust + frontend JS + Postgres schema + the glue between), inter-module marshaling is the COMPOSED homomorphism of two single-module translates through a shared transport model. The "glue derivation" mechanism is `fold_node` + algebra-inhabitance-search composed twice through a transport carrier — same primitive, applied twice.

For this to work, two substrate slots must exist:
- `extdeps/protocols/` — transport semantics (REST, GraphQL, gRPC, ...). **Currently missing.**
- `std/system.dag` or equivalent — module decomposition substrate. **Currently informal; may already be implicit in module declarations but warrants explicit substrate.**

Out of scope for the initial single-target compiler; in scope for the architecture (it must not cement a shape that prevents glue derivation later).

## The primitive set — the things the compiler IS

| Primitive | Purpose | Status |
|---|---|---|
| `fold_node` (catamorphism over Node) | structural recursion — anti-walker-dissolution substrate (Practice 10 row 1) | **Landed** in `std/node.dag` line 47, with `NodeFold<R>` algebra carrier |
| `traverse` over the `Outcome<T>` effect carrier | effect threading — anti-traverse-dissolution substrate (Practice 10 row 2) | **Status uncertain** — not visible via quick grep; may be implicit in fold_node-with-short-circuit-algebra. Needs explicit substrate. |
| Grammar-as-bidirectional-data | one declarative model serves both parse and serialize (Practice 8) | **Partially landed** — `Grammar / ModeledGrammar / VoidGrammar` declared in `02_parse.dag`; `dag.dag` uses it. Inverse-grammar walk (emit-side) is scaffolded but not implemented |
| Algebra-inhabitance search | given (source algebra-instance, target's declared inhabitants), find the structure-preserving map. THE homomorphism mechanic. | **Not yet declared.** Inhabitance vocabulary exists (`cardinality.dag`, `nat.dag` law subjects), but the SEARCH primitive that translate/eval/coercion all consume is unstarted. This is the largest novel substrate move. |
| Diagnostic + Locus carrier | fail-closed reporting (Practice 1) | **Landed** in `std/diagnostic.dag` |

That is the full compiler-primitive surface. Five things. Every stage, every pass, every translation, every eval, every glue derivation is structurally `fold_node(tree, algebra)` + `traverse` for effect threading + one of the other three primitives for the work-specific operation.

## The pipeline — stages as named algebras

```
                                  ┌─── pure middle (folds over Node) ───┐

  text ── parse(input_lang) ─── normalize ─── resolve ─── ground ──┬── translate(target_lang) ── serialize(target_lang.grammar) ── target_text
                                                                    └── eval(host_model) ── execute(host)
```

Each stage is `fold_node(upstream_carrier, stage_algebra)`. Facts flow forward (Practice 3, strong form): each stage extends the SAME tree with its added facts. Nothing re-walks. The Node tree is monotonic — resolve adds bind-edges; ground adds inhabitance witnesses.

| Stage | Algebra produces | Owns the algebra |
|---|---|---|
| `parse(input_lang)` | Node tree in input_lang's grammar | input_lang.dag (grammar productions) |
| `normalize` | canonical Node (sugar dissolved to substrate) | input_lang.dag (sugar dissolutions) — possibly fused with parse, see Open Q3 |
| `resolve` | bound Node (Atoms → Decl edges) | scope-resolution algebra (likely substrate, may consult input_lang for binding rules) |
| `ground` | EnrichedSource (each Node carries algebra-inhabitance witness, typeshape) | the substrate's algebra-inhabitance machinery |
| `translate(target_lang)` | Target Node tree | derived via algebra-inhabitance search across (EnrichedSource, target_lang) |
| `serialize(grammar)` | target source text | target_lang.dag (grammar productions, inverse direction) |
| `eval(host_model)` | host Value | host_model's interpretation algebra |

**Carrier shape progression:**
```
Text → ParseTree → NormalizedTree → ResolvedTree → InferredTree (EnrichedSource)
                                                   ├─→ TargetNodeTree → TargetSource (TranslateTo mode)
                                                   └─→ Value (Eval mode)
```

## Carrier shapes — what each tree carries

### InferredTree (the central artifact)

The post-grounding tree. Each Node N carries:
- **Locus** — source position (from parse, propagated)
- **Binding** — for any Atom: resolved FunctionRef / Decl reference (from resolve, propagated)
- **Typeshape** — the inferred type, itself a substrate Node referencing an algebra-instance
- **Inhabitance witness** — which algebra-instance this Node inhabits + the structural proof (consumed by translate's homomorphism search and by eval's interpretation)

InferredTree does **NOT** carry:
- Cost, complexity, ownership, parallelism, termination — these are **lens facts** (P3 above) computed AGAINST InferredTree, not baked in
- Target-specific anything — translate is parametric on target_lang

See Open Q2 for the dimensional-facts question.

### LanguageModel

Declared in `extdeps/languages/X.dag`. Contains:
- Grammar (bidirectional productions — concrete syntax ↔ Node)
- Primitive types (each a Practice-8 fact-bundle)
- Sugar dissolutions (surface form → substrate Node)
- Algebra inhabitance declarations (which primitives inhabit which algebras)

### HostModel

Structurally a LanguageModel with no grammar (no serialization step). Declares:
- Algebra inhabitance for substrate primitives
- The host's representation for each substrate primitive (Transform → host function call; Branch → host if; Value → host literal)

See Open Q1.

### TargetSource / Value

Terminal output. TargetSource is target-grammar-serialized text + diagnostics. Value is the host's runtime representation + diagnostics.

## What we have today (relevant substrate)

- `std/node.dag` — Node + Edge + `fold_node` + `NodeFold<R>` algebra carrier. **Foundation in place.**
- `std/algebra.dag` — algebra structures (Magma, Monoid, …) — the inhabitance authority
- `std/cardinality.dag` — inhabitance + bounded-natural refinement + descent evidence
- `std/diagnostic.dag` — Diagnostic + Locus, fail-closed reporting
- `std/refinement.dag` — base-type + fail-closed validation substrate (T-25-core, just landed)
- `std/verification.dag` — TestClaim substrate (eval's downstream consumer)
- `extdeps/languages/dag.dag` + `rust.dag` + `python.dag` + `go.dag` + `cpp.dag` + `typescript.dag` — Wave-1 LanguageModels with Practice-8 fact-bundles
- `compiler/01_tokenize.dag`, `02_parse.dag`, `03_normalize.dag`, `03_resolve.dag` — pipeline scaffolds; T-8 implementation in flight
- `compiler/04_infer.dag`, `05_emit.dag`, `05_eval.dag`, `00_compile.dag` — scaffolds with correct interface shape, **bodies not implemented**

Falsification probes (not load-bearing for the initial homomorphism):
- `extdeps/languages/` verilog, llvm_ir, machine_code, ptx, lean — probe the 5-behavior thesis across heterogeneous domains
- `extdeps/formats/spice.dag` — physics simulation probe (no control flow)

## What's NOT in scope for this design

- **Glue derivation / omni-stack** — orthogonal substrate (P4). Architecture must not preclude, but no implementation in the initial single-target compiler.
- **`extdeps/protocols/`** — missing substrate; deferred until glue is in scope.
- **Lens framework integration** — P3. Lenses are downstream consumers, not compiler features.
- **Per-target emitters** — does not exist (P1).
- **Multi-module compilation** — single-module first; module decomposition substrate (`std/system.dag`) is a follow-on.

## Open questions — input wanted

### Open Q1 — HostModel: distinct type, or LanguageModel variant?

Eval needs a model of "what does the host runtime do for each substrate primitive." Options:

- **Q1a.** HostModel is a distinct substrate type in `std/host.dag` (or similar) — explicitly NO grammar field, declares only algebra-inhabitance + per-primitive host-operation mapping.
- **Q1b.** HostModel is a LanguageModel with `grammar = VoidGrammar` (the existing empty-grammar variant). Same machinery, just with grammar omitted.
- **Q1c.** "Host" is just `extdeps/languages/machine_code.dag` (or some other language model) — eval is then literally `translate-to-machine-code + execute`, with no new type at all.

Q1c is the most-economical (no new substrate); Q1a is the most-explicit. Q1b is in between. Recommendation: probably Q1a, because the host-side concerns (allocation, execution model, runtime values) are categorically different from emit-side concerns (grammar, serialization). But open to argument.

### Open Q2 — InferredTree dimensional facts: zero or N?

Should InferredTree carry built-in dimensional facts (cost, complexity, termination, ownership) computed during grounding, or should those facts be derived separately by lens-folds over a leaner InferredTree?

- **Q2a.** InferredTree carries ONLY structural grounding (typeshape, inhabitance, binding). All dimensional facts are computed as separate folds (potentially user-extensible). Aligns with P3 (lenses are not compiler-concerns).
- **Q2b.** InferredTree carries the "core" dimensional facts (termination, cost) needed for downstream guarantees (THESIS Tier 1 CX gate), and user-defined dimensions are computed separately.

The current `04_infer.dag` scaffold leans Q2b — it imports `v4.lens.cost { SymbolicCost }`. P3's "no specific named lens in the compiler" leans Q2a. Recommended Q2a, but this is a real fork worth deciding.

### Open Q3 — Stage fusion: separate parse + normalize, or one stage?

`parse` produces a Node tree in input_lang's grammar; `normalize` dissolves sugar to substrate primitives. The sugar IS declared in input_lang's grammar. Could parse produce canonical substrate Nodes directly?

- **Q3a.** Keep parse and normalize separate (current scaffold).
- **Q3b.** Fuse — parse consults the language model's sugar dissolutions and produces canonical Nodes directly.

Q3b saves a stage and an intermediate carrier; Q3a is more debuggable (you can inspect a pre-normalized parse tree). Probably Q3a for now; Q3b is a future optimization.

### Open Q4 — `traverse` primitive: explicit or implicit?

Practice 10 row 2 (traverse dissolution) requires an explicit `traverse`/`sequence` over the diagnostic-effect carrier. Currently we have `fold_node`. A `fold_node` whose algebra short-circuits on `Rejected` IS effectively a traverse. But the substrate primitive being implicit means every fold-author writes the short-circuit ladder, which is exactly the traverse-dissolution finding.

- **Q4a.** Declare an explicit `traverse_node` in `std/node.dag` that takes a fail-closed algebra. Compiler stages use this when threading diagnostics.
- **Q4b.** Document the convention that fold_node algebras must use `Outcome<T>` carriers and short-circuit, without a separate `traverse_node` primitive.

Recommendation: Q4a — explicit primitive matches the discipline doc. Otherwise every algebra-author re-derives the short-circuit pattern (traverse dissolution).

### Open Q5 — Resolve's substrate

Resolve is binding (Atom → Decl). Options:

- **Q5a.** Resolve is its own stage with its own algebra; binding rules consulted from input_lang.
- **Q5b.** Resolve is part of parse — parse consults language scope rules and produces bound Nodes directly. Fuses with parse like Q3b.

The current scaffold is Q5a. Probably keep — binding has scope-discipline beyond what parse can satisfy in one pass.

## What this doc does NOT decide

- Specific worker fan-out / who-implements-what (separate doc, follows ratification here).
- The shape of `extdeps/protocols/rest.dag` (out of scope per P4).
- Whether the bootstrap seed is Rust, native code, or another language (separate discipline — see `docs/design-pure-bootstrap-zero.md`).
- The detailed shape of the algebra-inhabitance search algorithm (substrate design follow-up).

## Process for amending this doc

This is a living architectural reference. Amendments:
- A change to a ratified premise (P1–P4) requires explicit operator ratification + reconcile of any downstream substrate.
- Resolution of an Open Q is recorded inline (move from "Open" section to a new "Ratified" subsection with date).
- Implementation that contradicts a premise is a STOP — escalate, do not improvise.
