# gunbc Roadmap

## Architectural Thesis

**Node and DAG are the only compiler primitives.**

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules, and emits target code. All
domain knowledge — types, cardinality, containers, optionality, and
target-language facts — lives in `.dag` definitions, not in the compiler
implementation.

A `Node` is the universal graph carrier. Names introduce opaque
namespaces over node-shaped compositions. Distinctions such as type
position, value/expression position, pattern position, and binding-site
descriptor are operational roles in the pipeline, not separate
ontological categories of node.

### Three Structural Principles

These principles refine the thesis based on root-cause analysis of the
current invariant violations (2026-03-23). Every active violation traces
back to one of these principles being underdeveloped.

**1. Names are opaque namespaces.**

Type names (`Int`, `Map`, `List`, etc.) are human-readable labels for
structural compositions, not compiler-meaningful identifiers. Names label
compositions built from machine primitives and algebraic constructors.
`Bit`/bitvectors live in the machine layer; `List`, `Map`, and `Set` live
in the algebraic layer with denotational laws. The compiler must not
branch on node names for structural decisions. At every level above the
fundamental unit, names are opaque.

Enforcement: inference receives nodes with opaque names and no name
registry. It can thread names through to output nodes and diagnostic
messages, but cannot branch on them. Emit receives the registry to
produce target-language identifiers. Scrambled-name tests (rename all
types to arbitrary strings, verify inference produces identical
structural decisions) verify the property wall.

**2. Compiler errors are orthogonal to the node graph.**

When inference fails, the result is not a node — it is a structurally
distinct failure. The compiler produces errors; it should never need to
rediscover them by string-checking node names.

Representation: `InferredNode = Resolved { node: Node } | CompilerError
{ message: String, span: SourceSpan }`. Inference returns `InferredNode`,
not `Node`. A child that fails propagates failure to the parent
expression. Emit never sees error nodes. `Dynamic` and `Error` unify
into `CompilerError` — both mean "inference couldn't determine this,"
and both are failures, not types. Generics now exist, but infer still
does not operate over unresolved type variables: slot substitution
happens before inference sees the graph, so type variables remain a
distinct structural concept rather than an inference-era name check.

**3. Syntactically distinct forms for the same operation normalize before
inference.**

The pipeline has a normalization boundary between resolve and infer.
After normalization: `Call`→`MethodCall` bridging is complete, nodes
carry their declared structural properties from `.dag` type definitions,
and parameterized types always carry their declared arity of children.
Infer receives a fully-normalized graph and processes one form per
semantic operation — no divergent code paths for the same concept.

### Dissolution Layers

| Layer | What dissolves | Compiler stops knowing | Status |
|-------|----------------|------------------------|--------|
| **L1: Types** | Name-checking, `node_is_*`, type constructors, `.connective` reads | What `List`, `Map`, `Int`, etc. mean | **Active** — 420 ratchet sites remaining |
| **L2: Expressions** | `ExprData` semantic knowledge, full ExprData walks | What `if`, `for`, `match`, `let`, etc. mean | **Bridge landed and dissolved** (P5.11 complete) |
| **L3: Syntax** | `kind_tag` string dispatch, hardcoded parser branches | How to parse surface syntax | **Active** — compositional parser (R3/Stream 0) |

L1 is the urgent layer. Its endgame is: the compiler processes graph
structure and reads structurally declared properties from `.dag` type
definitions. Names are opaque. Inference cannot read them.

### Algebraic Type Vision

```
// Level 0: Fundamental unit
type Bit = True | False                           // |[Bit]| = 2

// Level 1: Fixed-width compositions
type Byte = Tuple<Bit, Bit, Bit, Bit, Bit, Bit, Bit, Bit>  // |[Byte]| = 2^8

// Level 2: Named compositions (opaque namespaces)
// Int is Interpret<Signed, Word64> — a namespace, not a compiler-known concept

// Level 3: Algebraic structures
// Denotational (what they mean):
//   Set<A>   = A -> Bool         (finite support — membership)
//   Bag<A>   = A -> Nat          (finite support — multiplicity)
//   Map<K,V> = K -> (1 + V)      (finite support — keyed lookup)
//   List<A>  = Sigma n. Fin(n) -> A  (length + value per position)
```

Phase timeline:

| Phase | What's reachable | What's still a bridge |
|-------|-----------------|----------------------|
| Phase 1 | `InferredNode`, normalization, arity bridge, cardinality model, algebraic spec | Arity hardcoded; cardinality as binding annotation |
| Phase 2 | Gist end-to-end | Same bridge; no new emit heuristics needed |
| Phase 3 | Generics land. Algebraic specs become real `.dag` declarations. Arity bridge deleted. | None |
| Phase 4 | Shared emit reads `LanguageSpec` + structural declarations. Emit becomes name-opaque. | None |
| Phase 5 | L1=0. Scrambled-name tests pass. | None |
| Beyond | Bit-graph model. Primitives as compositions. Full structural type algebra. | None |

### Compositional Basis

**1. Compiler-model primitives (what the compiler operates on):**

- `Node` as the universal carrier
- Product composition (Conj) / Coproduct composition (Disj)
- Cardinality on bindings (Required, CardOptional)
- Generic slot composition (`<T>`)
- Recursion / self-reference (SCC-detected cycle metadata)
- Collection constructors (List, Set, Map — type-level, not name-level)

**2. Value/type-algebra primitives (what the kernel knows):**

Currently: `Int`, `String`, `Bool`, `Float`, `Unit` are kernel
primitives. The algebraic vision says they should be declared
compositions. This is an acknowledged intermediate state. The kernel
shrinks as the algebraic model matures (Phase 3+: real `.dag`
declarations; Phase 5+: `is_kernel_type` dissolves).

**3. Structural constructors:**

| Constructor | Builds | Compiler representation |
|-------------|--------|------------------------|
| Product | Records, tuples, structs | `connective: Conj`, named children |
| Coproduct | Enums, variants, sums | `connective: Disj`, named children |
| Cardinality | Presence/absence on bindings | `return_cardinality: CardOptional` |
| Parameterization | Generic types (`List<T>`) | Slot substitution before inference |
| Recursion | Self-referential types | SCC cycle metadata on Node |
| Collection | Indexed structures | `List<A>`, `Set<A>`, `Map<K,V>` type constructors |

**4. What is derived (named namespaces over the above):**

| Derived concept | Composition |
|----------------|-------------|
| Tuple | Unnamed product (Conj, positional children) |
| Record / struct | Named product (Conj, named children) |
| Enum | Named coproduct (Disj, variant children) |
| Optional binding | Cardinality 0..1 on the binding site |
| Type alias | Namespace over an existing composition |
| Function | `Callable { params: List<Param>, return: Node }` |
| Service | Record of operations + transport/runtime metadata |

**5. Container sharing — rendering fix, not IR change (2026-03-27)**

Fan-out (how many consumers a binding has) is the out-degree of a
binding's edges — already present in the graph structure. The Rust
container templates produce bare representations (`Vec<{0}>`) while
user types get shared (`Rc<T>`). Since `.dag` has value semantics and
the emitter inserts `.clone()` on every multi-use binding, bare
collections have O(n) clone cost — the root cause of every recurring
performance regression (FF-1, FF-5, FF-8, OOM incident).

**The fix:** Change Rust container templates to shared representations
(`Rc<Vec<{0}>>`, `Rc<HashMap<{0}, {1}>>`), update `05_emit_rust.dag`
and `runtime_rust.dag` for coherence. Atomic with stage0 regeneration.
Root-caused 2026-03-27; hand-patch proof: parser 37s to 0.4s.

### Composition Stack

```
Layer -1: Type Constructors                      *** NOT YET IN STD ***
  Product / Coproduct / Cardinality

Layer 0: Logic
  Classical = True | False                       (std/logic.dag)

Layer 1: Machine
  Bit, Vector<n, Bit>, Byte, Word32, Word64      (std/bit.dag)

Layer 2: Named compositions
  Int, Nat, Char, String                         (std/integer.dag, std/types.dag)

Layer 3: Collection algebras
  List<A>, Set<A>, Map<K,V>                      (std/types.dag)

Layer 4: Structural compositions                 *** MISSING FROM STD ***
  Span<I>, Tree<A>, DAG<Id,A>, Annotated<A,F>, LabeledTree<A>

Layer 5: Parser/source domain
  Token<Shape>

Layer 6: Compiler domain
  (domain-specific records using Layer 4 shapes)
```

See `docs/algebraic-type-spec.md` for the full collection algebra,
denotational model, law layer, support/algebra laws, and
occurrence/cardinality model.

---

## End Goal

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules defined in `.dag`, and emits
target artifacts. Adding a type, expression, language, transport, or
runtime contract should mean editing `.dag` files, not compiler code.

Concrete acceptance:

- Zero type-world knowledge in the compiler (L1 complete, **Phase 5 gate**):
  names are opaque namespaces; inference processes graph structure only;
  scrambled-name tests pass; no arity bridges remain
- Compiler errors are orthogonal to nodes: `InferredNode` wrapper;
  no error/Dynamic sentinels in the type graph
- Container types have real `.dag` algebraic declarations grounded in the
  Collection Denotational Model; optionality is cardinality on bindings
  (not a type constructor); arity and uniqueness properties fall out of
  the denotations, not compiler knowledge
- Emit is name-opaque: shared emit reads `LanguageSpec` + structural
  declarations for type→target-identifier mapping (Phase 4); no hardcoded
  `if type_name == "Map" { "HashMap" }` patterns
- One shared emit walker drives all target languages through a common
  compiler-owned spine
- Language-specific facts live in `dsl/extdeps/languages/*`; program-
  dependent lowering lives in compiler-owned adapters
- Ownership and complexity proofs are wired into the compile pipeline
- At least one real program (`gist`) compiles and runs end to end
- v1 is archived (fully removed via PR #200)
- Compiler-internal structure converges onto `Node` compositions

---

## Frontend/Backend Design Direction: Parse-Emit Symmetry

**Governing principle:** if we can emit a language, we should be able to
parse it. The same `LanguageSpec` drives both directions.

```
parse(spec, source) → graph      // frontend
emit(spec, graph)   → source     // backend
```

This is the natural extension of two established decisions:
- Languages are extdeps modeled from specs (`dsl/extdeps/languages/`)
- The compiler is a generic graph processor (no hardcoded language knowledge)

Currently these decisions are applied asymmetrically: emission reads from
`LanguageSpec` data, but parsing is hardcoded in Rust match arms. The
parser and emitter are parallel implementations of language knowledge —
one in code, one in data. Per invariants, they will diverge.

### What LanguageSpec carries for symmetry

| Fact | Emission | Parsing |
|------|----------|---------|
| Item tags | Which keyword to emit (`fn`, `def`) | Which keywords start items |
| Structural forms | How to render params, return, body | How to recognize them |
| Operators | Which symbol for BinAdd | Precedence, associativity |
| Block delimiters | `{`/`}` vs indentation | Block boundary detection |
| Type syntax | Template rendering (`Vec<{0}>`) | Template parsing |
| Binding syntax | `let x =` vs `x :=` | Binding recognition |

Every emission template has a parsing dual. `Vec<{0}>` renders
`Vec<Int>` — reversed, it parses `Vec<Int>` back to
`Generic("Vec", [Int])`.

### The .dag language is the first instance

The v1 of this design extracts the implicit .dag syntax spec from the
hardcoded parser:
1. Keyword table in `01_tokenize.dag` → data in a .dag spec
2. Operator precedence in `02_parse.dag` → data in a .dag spec
3. Item forms in `parse_item` → declarative structural spec
4. Statement forms in `parse_stmt` → declarative binding spec

The parser becomes a generic interpreter of syntax specs. The .dag spec
is instance #1. Future frontends (Python, Go source → graph) use the
same mechanism with different specs.

### Round-trip invariant

```
parse(spec, emit(spec, graph)) ≅ graph
```

### Irreducible differences

Some languages have structural parsing/emission differences that can't
be captured as template data (Python indentation, Rust lifetimes, Go
implicit interfaces). These stay as thin per-language modules.

### Relationship to R3 (compositional parser)

R3 is the implementation path for the parsing side. The compositional
parser recognizes structural forms driven by spec data. R3 exit criteria
(no keyword match arms, item identity as data) directly enable the
parse-emit symmetry.

---

## Current State (2026-03-28)

**Phases 1-4 complete. Phase 5 active.**

**Bootstrap status:** v1 retired (PR #200). v2 self-hosts. Bootstrap
fixed-point green. Stage0 compiles, 146 tests pass, 0 diagnostics on
self-compile. Self-compile time: ~6.47s (release mode). Clippy clean.

**L2 bridge dissolved** (P5.11 complete, 2026-03-26). ExprData children
now live in `node.children` as compositional Nodes. Bridge functions
`expr_children`/`map_expr_children`/`with_expr_data` deleted. P5.12
assessed ExprData tag — verdict: RETAIN as closed semantic tag.

**dsl/ compilation:** 9 → 3 parse diagnostics after parser fixes (this
session). Remaining 3 are pre-existing limitations: `and`/`&&` operator
ambiguity, block/record disambiguation in match arms, and generics on
pattern declarations.

**Container sharing (FF-8):** Root-caused 2026-03-27. Rendering change
in `LanguageSpec` container templates, not new compiler machinery.
Hand-patch proof confirms fix (parser 37s → 0.4s). Atomic fix pending.

**Root-cause audit (2026-03-23):** Three root causes behind all ~66
invariant violations — I (incomplete types ~32), II (error-as-name ~18),
III (divergent paths ~17). Most symptoms resolved through Phases 1-4.
Remaining violations are tracked in L1 dissolution (Stream 1).

**Stream 0 (compositional parser R3) is the architectural priority.**
**The practical priority is getting review.dag → compiled binary.**

---

## Critical Path: review.dag → Binary

The motivating use case: compile `review.dag` (a cyclical PR review
agent) into a native binary, replacing the current shell script runtime.
This requires clearing 4 stages. Stages 1-2 are compiler work; stages
3-4 are domain wiring.

### Stage 1: Parser (9 → 3 remaining)

Parser fixes for `uses Resource(mode:)`, `[after/when]`, `hermetic`,
`where` predicates, `is_text_readable`, paren variant patterns, node
declarations, and constrained assignments are written in both `.dag`
source and stage0 Rust. 6 of 9 errors fixed. Remaining 3 are DSL-side
workarounds (`and` → `&&`, block/record disambiguation, generics on
patterns).

**Blocker:** `regenerate-stage0.sh` has bugs (module renaming mismatch
with lib.rs, serde imports, duplicate types). Manual stage0 porting
works but the script needs fixing for sustainable regeneration.

### Stage 2: Rust Codegen (~280 errors)

| Gap | Errors | Root cause | Fix location |
|-----|-------:|------------|--------------|
| `Bool` → `bool` | ~153 | Primitive type mapping missing | `05_emit_rust.dag` |
| `Deserialize` trait | ~124 | Emitter generates serde derives without deps | `05_emit_rust.dag` |
| `expr_children` helpers | ~130 | P5.11 accessors emitted as calls but undefined | `05_emit.dag` |
| Module name mismatch | ~15 | `regenerate-stage0.sh` renames vs `lib.rs` | Script fix |
| `CodegenBackend` undeclared | ~10 | Type referenced but not emitted | `05_emit.dag` |
| `Secret` duplicate | 1 | Two modules emit same struct | `05_emit.dag` namespace |
| `Unit` unhandled | 1 | Not mapped to `()` | `05_emit_rust.dag` |

Priority: `Bool`→`bool` clears ~153 of ~280 errors. Serde removal
clears ~124. Together they resolve ~95% of codegen errors.

### Stage 3: Domain Workflow Compilation

| Gap | What's needed |
|-----|---------------|
| Cross-repo imports | `review.dag` imports from `gunbc/dsl/extdeps/`. Compiler needs multi-root `--source-dir` or review.dag moves into gunbc. |
| `for` comprehension codegen | `for pr in open_prs.pulls { ... }` with service calls inside loop body. Works for gist but untested with REST services. |
| REST transport codegen | `github.Pulls.List/Diff/CreateComment` → reqwest HTTP calls. Shell transport works; REST uses the gists.dag pattern. |
| CLI entrypoint generation | `main.rs` with clap subcommands for `review-cycle` and `review-pr`. Currently hand-maintained. |
| Auth injection | `github_token()` through GCP Secret Manager credential chain or env var fallback. |

### Stage 4: Feature Parity with Shell Runtime

| Gap | Shell has | .dag needs |
|-----|-----------|------------|
| Stateless dedup | Queries reviews API at commit SHA | `github.Pulls.ListReviews` service op |
| Line-level comments | Posts via `gh api` reviews endpoint | `github.Pulls.CreateReview` service op |
| Fix verification | Fetches prior violations, asks LLM if fixed | `verify_fixes` composing ListReviews + LLM |
| Cron upsert | Tag-based idempotent crontab | Modeled in `extdeps/cron.dag`, blocked on Stage 1 |
| Reference docs | Fetches algebra.dag, extdeps.md as review context | Pure function, easy to model once compilation works |

**Critical path: Stage 1 → Stage 2 → Stage 3.** Stage 1 is one
successful stage0 regeneration script fix. Stage 2 is codegen fixes
in `05_emit_rust.dag`. Stage 3 is domain wiring. Stage 4 is
incremental after the binary works.

---

## Active Work (Phase 5)

| Stream | Branch | Focus | Exit criteria |
|--------|--------|-------|---------------|
| **Stream 0: Compositional Parser (R3)** | `sharp-lynx-892` | Replace keyword-driven `parse_item`/`parse_stmt` with structure-driven model | `parse_item` has 0 keyword match arms; adding item type = 0 parser edits |
| **Stream 1: L1 Type Dissolution** | `l1-type-dissolution` | P5.7 predicates, P5.13 kernel decls, type constructors, type-name comparisons, CollectionKind bridge | L1 ratchet 420 → 0 |
| **Stream 2: Expression Model & Frontend** | *(unassigned)* | P5.1 token coherence, P5.5 residual enum cleanup, `assemble_stage0` fixups | Structural model maturity |
| **Stream 3: Container Sharing** | `perf/v2-tokenizer-root-cause` | Rust container templates → `Rc<Vec<{0}>>` etc. + emitter + runtime + stage0 regen | Eliminate O(n) clone class (FF-8) |

### Stream 0: Compositional Parser — Implementation Plan

**Goal:** The parser and emitter are symmetric. Both read from
`LanguageSpec`. Adding a language = adding a spec file, not code.

**Step 1: Extract .dag SyntaxSpec from hardcoded parser.**
Define a `SyntaxSpec` type in `dsl/extdeps/languages/dag/syntax.dag`
that captures the implicit grammar currently buried in `02_parse.dag`:
- Keyword → item-tag table (replaces tokenizer keyword map + `parse_item` match)
- Operator precedence table (replaces `infix_bp` / `prefix_bp` functions)
- Structural form declarations: which optional modifiers each tag accepts
  (type params, params, return, uses, provides, body style)
- Block/record disambiguation rule (`: ` after identifier = record field)
- Binding forms (let, bare assignment, node declaration, constrained assignment)

**Step 2: Make parser generic over SyntaxSpec.**
Refactor `02_parse.dag` so `parse_item` reads the tag table instead of
matching on `ShKw*` tokens. `parse_stmt` reads binding forms from spec.
Operator precedence comes from the spec table, not hardcoded functions.
The .dag `SyntaxSpec` is the first (and initially only) instance.

**Step 3: Validate symmetry with emission.**
Ensure `LanguageSpec` and `SyntaxSpec` share the same fact tables where
applicable (item tags, operator symbols, type templates). Define the
round-trip invariant test: `parse(spec, emit(spec, graph)) ≅ graph`.

**Step 4: DSL-side workarounds for current parser limitations.**
While the compositional parser is in progress, fix the remaining 3
parse diagnostics in the DSL files (not in the parser):
- `filesystem.dag`: `and` → `&&`
- `auth/patterns.dag`: rewrite `{ token: value }` match arms
- `std/patterns.dag`: defer generics on patterns until spec-driven parser

**Step 5: Second language frontend.**
With the spec-driven parser working for .dag, define a second
`SyntaxSpec` (e.g., a subset of Python or a simplified frontend) to
validate that the architecture actually supports multiple frontends.

---

## Execution Order

| Order | Phase | Status | Gate |
|-------|-------|--------|------|
| 1 | Phase 1: Soundness, root causes, L1 dissolution | **DONE** | InferredNode, normalization, arity, cardinality, algebraic spec |
| 2 | Phase 2: Gist end-to-end | **DONE** | Stage0 compiles gist; lib target builds |
| 3 | Phase 3: Compile contract, v1 retirement, generics | **DONE** | v1 retired; generics + recursive generics; arity bridge deleted |
| 4 | Phase 4: Shared emit, LanguageSpec, backend boundaries | **DONE** | Shared dispatch; LanguageSpec authority; DAG backend; generated tests |
| 5 | Phase 5: L1=0, convergence, L2/L3 preparation | **ACTIVE** | Scrambled-name tests pass; L1 ratchet = 0; compositional parser |

---

## Completed Phases

### Phase 1: Soundness, Root Causes, and L1 Dissolution (Done)

Fixed all regressions (R1-R4). Eliminated three root causes of invariant
violations. `InferredNode` wrapper complete (P1.9): `rt_node` returns
`NodeType = Typed | InferError | Untyped`; `node_is_error_type`/
`node_is_dynamic` deleted. Normalization stage wired (P1.14).
Arity bridge enforced (P1.17). Optional dissolved into cardinality
(P1.4): `Field.optional` removed, `optional_node()` deleted. Algebraic
type spec written (`docs/algebraic-type-spec.md`). Emit catch-all
fail-closed (P1.12). Testgen verification gate passing (P1.21).
Diagnostic ratchet at 0. All exit criteria MET.

### Phase 2: Gist End to End (Done)

11-file gist closure compiles with 0 diagnostics, 18 files emitted
(P2.1). Service operation bodies, `main.rs` dispatch, and multi-module
extdep imports all working. Lib target compiles and builds
(`v2_gist_full_pipeline`). `04_infer.dag` decomposition partially done:
`04_cycle`, `04_resolve`, `04_emit_info`, `04_sigs`, `04_access`,
`04_items`, `04_lookup`, `04_patterns`, `04_service` extracted.

### Phase 3: Compile Contract, Generics, v1 Retirement (Done)

v1 retired (PR #200 + #204). Generics landed including recursive
(P3.6/P3.8): `Pair<A,B>`, `Box<T>`, `MyList<T> = Nil | Cons` all work.
Arity bridge deleted (P3.7): compiler reads arity from `.dag`
declarations. Compile bundle has authoritative shape with ownership and
artifact planning (P3.2/P3.3). Runtime shim dissolved (P3.4).
Fixed-point convergence green (P3.1). All exit criteria MET.

### Phase 4: Shared Emit, Projections, Backend Boundaries (Done, 2026-03-26)

`LanguageSpec` is the single authority (P4.1). Shared emit owns full
callback-based `emit_shared_expr` recursion and `emit_shared_tco_expr`
walker (P4.2). `TestProjection` is a first-class output with shared
`extract_test_projections` (P4.3). DAG backend emits versioned
`dag-artifact.json` (P4.4). Typed backend plumbing and CLI surface
(P4.5). Callable-type parameters landed (P4.7: all 6 steps complete).
Equivalence validation passed (P4.6). All exit criteria MET.

---

## Phase 5: L1=0, Convergence, and L2 Preparation

**Gate:** L1 dissolution is complete. The compiler has zero type-world
knowledge. Names are opaque. Scrambled-name tests pass.

### Phase 5 Workboard

#### Track A: L3 dissolution (parser) — Stream 2

| ID | Item | Status |
|----|------|--------|
| P5.0 | `kind_tag` string dispatch elimination | **Done** |
| P5.1 | Token coherence (`Token { text, span, shape }` with `TokenShape`) | **Stream 2** |

#### Track B: Structural dissolutions — Stream 2

| ID | Item | Status |
|----|------|--------|
| P5.2 | Module/import dissolution | **Done** |
| P5.3 | Diagnostic / compile-output dissolution | **Done** |
| P5.4 | Service/support type dissolution | **Done** |
| P5.5 | Residual semantic enum cleanup | **Stream 2** |

#### Track C: L1 final deletions (the L1=0 gate) — Stream 1

| ID | Item | Status |
|----|------|--------|
| P5.6 | Scrambled-name tests (full suite) | **Done** — 6 tests comparing full typed-graph JSON |
| P5.7 | Delete `node_is_*` predicates | **Stream 1** — P5.7a (duplicate properties) done; P5.7b (CollectionKind) done but creates bridge debt |
| P5.8 | Delete `normalize_type_name` | **Done** |
| P5.9 | Delete `classify_type_structure` from emit | **Done** |
| P5.10 | Connective dissolution assessment | **Done** — verdict: keep Conj/Disj as permanent graph primitives |
| P5.13 | Kernel type `.dag` declarations | **Stream 1** — Part A done (algebra types in `dsl/std/algebra.dag`); Part B next (rewrite Int/Float/String as compositions, refactor inference to structural method lookup) |
| — | Type constructor reduction | **Stream 1** — 158 ratchet sites |
| — | Type-name comparison reduction | **Stream 1** — 40 ratchet sites |
| — | CollectionKind bridge evolution | **Stream 1** — dissolves when method algebras land |

#### Track D: L2 dissolution — Stream 2

| ID | Item | Status |
|----|------|--------|
| P5.11 | ExprData child dissolution | **Done** — children in `node.children`; ~300 lines of walker boilerplate eliminated |
| P5.12 | ExprData tag dissolution assessment | **Done** — verdict: RETAIN as closed semantic tag |

### P5.13 Design: Kernel Type Declarations

**Problem:** 27 sites branch on kernel type names (`Int`, `Bool`, etc.).
The compiler hardcodes 8 kernel types as a string list in `00_core.dag`.

**Part A (done):** Algebraic structure types declared in
`dsl/std/algebra.dag` (OrderedRing, Field, BooleanAlgebra, FreeMonoid,
PartialFunction). Kernel type declarations in `types.dag` (Bool, Unit,
Secret, Json, Bytes). Bootstrap loads algebra.dag, types.dag,
containers.dag.

**Part B (next):**
1. Rewrite `integer.dag`: `type Int = OrderedRing<Word64>` — **done** (B1)
2. Rewrite `float.dag`: `type Float = Field<Word64>` — **done** (B1)
3. Rewrite `string_type.dag`: `type String = FreeMonoid<Char>` — **done** (B1)
4. Refactor `04_method.dag`/`04_infer.dag`: resolve `+` to `add` field
   of type's algebraic composition (currently ~60 string branches)
5. Delete `kernel_types`, `is_kernel_type`, `is_int_type_node`, etc.

**Design principle:** The compiler knows only Node, Conj/Disj,
Cardinality, and Bit. Everything else is DAG composition processed
structurally. No labels, no name checks.

### Compositional Pipeline (C-series results)

All C-series items complete. Key outcomes:

| ID | What | Result |
|----|------|--------|
| C1 | Node vocabulary into `00_core.dag` | Done — `leaf_node`, predicates, structural helpers moved |
| C2 | Fix fact composition at definition sites | Done — `.inferred` is structural; emit reads directly, no re-resolution |
| C3 | Scope elimination | Done — no emit file reads `scope.type_env` |
| C4 | Single-pass transitive computations | Done — service expansion halts on registry stability |
| C5 | Timing instrumentation | Done — per-phase timing in stage0 `compile.rs` |

### Fan-Out Preservation

| ID | Item | Status |
|----|------|--------|
| FO-1 | Fold accumulator fan-out fix | **Done** — 3.2x speedup |
| FO-2 | Kahn cycle detection O(V+E) | **Done** |
| FO-3 | v1 emitter rendering audit | Open |
| FO-4 | v2 emitter fan-out fact | Open |
| FO-5 | Fan-out preservation ratchet | Blocked on FO-3 |

### Phase 5 Milestones (2026-03-27)

- **Diagnostic ratchet: 0.** Zero type errors or warnings on self-compile.
- **Fixed-point: PASSES.** Stage0 → stage1 → stage2 converges.
- **Self-compile: 6.47s** (release). Tokenize 4.87s, Parse 78ms,
  Resolve 1ms, Reconcile 244ms, Emit 1.27s.

### Phase 5 Exit Criteria

- **L1=0:** scrambled-name tests pass (done); no `node_is_*` predicates
  (done); no `normalize_type_name` (done); no `classify_type_structure`
  (done); no arity bridges (done)
- **Remaining L1 (420 total):** 158 type constructors, 148 predicate
  calls, 47 Conj/Disj refs, 40 type-name comparisons, 27 connective
  accesses. Dissolves through P5.13 kernel declarations and method
  algebra queries.
- `CollectionKind` — bridge debt (dissolves when method algebras land)
- Each convergence step survives re-bootstrap and fixed-point verification

---

## Cross-Cutting Reference

### Compositional Refactor Targets (`R*`)

| ID | Module | Current → Target | Status |
|----|--------|-------------------|--------|
| R1 | `00_core.dag` | C → A | Phase 1/3 |
| R2 | `01_tokenize.dag` | A → A | Done |
| R3 | `02_parse.dag` | D → B+ | **ACTIVE (priority)** — compositional parser model |
| R4 | `03_resolve.dag` | A → A | Done |
| R5 | `04_infer.dag` | D → B+ | Phase 1 |
| R6 | `05_emit*.dag` | D → B+ | Phase 4 (done) |
| R7 | `complexity.dag` | B → A | Phase 4 |
| R8 | `ownership.dag` | A- → A | Phase 3 |
| R9 | `compile.dag` | B- → A | Phase 3 |

### R3 Detail: Compositional Parser Model

**Problem (2026-03-28).** The parser has accumulated structural debt.
`parse_item` has 9 keyword match arms. Each new DSL syntax form adds
dedicated parse functions and match arms. The keyword enum has 9 item
types across 3 files.

**Target model:**

```
Item     = Tag Name [TypeParams] [Params] [Return] [Uses] [Provides] Body
Body     = BlockBody | FieldBody | AssignBody | DeclOnly
Statement = LetBinding | NodeBinding | Return | Expr
Binding  = Name [Constraints] (: | =) Expr [Return]
```

Item identity is the tag string, not an enum variant. Adding a new item
type requires zero parser changes. Block vs record resolved structurally:
`{ ident : expr }` is always record; `{ stmt; stmt }` is always block.

**Exit criteria:**
- `parse_item` has no keyword-specific match arms
- Adding a new item type requires editing 0 parser files
- Block/record disambiguation is structural, not heuristic
- Type params, constraints, return annotations available to any form

---

## Backlog

### Language Features

| Item | Why deferred |
|------|--------------|
| Full linear type checking | Ownership proof started; full proof beyond current migration |
| `[when]` string comparison | Only boolean fields supported; blocks conditional service dispatch |
| `[when]`/`[after]` inside `for` | Bracket clauses only on top-level step bindings |
| Multiple `uses` clauses | Only one per `func`; workaround: use `shell.Exec.Run` |
| `fixture`/`test` blocks | Blocked on compositional parser — would grow keyword enum |

### Desired Parser Features (2026-03-28)

| Feature | DSL workaround | Proper fix |
|---------|----------------|-----------|
| `uses Resource(mode: X)` parameterized resources | Drop `uses` clause | Compositional parser: `uses` accepts arbitrary config |
| `[after X, when X.field]` multi-clause brackets | Implicit data-flow ordering | Bracket clause accepts comma-separated constraints |
| `[when]` string comparison `x == "foo"` | `match` + `shell.Exec.Run` | Bracket clause accepts arbitrary boolean expressions |
| Multiple `uses` clauses per func | `shell.Exec.Run` for secondary resources | `uses net: Network, fs: Filesystem` |
| `fixture`/`test` blocks | Comment out; tests via cargo test | Structural item tags |
| `and`/`or` as operators | `&&`/`||` | Parser recognizes as boolean operators |
| `{ ident: value }` in match arms | Named types or `let` bindings | Structural block/record disambiguation |
| `pattern<T>` generics on patterns | Monomorphize manually | Type params on any structural form |
| `where` predicates in params | Doc comment or runtime validation | `where` as structural modifier |

### Compiler Improvements

| Item | Why deferred |
|------|--------------|
| Anonymous record target resolution | R2 stopgap; real fix is proper field access for any arity |
| Collection intrinsic semantics in shared IR | After shared emit; Collection Denotational Model pins methods |
| TCO backend contract | Clean up during/after shared emit extraction |
| `assemble_stage0` fixups (5 known issues) | Not blocking; manual corrections per regeneration |
| Statement/expression emit classification | Python/Go statement-oriented; emit assumes expression-orientation |
| Cross-language test generation parity | Go and Python test emission needs gaps filled |

### Rust Codegen Issues (stage0 regeneration blockers)

Self-compile produces 0 diagnostics, but generated Rust fails `cargo
check` with ~280 errors in three layers:

| Layer | Errors | Root cause |
|-------|--------|-----------|
| Primitive type mapping | ~154 | `Bool` → `Bool` not `bool`; `Unit` → `()` |
| Module/crate wiring | ~151 | Missing `use serde::Deserialize`, lib.rs module declarations, duplicate Secret |
| Missing expr_children helpers | ~130 | P5.11 accessors (`let_value`, `if_condition`, etc.) not landing in generated Rust |

---

## Verification

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` | After every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | After every change |
| V2 compiler tests | `cargo test -p v2-compiler-tests` | After every change |
| Diagnostics ratchet | `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` | After `.dag` changes |
| Fixed point | `cargo test -p v2-compiler-tests v2_bootstrap_fixed_point -- --ignored` | After `.dag` changes affecting bootstrap |
| Gist pipeline | `cargo test -p v2-compiler-tests v2_gist_full_pipeline -- --ignored` | After emit changes |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | After `.dag` changes (goal: 0) |
| Scrambled-name tests | `cargo test -p v2-compiler-tests v2_scrambled_name_inference` | After inference changes |

### Test Generation Exit Criteria

Generated tests are first-class compiler outputs. They must:

1. **Exist for all three targets.** Every service operation with mock data
   produces a test file for Rust, Go, and Python.
2. **Compile/parse in the target language.** Rust: `rustc --edition 2021`.
   Go: `go vet`. Python: `ast.parse(...)`.
3. **Exercise the operation.** Instantiate mock data, call the service
   operation, assert the return value.
4. **Parity across targets.** Same operations tested, same mock data,
   same assertion shape.
