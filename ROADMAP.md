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
| **L1: Types** | Name-checking, `node_is_*`, type constructors, `.connective` reads | What `List`, `Map`, `Int`, etc. mean | **Active** — CollectionKind dissolved (2026-03-28); kernel type dissolution remaining |
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

### CollectionKind dissolution (2026-03-28) — COMPLETE

**What was dissolved:** The `CollectionKind` enum (6 variants: `ListKind`,
`SetKind`, `NonEmptyListKind`, `NonEmptySetKind`, `MapKind`, `NoCollection`)
and the `collection_kind` field on `Node`. 184 sites across 17 .dag files
and 8 stage0 .rs files. Net: -501 lines, +306 lines.

**Design:** No new field on Node (a label/string would violate the
"compiler is a DAG processor" invariant). Instead:

1. **Resolve time** (names allowed): `container_types` data list in
   `00_core.dag` controls which parameterized types stay unexpanded.
2. **After resolution** (structural): containers are uniquely identifiable
   as nodes with `children > 0 && connective == NoConnective`. Products
   have Conj, coproducts have Disj, callables use the params field,
   tuples have Conj.
3. **Emit time** (names allowed): `to_snake(n.name)` as LanguageSpec
   template key. Each language template map has explicit entries for
   all container variants.

**Structural predicates** (pass scrambled-name test):
- `node_is_collection(n)` — children > 0, no connective
- `node_is_keyed_collection(n)` — above AND 2 children (Map)
- `node_is_element_collection(n)` — above AND 1 child (List/Set/NonEmpty*)

**Stepping stone:** `container_types` data list is name-based at resolve
time. Dissolves further when algebraic inhabitation (FreeMonoid,
BooleanAlgebra, PartialFunction from `std/algebra.dag`) replaces it.

**Remaining L1 cleanup:**
- `generated_tests.rs` still embeds old .dag source with `CollectionKind` —
  regenerated on next self-compile pass
- Kernel type dissolution (`is_kernel_type`, `is_int_type_node`, etc.) —
  requires algebraic ontology from `std/algebra.dag`
- `container_types` data list → derives from algebraic inhabitation (future)

### Compositional Basis

**1. Compiler-model primitives (what the compiler operates on):**

- `Node` as the universal carrier
- Product composition (Conj) / Coproduct composition (Disj)
- Cardinality on bindings (Required, CardOptional)
- Generic slot composition (`<T>`)
- Recursion / self-reference (SCC-detected cycle metadata)
- Collection identity — structural (no enum, no field on Node; `container_types` data list + shape after resolution)

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
| Collection | Indexed structures | Structural shape after resolution (children + no connective); `container_types` data list |

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

## Design Direction: Decidability (DAG-Reducibility)

**Governing principle:** every `.dag` program is decidable. The DAG is
the only computational primitive. Recursion, loops, and cyclic-looking
patterns are surface syntax sugar that must decompose into bounded
iteration over finite structure.

This is the complexity-side dual of parse-emit symmetry: just as the
parser and emitter are symmetric views of the same spec, every
recursive surface form has a bounded iterative lowering that the
compiler can prove terminates.

### What this means for the language

Undecidable programs are **structurally unrepresentable**, not detected
and rejected. This is the "construction over ratchets" principle
applied to the language itself — the same way `ExpectedToken` makes
string-based parser dispatch impossible by construction, the language's
iteration primitives make unbounded computation impossible by
construction.

The language provides:
- `fold`, `map`, `filter`, `flat_map` — bounded by collection size
- `repeat(bound: N)` — bounded by explicit count
- Recursive functions — bounded by structural descent on children

The language does not provide:
- `while(true)` — no unbounded loop primitive
- `loop` without a bound — no general loop
- Unrestricted recursion — must descend on a well-founded measure

The complexity analyzer's role is not to *filter out* bad programs.
It is to *confirm* what the language already guarantees. If `?O(?)`
appears, the bug is in the analyzer or a missing lowering in the
compiler — not in the user's program.

| Surface pattern | Why it's bounded | Lowering |
|---|---|---|
| Tree walk (visit children) | strict child descent | `fold` over `node.children` |
| Tokenizer/scanner loop | monotonic position advance | iterate until `pos >= len` |
| Accumulator recursion | decreasing counter or list length | `fold` with init + step |
| Mutual recursion (SCC) | shared decreasing measure | topological fold |
| Long-running process | explicit bound `repeat(bound: N)` | bounded loop, N can be 2^64 |

### Relationship to existing work

- **S85 (recursive types):** SCC-derived cycle metadata already
  classifies recursive type structures. This extends the same principle
  to function bodies — the compiler proves the recursion is bounded.
- **Complexity engine:** currently interim polynomial algebra. Under
  this direction, the engine is a proof witness, not a filter. Its job
  is confirming structural guarantees, not catching violations.
- **`RecursionPattern` (complexity.dag:201-204):** `LinearRecursion |
  DivideAndConquer | UnresolvableRecursion` is declared but never wired
  in. This is the classification mechanism — `UnresolvableRecursion`
  means the analyzer can't see the descent, which is a compiler bug.
- **Compositional parser (Stream 0):** the parser itself must be
  decidable. Spec-driven structural recognition over finite token
  streams, not unbounded backtracking.

### Concrete next steps

1. Audit language iteration primitives — ensure no primitive permits
   unbounded computation. If one exists, redesign it with an explicit
   bound parameter.
2. Wire `RecursionPattern` into `cost_of_expr` — classify every
   recursive call as `LinearRecursion` or `DivideAndConquer`. Any
   `UnresolvableRecursion` is a compiler gap to fix.
3. Ensure `cost_of_expr` (tree walk, bound = |nodes|) and
   `tokenize_loop` (scanner, bound = |source|) express their descent
   measures so the analyzer resolves them.
4. Fix `trace_pop_frame` O(|stack|^2) — `take(count - 1)` copies the
   list; needs O(1) pop.

### Enforcement (see Guarantee Map)

This is a **Tier 1 (structural)** guarantee, not Tier 2 (tested).
The language makes undecidability unrepresentable. The complexity
ratchet test exists to verify the analyzer sees the bounds correctly
— it's a test of the *analyzer*, not a gate on *programs*.

### Non-goal

This does not restrict what users can express — only how they express
it. Recursive types, recursive functions, and long-running processes
are all supported. The language requires the bound to be explicit or
structurally derivable. "Run for 30 trillion years" is fine.
"Run forever" is not expressible.

---

## Design Direction: Guarantee Map

**Problem:** We have invariants, we have tests, but no clear picture of
which invariants are structurally enforced, which are tested but
breakable, which are aspirational, and which are fundamentally not
provable. Without this map, invariants drift — we write rules we can't
verify and skip enforcement we could have.

### Tier 1: Structurally enforced (compiler rejects violations)

These guarantees hold because the code is structured so violations
don't compile or can't be represented.

| Guarantee | Mechanism | What breaks if violated |
|---|---|---|
| Parser token dispatch is typed | `ExpectedToken` enum + exhaustive match | Missing match arm = Rust compile error |
| ExprData variants are closed | Enum with exhaustive match in all walkers | Missing arm = Rust compile error |
| Inference returns `InferredNode` not `Node` | Type system separates resolved/error | Can't accidentally treat error as valid node |
| Arity from `.dag` declarations | Generic slot count read from type def | Can't hardcode arity for a type |
| Fan-out = graph out-degree | Computed from binding reference count | Can't fabricate use-count |

**Property:** violations are unrepresentable. No test needed — the
compiler enforces it. This is the gold standard; all invariants should
aspire to this tier.

### Tier 2: Tested and gated (CI catches regressions)

These guarantees hold because automated tests verify them on every
change. They could be broken by editing the code, but the test suite
catches it.

| Guarantee | Test | CI status |
|---|---|---|
| Inference is name-opaque | 6 scrambled-name tests (rename all types → identical graph) | `cargo test -p v2-compiler-tests` — **in CI** |
| No string-based parser dispatch | Source audit: `ExpectedToken` exists, `kind_matches_tag` absent | `cargo test -p v2-compiler-tests` — **in CI** |
| Canonical accessor pattern | Source audit: `node.children` not raw `expr_data` field access | `cargo test -p v2-compiler-tests` — **in CI** |
| All `.dag` files parse | Parse audit: every `.dag` file parses with 0 errors | `cargo test -p v2-compiler-tests` — **in CI** |
| Clippy clean | `cargo clippy --all-targets -- -D warnings` | **in CI** |
| Self-compile 0 diagnostics | Diagnostic ratchet test (DIAG_RATCHET = 0) | `--ignored` — **manual gate** |
| Bootstrap fixed point | stage0→stage1→stage2 emit identical source | `--ignored` — **manual gate** |
| Self-compile < 30s | Performance ratchet test | `--ignored` — **manual gate** |

**Property:** a regression produces a failing test. The `--ignored`
tests are slower and run manually before merge, not in CI on every
push. They could be promoted to CI if build time permits.

### Tier 3: Machinery exists but not gated (report-only)

The analysis runs and produces results, but nothing blocks on the
output. These are the most dangerous — they give the illusion of
coverage without enforcement.

| Guarantee (aspirational) | Machinery | What's missing |
|---|---|---|
| All functions have proven time bounds | Complexity analyzer produces `ComplexityViolation` list | No test asserts `violations.is_empty()` |
| All functions have proven space bounds | `output_size` field exists in `ComplexitySummary` but only populated for collection-producing ops; most branches return `empty_map()` | Replace `output_size` map with `space: CostExpr` peer to `work`/`span`; populate at every branch using space composition algebra (same walk, different combinators) |
| L1 type knowledge = 0 | `scripts/l1-ratchet.sh` counts violations | Script not in CI; no test wraps it |
| Ownership proof coverage | `ownership.dag` runs, produces ownership annotations | No ratchet on uncovered functions |
| Emitted Rust compiles | Stage0 → `cargo check` | 589 errors; ratchet exists (Lane A) but count still high |

**These are the immediate action items.** Each one has working
machinery — the gap is wiring it into a test that fails on regression.

### Tier 4: Not yet enforceable (design exists, no machinery)

| Guarantee (future) | Design | What's needed |
|---|---|---|
| Parse-emit round trip | `parse(spec, emit(spec, graph)) ≅ graph` | Compositional parser (Stream 0) must land first |
| `parse_item` has 0 keyword arms | R3 exit criteria defined | Ratchet counting keyword match arms in `parse_item` |
| All containers have O(1) clone | FF-8 root-caused; fix designed | Container template change + ratchet on clone cost |
| Complexity bounds are tight (not just proven) | `Conservative` vs `Proven` certainty | CAS-grade precision (log, sqrt) in cost algebra |
| Space bounds are complete | `output_size` tracks collection output | Promote to `space: CostExpr` peer field; populate all branches (same walk as time, different algebra) |
| Import-driven source resolution | FF-9 design written | Compiler resolves imports from source roots, not caller-provided list |

### Tier 5: Fundamentally limited

| Property | Why it can't be fully proven | What we can do instead |
|---|---|---|
| Semantic correctness of emitted code | Generated Rust/Go/Python compiles, but behavior = "does it do what the `.dag` means?" requires end-to-end tests per program | Generated test stubs (Tier 4 test gen) + gist pipeline as integration test |
| Completeness of language coverage | No test that every `.dag` language feature has emission support for every target | Differential: compile same `.dag` to all targets, diff structural coverage |
| Exact complexity bounds | Polynomial algebra can't express log/sqrt/constants | `Conservative` certainty tracks where bounds are loose; CAS upgrade is long-term |
| Correctness of `.dag` type definitions | Algebraic laws (e.g., List is a free monoid) are declarations, not proofs | Property-based tests on generated code exercising the laws |

### Enforcement roadmap

**Immediate (wire existing machinery into gates):**

1. **Complexity ratchet test.** Same pattern as diagnostic ratchet:
   compile self, assert `complexity.violations.len() == 0`. Promotes
   Tier 3 → Tier 2.
2. **L1 ratchet in CI.** Wrap `l1-ratchet.sh --check` in a test or CI
   step. Promotes Tier 3 → Tier 2.
3. **Emitted Rust error ratchet.** Track `cargo check` error count on
   generated stage0; ratchet down from ~280. Promotes Tier 3 → Tier 2.

**Near-term (new machinery, builds on current work):**

4. **Keyword arm ratchet.** Count `ShKw*` match arms in `parse_item`;
   ratchet toward 0 as Stream 0 progresses.
5. **Round-trip smoke test.** Once compositional parser handles a
   subset: parse a `.dag` file, emit it, reparse, assert graph
   equality. Grows with parser coverage.
6. **Ownership coverage ratchet.** Count functions with unresolved
   ownership annotations; ratchet toward 0.

**Structural (promote Tier 2 → Tier 1 by construction):**

7. **Complexity bounds by construction (decidability invariant).**
   The language only permits bounded iteration — `?O(?)` is
   unrepresentable, not caught by a test. The complexity ratchet test
   (Step 1) verifies the *analyzer* sees the bounds, not that programs
   are bounded — programs are bounded by construction. This is the
   model: structural guarantee first, analyzer confirmation second.
8. **Parser extensibility by construction.** When `parse_item` reads
   from `SyntaxSpec` instead of matching on keywords, adding a keyword
   arm becomes impossible — there are no arms to add. Stream 0 endgame.

---

## Design Direction: Boundary Sufficiency

**Problem:** The compiler has ~362 sites where stages branch on
type/method names instead of reading structural facts from data. These
cluster around 4 missing wiring points — places where data exists but the
stage bypasses it. When a boundary doesn't carry a fact the downstream
stage needs, the stage uses the name as a proxy.

**Principle:** A stage boundary is *sufficient* when the data it carries
contains all the structural facts the downstream stage needs, making
name-based proxy reads unnecessary. Sufficiency is additive (enrich
boundaries), not subtractive (restrict access). No policy, no opacity —
just completeness.

**The test:** scramble all user-defined names flowing across a boundary.
If every downstream decision remains identical, the boundary is
sufficient. The existing 6 scrambled-name inference tests already prove
this for resolve→infer. Extending these to emit proves it end-to-end.

**Why not ratchets:** A ratchet counts proxy sites. Fix one missing fact,
and hundreds of sites disappear — but the ratchet tracks them one by one.
Sufficiency testing proves the property directly. It's self-maintaining:
new features get covered when they appear in test programs.

**Critical discovery:** The data already exists on both sides.
`LanguageSpec` in `dsl/std/languages.dag` is fully populated (20+
sub-types, all populated for Rust/Go/Python). `enrich_kernel_type`
already produces correct Conj products. `std/algebra.dag` documents the
hierarchy. The emitter and inference stages *bypass* this data.

### Gap Inventory

| # | Gap | Sites | Data exists at | Stage bypasses with |
|---|-----|-------|----------------|---------------------|
| 1 | Emitter bypasses LanguageSpec | ~57 | `dsl/std/languages.dag` (StatementSyntax, CollectionOps, etc.) | `match target { Rust => "let " ... }` |
| 2 | enrich_kernel_type branches on names | ~120 | `std/algebra.dag` + already-correct Conj products | `if name == "Int" { ... }` |
| 3 | Builtin call sigs are code | ~30 | Should be `.dag` extern fn declarations | `if name == "string_length" { ... }` |
| 4 | Method-name dispatch in inference | ~56 | `intrinsic_method_index()` already maps to enum | `if method_name == "fold" { ... }` |

### Execution Phases

**BS-1 (emit wiring):** Replace ~57 `match target` branches with
LanguageSpec field lookups. The spec is already threaded through emit.
Each `Rust => "let "` becomes `spec.statements.let_binding`. Mechanical.

**BS-2 (algebra data table):** Replace 7-branch `if name ==` chain in
`enrich_kernel_type` with a data map lookup. Define `data kernel_algebra`
mapping type names to field lists. One lookup replaces 7 branches.

**BS-3 (emit sufficiency proofs):** Extend scrambled-name tests to emit.
Compile with normal names and scrambled names, normalize emitted code,
assert structural identity. Proves emit decisions are name-independent.

**BS-4 (method enum dispatch):** Convert `if method_name == "fold"`
string dispatch to `match intrinsic { Fold => ... }` enum dispatch.
Same pattern complexity.dag already uses. Closed-set exhaustive matching
replaces open-set string matching.

```
BS-1 (emit wiring)  ───→ BS-3 (emit sufficiency tests)
                              ↑
BS-2 (algebra data) ───→ BS-4 (method enum dispatch)
```

BS-1 and BS-2 are independent. BS-3 depends on BS-1. BS-4 depends on
BS-2's pattern.

### Progress (2026-03-28)

BS-1 through BS-4 landed. 38 proxy-read sites converted to structural
dispatch. 3 scrambled-name emit tests added (all pass). 156 tests, 0
failures.

### Remaining proxy-read sites (exhaustive inventory)

Every remaining site where the compiler branches on a name. Organized
by the structural fact being proxied.

**Category A: Kernel type identity (12 sites, data-driven)**

These use `is_kernel_type()` and `is_container_type()` which read from
data constants (`kernel_types`, `container_types`). Already data-driven
— the list is declared, not coded as branches. Dissolves when kernel
types are loaded from `.dag` declarations via import resolution (FF-9).

| File | Site | What it checks |
|------|------|----------------|
| `04_types.dag:363,599` | `rt.name != "None" && !is_kernel_type(rt.name)` | User-defined vs kernel in type shape |
| `04_types.dag:623-626` | `is_kernel_type(n.name)` | Wrapping decision in node_type_shape |
| `04_infer.dag:2526` | `is_kernel_type(dep) \|\| dep == "None"` | Cycle detection filtering |
| `04_resolve.dag:150` | `is_container_type(n.name)` | Prevent container expansion |
| `04_resolve.dag:450` | `is_kernel_type(n.name)` | Alias transparency |
| `02_parse.dag:2536` | `is_container_type(n.name) && n.name == "Map"` | Map vs other containers |
| `02_parse.dag:2542` | `n.name == "Refined"` | Refinement type detection |
| `02_parse.dag:2548` | `n.name == ""` | Tuple (anonymous) detection |
| `04_types.dag:341` | `n.name == "Refined"` | Unwrap refined types |

**Category B: Builtin function dispatch (26 branches, 1 function)**

`infer_builtin_call_type` in `04_method.dag:71-93` maps free-standing
builtin function names to return types. These are runtime bridge
functions (`string_length`, `code_point`, `char_at`, `scan_while`,
`lookup`, `map_get`, etc.) whose signatures are not declared in `.dag`.

Fix direction: **compositional .dag modeling**, not a new enum. These
are functions — they should have `.dag` declarations with typed
parameters and return types, loaded into the function environment during
`build_type_env`. Once declared, they resolve through the same
structural lookup path as user-defined functions.
`infer_builtin_call_type` dissolves entirely.

**Category C: Method index maps (2 maps, ~47 entries)**

`intrinsic_method_index()` (19 entries) and
`runtime_bridge_method_index()` (28 entries) in `04_method.dag` map
method name strings to `IntrinsicMethod`/`RuntimeBridgeMethod` enums.

These are inherently name→enum bridges — the compiler needs to
recognize "fold" to know it has special typing. The maps are the right
intermediate step. Long-term: method behavior declared in `.dag` type
algebra definitions, making the enums unnecessary. Short-term: the maps
centralize the knowledge and the enum gives exhaustiveness checking.

**Category D: Optional/sum variant names (14 sites)**

`"Some"`, `"None"`, `"value"` appear in inference, lookup, patterns,
and emit. These are the absence/presence variants of the optional type.

| File | Sites | What it checks |
|------|-------|----------------|
| `04_infer.dag:508` | `variant_name == "Some" \|\| "None"` | Optional cardinality |
| `04_patterns.dag:146` | `variant_name == "None"` | Exhaustiveness for None |
| `04_lookup.dag:93,203` | `field_name == "value"` | Optional inner value access |
| `04_infer.dag:1832` | `fir.typed_field.name == "value"` | Record literal value field |
| `05_emit_rust.dag:1176,1190,1271,1344,1394,2612,2629,2806` | `name == "Some" \|\| "None"` | Rust Option variant rendering |
| `04_types.dag:363,599` | `rt.name != "None"` | Filter None from type shape |

Fix direction: Optional is a structural concept (cardinality on
bindings), not a named type. The remaining `Some`/`None` references are
in emit (rendering Rust's `Option` type). These dissolve when Optional
rendering moves to a LanguageSpec declaration (`absence_variant`,
`presence_variant` fields) — already identified in INVARIANTS.md.

**Category E: Tuple field names (2 sites)**

`"first"`, `"second"` in `04_emit_info.dag:116,125` detect 2-tuples.
These are named product fields — structural (Conj + 2 children + named
"first"/"second"). Dissolves when tuples are positional rather than
named, or when emit reads tuple structure from the node shape rather
than checking field names.

**Category F: Transport kind dispatch (7 sites)**

`transport_kind()` in `00_core.dag:788-791` converts transport node
names to `TransportKind` enum via `if t.name == "local"` etc. Used in
resolve and all 3 emitters.

Already partially structural (the enum exists). Remaining name
comparison is the parse→enum bridge. Dissolves when transport kind is
set structurally at parse time rather than inferred from the node name.

**Category G: Configuration property keys (2 sites)**

`config_property_key()` and `config_property_name()` in `00_core.dag`
convert between string property names and `ConfigPropertyKey` enum.
Small, stable, closed set. Low priority.

**Category H: Diagnostic/AST marker properties (8 sites)**

`p.name == "severity"`, `p.name == "__is_module"`, etc. in
`00_core.dag`. These are structural markers on AST nodes — properties
that classify node purpose. They're the `.dag` equivalent of AST node
kinds. Dissolves when node classification uses ExprData or a structural
field rather than property name strings.

**Category I: Target language dispatch (12 sites)**

`match target { Rust => ... Go => ... Python => ... }` in
`languages.dag` and emit files. These dispatch on the `RenderTarget`
enum (closed set, exhaustive). This is structural — the enum is the
right representation. The issue is when the branch body hardcodes
syntax strings instead of reading from spec. BS-1 addressed the shared
emit cases; per-backend files still have rendering logic that's
inherently target-specific.

**Category J: Builtin value names in emit (3 sites)**

`func == "empty_map"`, `func == "lookup"` in `05_emit_rust.dag`.
Runtime bridge function names used for Rust-specific rendering. Same
fix direction as Category B — declare as `.dag` functions, resolve
structurally.

### Scrambled-name test expansion ideas

The current emit tests use simple struct programs. More complex programs
would stress-test further and catch subtler name dependencies:

1. **Enums with data variants.** `type Color = Red { r: Int } | Green`
   scrambled to `type Shade = Alpha { r: Int } | Beta`. Tests enum
   rendering, variant qualification, match arm construction.

2. **Generic types.** `type Box<T> { value: T }` with `Box<Foo>` usage.
   Tests generic parameter rendering, type substitution in emit.

3. **Container types with user types.** `List<Foo>`, `Map<String, Bar>`.
   Tests container template application with user type arguments.

4. **Method calls on user types.** Functions that call methods on
   struct fields. Tests method dispatch rendering is name-independent.

5. **Nested types.** `type Outer { inner: Inner }` with field access
   `o.inner.x`. Tests chained field access rendering.

6. **Optional fields.** `type Config { name: String? }` with match on
   presence/absence. Tests Optional rendering path.

7. **Services (if supported in test harness).** Service definitions
   with operations. Tests service rendering path.

Each test follows the same pattern: compile A and B with scrambled
user-defined names, normalize, assert structural identity. Adding
these incrementally as emit correctness improves.

### Relationship to existing streams

- **Theme A** (~177 sites): BS-2 + BS-4 (done)
- **Theme B** (~57 sites): BS-1 (done)
- **Stream 1 (L1 dissolution)**: BS-2 directly reduces L1 ratchet
- **Guarantee Map**: BS-3 promotes emit sufficiency from Tier 3 → Tier 2
- **Category B** (builtins): compositional .dag modeling, not new enums

---

## Design Direction: Emission Correctness by Construction (LintModel)

**Problem:** The emission pipeline produces code by string concatenation.
Whether the output is valid (compiles, passes linting) is discovered
only by running external tools after the fact. 589 Rust errors and
parallel Python issues result from the emitter "forgetting" imports,
type params, async annotations, and primitive lowerings. Fixing these
one-by-one is patching; the structural fix is making incorrect emission
unrepresentable.

**Principle:** Model what `cargo check`, `clippy`, `pylint`, `go vet`
etc. enforce as **data per language**. The emitter reads the model.
Correct output is structural, not verified.

```
Current:  emit (string concat) → hope → cargo check (external oracle)
Target:   emit (reads LintModel) → correct by construction → cargo check (redundant)
```

### The three-spec model

```
LanguageSpec — how to RENDER target syntax (keywords, templates, conventions)
SyntaxSpec   — how to PARSE source syntax (item forms, operators, literals)
LintModel    — what makes rendered code VALID (imports, types, expressions, naming)
```

`LanguageSpec` and `SyntaxSpec` already exist. `LintModel` completes the
triangle. Together they model the full language as data, enabling
parse-emit symmetry with correctness guarantees.

### What the LintModel carries

| Rule category | What it models | External tool equivalent |
|---|---|---|
| Import derivation | "using type X in module M → import Y" | `cargo check` E0433, `pylint` import-error |
| Type well-formedness | "generic type must declare params; primitives must lower" | `cargo check` E0412/E0425 |
| Expression well-formedness | "await requires async fn; statements need terminators" | `cargo check` E0728, `pylint` syntax-error |
| Naming conventions | "types PascalCase, functions snake_case" | `clippy` naming warnings, `pylint` naming |
| Formatting | "indentation width, brace style, import grouping" | `rustfmt`, `black`, `gofmt` |

### How it clears 589 errors by construction

| Error category | LintModel rule | Construction guarantee |
|---|---|---|
| Deserialize import (316) | `TraitImpl("Deserialize")` → `use serde::Deserialize;` | Can't render trait impl without import |
| Generic params (122) | Generic type declaration must carry param list from DAG | Can't render `T` without declaring `<T>` |
| `Bool`→`bool` (40) | `Primitive("Bool")` → `"bool"` (read from `rust_primitives`) | All primitives lowered via type map |
| `async fn` (17) | `AwaitInBody` → async function declaration | Can't render await without async |
| `FreeMonoid`→`Vec` (12) | `AlgebraicType("FreeMonoid")` → `"Vec"` | Algebraic types lowered via type map |
| `Callable` (9) | `CallableType` → `Rc<dyn Fn(T) -> U>` | Callable rendering is a type rule |
| Service vars (20) | Service dep → in-scope binding construction | Can't render service call without binding |
| String escapes (6) | Brace escaping rule per interpolation syntax | Format string rendered per language spec |

### Swappable models

LintModel instances are data, not code. Different instances for different
strictness levels:

```
rust_standard_lint    — cargo check + default clippy
rust_strict_lint      — clippy::pedantic, no unwrap, no expect
python_standard_lint  — py_compile + pylint defaults
python_strict_lint    — ruff strict, mypy
```

The emitter is generic over the model. `--lint strict` vs `--lint standard`
selects the instance. The language extdep carries multiple model instances;
the CLI flag picks one.

### Import derivation — highest leverage rule

Import derivation clears 316 of 589 errors (54%) by construction. The
mechanism:

1. Emitter renders a type reference → records a `TypeUsage` fact
2. Emitter renders a trait impl → records a `TraitUsage` fact
3. At module close, the renderer reads the LintModel's import rules
4. Each `TypeUsage`/`TraitUsage` → matching `ImportRule` → `use` statement
5. Import statements emitted in module preamble, grouped and sorted

Missing imports are **structurally unrepresentable**. If you rendered
the type, the import exists. This is the same principle as
`ExpectedToken` making invalid parser dispatch unrepresentable.

### Internal unit testing (not external tool invocation)

Each LintModel rule is testable in isolation:

| Test kind | What it verifies | Speed |
|---|---|---|
| **Model test** | "Rust import model: Deserialize impl → `use serde::Deserialize;`" | Fast, no I/O |
| **Pipeline test** | "Emitter records TypeUsage when rendering type reference" | Fast, no I/O |
| **Renderer test** | "Module renderer: collected usages → correct `use` statements" | Fast, no I/O |

These test the MODEL's rules, not the output's text. They're ordinary
unit tests that run in `cargo test`, not `--ignored`.

### Hermetic integration tests (confirmation, not gate)

As redundant confirmation, hermetic `#[test]` functions invoke external
tools on a known `.dag` → emit → tool → assert pipeline:

| Test | External tool | Pattern |
|---|---|---|
| `rust_emit_compiles` | `cargo check` on emitted Rust | Same as existing `python_test_file_syntax_valid` |
| `rust_emit_lints_clean` | `cargo clippy` on emitted Rust | subprocess, assert exit 0 |
| `python_emit_compiles` | `python -m py_compile` on emitted Python | Already exists in `pipeline.rs` |
| `go_emit_compiles` | `go build` on emitted Go | subprocess, assert exit 0 |

These are `--ignored` tests (slow, need toolchains installed). They are
**self-contained unit tests**, not a maintained CI process. They confirm
the LintModel is complete — the model guarantees correctness; the
integration test confirms the guarantee.

### Relationship to existing infrastructure

The data already partially exists:

| Existing file | What it has | What's missing |
|---|---|---|
| `rust/imports.dag` | `use` templates, crate structure, `base_deps` | Derivation rules (WHEN to import) |
| `rust/types.dag` | `rust_primitives` with `Bool→bool` mapping | Emitter reading this map for ALL rendering |
| `rust/async.dag` | `async fn` templates, `await_suffix` | Propagation rule (body with await → async fn) |
| `LanguageSpec` | Syntax rendering, serialization, scaffold | `lint: LintModel` field |

The LintModel connects existing data to the emission pipeline via rules.
The data is modeled; the rules are not.

### Enforcement (see Guarantee Map)

| Tier | What | When |
|---|---|---|
| **Tier 1** (structural) | Import derivation: can't render type without import | E1 lands |
| **Tier 1** (structural) | Type well-formedness: can't render generic without params | E2 lands |
| **Tier 2** (tested) | Internal model tests: verify rules produce correct output | E4 lands |
| **Tier 2** (tested) | Hermetic integration: `cargo check` = 0 errors | E5 lands |
| **Tier 2** (tested) | Hermetic linting: `cargo clippy` = 0 warnings | E5 lands |

---

## Current State (2026-03-28)

**TOP PRIORITY: Compiler thesis inversion (~362 hardcoded name-based
branches).** The compiler encodes knowledge as code branches instead of
reading data. Three themes: (A) type/method dispatch by string
comparison, (B) emission hardcodes target syntax instead of reading
LanguageSpec, (C) testgen doesn't test compiler. See "Compiler
structural audit" section below. **Fix direction: Boundary Sufficiency
(BS-1 through BS-4).** The data already exists (LanguageSpec is fully
populated, algebra products are correct); stages bypass it. See
"Design Direction: Boundary Sufficiency" for the 4-gap analysis and
execution phases.

**PRIORITY 2: Diagnostic quality.** The DSL is unusable until
diagnostics include file name, line:column, source context, and
actionable suggestions. See "Diagnostic quality" section below.

**Phases 1-4 complete. Phase 5 active. Stream 0 (compositional parser)
landed (PR #226).**

**Stream 0 complete (2026-03-28, PR #226).** `SyntaxSpec` type landed
in `languages.dag`. Item dispatch, operator precedence, and literal
keywords are all spec-driven. `parse_item` has 0 keyword match arms.
30+ `ShKw*` variants consolidated to single `ShKeyword`. 70+ keyword
predicates deleted. Net -170 lines.


**Bootstrap status:** v1 retired (PR #200). v2 self-hosts. Stage0
binary compiles all .dag source: **46 files emitted, 0 diagnostics.**
151 fast tests pass, clippy clean. Self-compile time: ~260s (FF-8).

**Verification ratchets (Lane A complete, PR #227):**
- Diagnostic ratchet: 0 (passes)
- Emitted Rust error ratchet: 589 errors (down from 872)
- L1 ratchet: 51 (delegates to scripts/l1-ratchet.sh)
- Keyword arm count: 9 (exact match)
- Complexity ratchet: 2 violations (pre-existing)

**Codegen audit (2026-03-28): 589 Rust errors, 9 categories.**
Prior fixes (PR #229) were partial — serde removed from NonEmpty
wrappers but `impl Deserialize` still emitted without `use` imports;
generics fixed for some types but `FreeMonoid<T>` etc. still emit `T`
undeclared. Full breakdown:

| Category | Count | Root cause |
|----------|------:|------------|
| Deserialize trait not found | 316 | Emitter generates `impl Deserialize` but doesn't add `use serde::Deserialize;` at module top |
| Type param `T` not found | 122 | Generic types like `FreeMonoid<T>` emit `T` without declaring it as a type parameter |
| `Bool` type not found | 40 | `.dag` `Bool` → Rust should be `bool` (primitive lowering) |
| Service vars not in scope | ~20 | `github_pulls`, `shell_exec`, `cron_tab` etc. — service instances not instantiated |
| `await` outside async | 17 | `func` should emit `async fn` but emits sync `fn` |
| `free_monoid` type not found | 12 | `FreeMonoid<T>` not lowered to `Vec<T>` |
| `Callable` type not found | 9 | `fn(T) -> U` type syntax not mapped to Rust `Fn` trait |
| String escape `\{ \}` | 6 | Literal brace escapes in strings not handled |
| Missing mock data | 6 | Dry-run mode references missing mock responses |
| Other | ~41 | Missing values/imports, trait resolution |

Top 5 by impact (fixing these clears ~95%):
1. **Deserialize `use` statement** (316) — add `use serde::Deserialize;` to every module that has `impl Deserialize`
2. **Generic type params** (122) — `FreeMonoid<T>`, `PartialFunction<K,V>` need actual generic `struct` declarations with `<T>`, `<K,V>`
3. **`Bool`→`bool`** (40) — primitive type lowering
4. **`async fn`** (17) — `func` should emit `async fn`; service instances should be passed as params or constructed
5. **`FreeMonoid`→`Vec`** (12) — algebraic types should lower to Rust stdlib types

**Python codegen has the same structural issues** (match syntax,
statement/expression confusion, async). Both backends need the same
set of fixes in the emit pipeline — the root cause is shared emit
not reading enough from `LanguageSpec`.

**DSL compilation:** 78 files, 0 diagnostics, 94 files emitted (PR #228).
Compiler correctly fails closed (0 files emitted when any diagnostic exists).

**Known compiler invariant violations (worked around in DSL, must fix):**

Three structural bugs in the compiler forced DSL workarounds to reach 0
diagnostics. Each violates a core invariant. All workarounds are marked
in the DSL source with comments explaining what they work around.

| Bug | Invariant violated | Workaround | Files affected |
|-----|--------------------|------------|----------------|
| **`uses` variables not bound in scope.** Compiler parses `uses fs: Filesystem` and collects resource names for metadata, but never adds them to `scope.locals` during inference. Any pattern/func body that references `fs.read()`, `fs.probe()`, `fs.write()` fails with "undefined variable 'fs'". | Resources declared in `uses` clauses must be available as typed variables in the body scope. | Commented out 4 pattern bodies in `std.patterns` and 1 func body in `tools.codegen`. | `v2_compiler_infer.rs` — scope construction for func/pattern items |
| **Optional exhaustiveness hardcodes `Some`/`None`.** When matching on `T?`, the checker uses `vec!["Some", "None"]` as the variant names. Inner type variants (`TargetDir`, `Broken`, etc.) don't satisfy it. `null` keyword also doesn't satisfy it. Only a wildcard `_` or bind pattern bypasses the check. | Exhaustiveness checking must recognize the scrutinee's inner type variants when matching through an optional wrapper. | Added `_` wildcard arms in `std.filesystem` `skip_reason`. | `v2_compiler_infer_patterns.rs:264-265` |
| **Single-variant enums parsed as type aliases.** `type X = Y` where Y is a new variant name is treated as a type alias (lookup of existing type Y), not a one-variant enum definition. Fails with "unresolved type 'Y'". | `type X = Y` must define a one-variant coproduct, not alias an existing type. Parser/resolver must distinguish variant introduction from type reference. | Changed `StsGrantType = TokenExchange` to `String`. `CacheControl = Ephemeral` fixed on main (PR #229) similarly. | Parser or `v2_compiler_infer_env.rs` — type definition processing |

Additional workarounds applied (not compiler bugs, parser limitations):
- `local_auth()` commented out: parser doesn't support `if/else` as inline expression (`expected LBrace, found KwElse`).
- `scheme: AuthScheme = Bearer` changed to `String = "Bearer"`: compiler expects CLI parameter defaults to be string literals, not enum constructors.
- `char_display_width` restructured: ownership checker flags enum constructors used in multiple return paths as "2 consumers" (false positive on constructors vs bindings).

**Compiler structural audit (2026-03-28): ~362 hardcoded name-based branches.**

The compiler encodes knowledge as code branches rather than as data it
reads. This inverts the project thesis ("smart facts, dumb compiler").
Three themes, all rooted in the same cause: the compiler "knows" names.

**Theme A: Type/method dispatch is name-based (~177 sites).**

`infer_types.rs` has an `if/else` chain checking `"Int"`, `"Float"`,
`"Bool"`, `"String"`, `"List"`, `"Map"`, `"Callable"` — each branch
injects algebra methods. `infer_method.rs` has a 56-branch `if/else`
chain dispatching on method name strings (`"fold"`, `"map"`, `"count"`,
`"join"`, etc.). This is why `sum` and `repeat` were missing — adding a
method means editing compiler source. 107 hardcoded type-name
comparisons, 70 method-name comparisons.

| File | Type-name checks | Method-name checks | Total |
|------|------------------|--------------------|-------|
| `v2_compiler_infer_method.rs` | 0 | 56 | 56 |
| `v2_compiler_infer_types.rs` | 27 | 0 | 27 |
| `v2_compiler_infer.rs` | 7 | 10 | 17 |
| Other (5 files) | 5 | 4 | 9 |

**Invariant violated:** Types and methods should be defined in `.dag`
data declarations and resolved structurally. The compiler should read
the algebra registry, not contain it.

**Fix direction:** The algebra registry (`std.algebra`) already exists
in the DSL. The compiler should load it at startup and use it for method
resolution, replacing the `if/else` chains. This is a data-loading
change, not an architecture change.

**Theme B: Emission hardcodes target-language syntax (~57 sites).**

`v2_compiler_emit.rs` contains `"let "`, `"vec![]"`,
`.unwrap_or_else(|| ...)`, `"compile_error!()"`, `"|x| "` (lambda
syntax) — all hardcoded per `RenderTarget`. The `LanguageSpec` type
exists and the language extdep data files already provide type maps and
container templates, but the emitter bypasses them for ~57 constructs.

Missing from `LanguageSpec` (11 fields needed):

| Missing field | Currently hardcoded as |
|---------------|----------------------|
| `statement_terminator` | `";"` (Rust), `""` (Python/Go) |
| `variable_declaration_keyword` | `"let "` (Rust), `""` (Python/Go) |
| `assignment_operator` | `" = "` (Rust/Python), `" := "` (Go) |
| `lambda_syntax` | `"\|x\| "` (Rust), `"lambda x: "` (Python), `"func(x) { }"` (Go) |
| `callable_type_template` | `"Rc<dyn Fn(...)>"` (Rust), `"Callable[...]"` (Python) |
| `error_expression` | `"compile_error!()"` (Rust), `"raise RuntimeError()"` (Python) |
| `null_coalesce` | `.unwrap_or_else(...)` (Rust), `"or"` (Python) |
| `string_interpolation` | `format!(...)` (Rust), `f"..."` (Python), `fmt.Sprintf(...)` (Go) |
| `container_bracket` | `"<>"` (Rust/Go), `"[]"` (Python) |
| `tuple_type_template` | `"(A, B)"` (Rust), `"Tuple[A, B]"` (Python) |
| `indentation_width` | `4` (all targets) |

**Invariant violated:** Languages are extdeps modeled from specs.
Emission must read language data, not contain it.

**Fix direction:** Extend `LanguageSpec` with the 11 missing fields,
populate them in the language extdep `.dag` files, and replace the
`match render_target` branches in the emitter with spec lookups.

**Theme C: Testgen doesn't test the compiler; Go/Python are stubs.**

- `generated_tests.rs` (26,692 lines) is a static snapshot of `.dag`
  source embedded as const strings for tokenizer tests — not generated
  tests of compiler behavior.
- Rust test emission (`emit_test_file()`) works but Go/Python emit
  `assert True` placeholder stubs.
- Bootstrap tests verify the compiler produces 0 diagnostics and that
  stage0→stage1 is a fixed point, but the emitted Rust has ~872
  `cargo check` errors — so the compiler cannot verify its output
  actually compiles.
- No test that inference/emission produces *correct* code, only that
  it produces *some* code with 0 diagnostics.

**Fix direction:** (1) Fix the 872 codegen errors so emitted Rust
compiles. (2) Add a "golden output" test: compile a small `.dag` file,
`cargo check` the output, run the generated tests. (3) Fill Go/Python
test stubs so cross-language parity is testable.

**Diagnostic quality: INSUFFICIENT.** Diagnostics report byte offsets
with no file name, no line:column, no source context. Implementation
path: file name in SourceSpan, byte→line:column, source-context
rendering, parse-context threading, suggestions.

**Known invariant violation: Option rendering splits absence variants.**
Absence-variant rendering should be a LanguageSpec declaration, not
an emitter heuristic. See details below.

**L2 bridge dissolved** (P5.11 complete, 2026-03-26). ExprData children
in `node.children`. Bridge functions deleted.

**Container sharing (FF-8):** Root-caused 2026-03-27. Rendering change
in `LanguageSpec` container templates. Hand-patch proof: 37s → 0.4s.
Fix pending (Stream 3).

**Root-cause audit (2026-03-23):** Three root causes behind all ~66
invariant violations — I (incomplete types ~32), II (error-as-name ~18),
III (divergent paths ~17). Most symptoms resolved through Phases 1-4.

**Foundational directions for Phase 5 exit:**
- **Compiler thesis inversion — TOP PRIORITY (~362 name-based branches)**
- **Diagnostic quality — PRIORITY 2 (blocks DSL usability)**
- **Stream 0 (compositional parser) — LANDED (PR #226)**
- **Decidability (DAG-reducibility) — active**
- **Guarantee enforcement (all Tier 3 → Tier 2) — Lane A done (PR #227)**

---

## PLACEHOLDER_DELETE_BELOW

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

### Stage 2: Rust Codegen (589 errors)

| Gap | Errors | Root cause | Fix location |
|-----|-------:|------------|--------------|
| Deserialize `use` import | 316 | `impl Deserialize` emitted without `use serde::Deserialize;` | `05_emit_rust.dag` module preamble |
| Generic type params undeclared | 122 | `FreeMonoid<T>` emits `T` without `<T>` on struct/enum | `05_emit.dag` generic emission |
| `Bool` → `bool` | 40 | Primitive type lowering missing | `05_emit_rust.dag` type map |
| Service vars not in scope | ~20 | Service instances (`github_pulls`, etc.) not instantiated | `05_emit.dag` service rendering |
| `await` outside `async` | 17 | `func` emits sync `fn`, should be `async fn` | `05_emit_rust.dag` func rendering |
| `FreeMonoid` → `Vec` | 12 | Algebraic types not lowered to stdlib equivalents | `05_emit_rust.dag` type map |
| `Callable` type | 9 | `fn(T) -> U` not mapped to `Fn` trait | `05_emit_rust.dag` type map |
| String brace escapes | 6 | `\{` `\}` not handled in string emission | `05_emit.dag` string rendering |
| Missing mock data | 6 | Dry-run mode references undefined mock responses | `05_emit.dag` mock rendering |
| Other | ~41 | Missing values/imports, trait resolution | Various |

Priority: Deserialize `use` (316) + generic params (122) + `Bool`→`bool` (40)
clears ~80%. Adding `async fn` (17) + `FreeMonoid`→`Vec` (12) clears ~95%.
Python has the same structural issues — both backends need the same emit fixes.

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
| **Stream 4: Guarantee Enforcement** | *(unassigned)* | Wire Tier 3 machinery into gates; add Tier 4 ratchets as design directions land | Complexity ratchet, L1 ratchet in CI, keyword arm ratchet, round-trip smoke test |
| **Stream 5: Compiler Correctness** | *(unassigned)* | Space complexity tracking; regeneration script | Space bounds in complexity report; regen script works |
| **Stream 6: Emission Correctness (LintModel)** | *(unassigned)* | LintModel type, import derivation, type/expr well-formedness, hermetic integration tests | Emitted Rust/Python/Go compiles, lints clean; 0 errors by construction not verification |

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

**Enforcement (see Guarantee Map):**
- **Immediate (Tier 3→2):** Keyword arm ratchet — count `ShKw*` match
  arms in `parse_item`, fail if above ratchet value. Tracks Stream 0
  progress mechanically.
- **Near-term (Tier 4→2):** Round-trip smoke test —
  `parse(spec, emit(spec, graph)) ≅ graph` on subset of `.dag` files.
  Grows as parser coverage grows.
- **Endgame (Tier 2→1):** When `parse_item` reads from `SyntaxSpec`,
  keyword arms are structurally impossible — there are no arms to add.

### Stream 4: Guarantee Enforcement — Implementation Plan

**Goal:** Every invariant in INVARIANTS.md has a corresponding
enforcement mechanism. No Tier 3 (report-only) items remain at Phase 5
exit.

**Step 1: Wire existing machinery into ratchet tests.**
Three tests, same pattern as the existing diagnostic ratchet in
`bootstrap.rs`:

| Test | What it checks | Ratchet value |
|---|---|---|
| `v2_complexity_violation_count` | `complexity.violations.len()` from self-compile | 0 (all functions have proven bounds) |
| `v2_l1_ratchet` | Wrap `l1-ratchet.sh --check` or port the regex counts into Rust | 420 → 0 |
| `v2_emitted_rust_error_count` | `cargo check` error count on generated stage0 | ~280 → 0 (tracks Stream 5) |

**Step 2: Add structural ratchets for active design directions.**

| Test | Tracks | Ratchet direction |
|---|---|---|
| `v2_parse_item_keyword_arms` | Stream 0 progress | Count `ShKw*` match arms in `parse_item` → 0 |
| `v2_space_complexity_coverage` | Stream 5 space tracking | Count functions with `output_size: empty_map()` → 0 |
| `v2_ownership_coverage` | Ownership proof maturity | Count functions with unresolved annotations → 0 |

**Step 3: Publish guarantee inventory.**
The Guarantee Map in this roadmap is the design doc. The test suite
is the executable version. Each ratchet test's doc comment links to the
corresponding Guarantee Map tier and the invariant it enforces. Anyone
can run `cargo test -p v2-compiler-tests -- --ignored` and see exactly
which guarantees hold and which are still ratcheting down.

### Stream 5: Compiler Correctness — Implementation Plan

**Goal:** Generated Rust compiles. Complexity analysis covers both time
and space. The regeneration script works reliably.

**Track A: Emitted Rust errors (~280 → 0)**

The errors are in three layers (see "Rust Codegen Issues" in Backlog).
Priority order by error yield:

| Fix | Errors cleared | Location |
|---|---|---|
| `Bool`→`bool`, `Unit`→`()` primitive mapping | ~154 | `05_emit_rust.dag` |
| Remove serde `Deserialize` derives | ~124 | `05_emit_rust.dag` |
| Emit P5.11 accessor functions (`let_value`, etc.) | ~130 | `05_emit.dag` |
| Fix `regenerate-stage0.sh` module naming | ~15 | Script |
| Remaining (duplicate types, undeclared refs) | ~12 | `05_emit.dag` namespace |

Many errors overlap (a single `Bool` type produces multiple downstream
errors), so the actual fix count is smaller than the error sum.

**Track B: Space complexity — same walk, different algebra**

Time and space are not separate analyses. The complexity analyzer
already walks the expression tree once and computes `work`, `span`,
and `output_size` in the same pass (`cost_of_expr`). The walk is
shared — the only difference is the composition operator at each node:

| Pattern | Time (work) | Space (peak live) |
|---|---|---|
| Sequential `a; b` | `add(a, b)` | `max(a, b)` |
| Parallel (match arms) | `max(a, b)` | `add(a, b)` |
| Loop `n × body` | `mul(n, body)` | `body` (streaming) or `mul(n, body)` (accumulating) |
| Recursion (depth × frame) | `mul(depth, frame)` | `mul(depth, frame)` (stack) |

The current gap is not a missing analyzer — it's that most branches
in `cost_of_expr` return `output_size: empty_map()` instead of
computing the space contribution. The fix is populating the existing
field using the correct algebra, not building separate machinery.

**Step 1:** Replace `output_size: Map<String, CostExpr>` with a
proper `space: CostExpr` field in `ComplexitySummary` — a peer to
`work` and `span`, not a second-class map. All three are `CostExpr`
trees composed by the same walk.

**Step 2:** Populate `space` at every branch in `cost_of_expr` using
the space composition algebra. Sequential = `max`, parallel = `add`,
loop = `body` or `mul` depending on accumulation. Most of the changes
are replacing `empty_map()` with the correct `cost_max`/`cost_par`
call — same structure as the `work` line, different combinator.

**Step 3:** Stack depth falls out of decidability. If every recursive
function has a proven descent measure, stack depth = that measure.
No new machinery — the bound is already computed for time; space
reuses it.

**Step 4:** Clone cost = fan-out × representation size. Once
containers are `Rc`-wrapped (Stream 3), clone cost is O(1) and the
space analyzer confirms it. Before the fix, the analyzer should
report the O(n) clone cost — making FF-8 class regressions visible
in the complexity report.

**Track C: Regeneration script**

`regenerate-stage0.sh` has known bugs (module renaming vs `lib.rs`,
serde imports, duplicate types). Fix the script so `./regenerate-stage0.sh && cargo check` passes. This is the practical gate
for Stream 5 Track A — without a working regen script, every codegen
fix requires manual stage0 porting.

### Gaps identified (2026-03-28)

Two items surfaced during the workboard design:

**1. `RecursionPattern` is declared but never used.** The complexity
analyzer defines `LinearRecursion | DivideAndConquer |
UnresolvableRecursion` (complexity.dag:201-204) but never calls it.
`cost_of_expr` recurses into children without classifying the recursion
pattern. This is the decidability enforcement point — wiring
`RecursionPattern` into the walk is how the analyzer proves (or
rejects) recursive functions.

**2. Space complexity would have caught FF-8.** The 20-minute
self-compile (FF-1/FF-8) was a space problem: O(n) clones on bare
`Vec`/`HashMap` inside loops. If space had been a first-class dimension
in the complexity report, the analyzer would have flagged
`O(|tokens|²)` space on the parser before anyone ran it. This
connects Stream 3 (container sharing) to Stream 5 Track B (space) —
the fix makes clone cost O(1), and the space analyzer proves it stays
that way.

### Stream 6: Emission Correctness (LintModel) — Implementation Plan

**Goal:** Emitted code compiles and lints clean by construction, not by
running external tools. The LintModel carries target-language validity
rules as `.dag` data. The emitter reads the model. Incorrect emission
is structurally unrepresentable.

**E1: LintModel type + import derivation (clears ~316 errors)**

1. Define `LintModel`, `ImportRule`, `ImportTrigger` types in `languages.dag`:

   ```
   type ImportTrigger
     = TypeUsage { type_name: String }
     | TraitImpl { trait_name: String }
     | DeriveMacro { macro_name: String }
     | ContainerUsage { container: String }
     | AsyncUsage

   type ImportRule {
     trigger: ImportTrigger
     import_path: String
   }

   type LintModel {
     name: String
     import_rules: List<ImportRule>
     type_rules: TypeWellFormedness
     expr_rules: ExprWellFormedness
     naming: NamingConvention
     formatting: FormatModel
   }
   ```

2. Add `lint: LintModel` field to `LanguageSpec`.
3. Define `rust_standard_lint`, `python_standard_lint`, `go_standard_lint`
   in language extdeps — import derivation rules for each language.
4. Wire import collection into emitter: rendering a type/trait records a
   `TypeUsage` fact; module close derives `use` statements from LintModel
   import rules.
5. Internal model tests: for each `ImportRule`, verify the correct `use`
   statement is derived.

**Gate:** 316 Deserialize errors → 0 (by construction).

**E2: Type well-formedness rules (clears ~174 errors)**

1. Define `TypeWellFormedness` rules:

   ```
   type TypeWellFormedness {
     primitive_map: List<PrimitiveLowering>      // Bool → bool
     algebraic_map: List<AlgebraicLowering>      // FreeMonoid<T> → Vec<T>
     callable_template: String                   // fn(T)->U → Rc<dyn Fn(T)->U>
     generic_params_from_dag: Bool               // struct Foo<T> from DAG slots
   }
   ```

2. Wire primitive lowering to read from `rust_primitives` (already in
   `types.dag` — `Bool→bool` is modeled, emitter just doesn't read it
   everywhere).
3. Add algebraic lowering rules: `FreeMonoid→Vec`, `PartialFunction→HashMap`,
   `BooleanAlgebra→bool` with explicit map in language extdep.
4. Generic param enforcement: struct/enum declarations MUST carry params
   from the DAG type definition's generic slots. Not optional.
5. Callable rendering: `Callable` types → language-specific function type
   (`Rc<dyn Fn(T) -> U>` in Rust, `Callable[[T], U]` in Python).
6. Internal tests: each lowering rule produces correct declaration.

**Gate:** 122 generic + 40 Bool + 12 FreeMonoid + 9 Callable = 183 errors → 0.

**E3: Expression well-formedness rules (clears ~43 errors)**

1. Define `ExprWellFormedness` rules:

   ```
   type ExprWellFormedness {
     async_propagation: Bool           // await in body → async fn
     statement_terminator: String      // ";" (Rust), "" (Python)
     brace_escape_in_format: Bool      // \{ → {{ in format!()
     service_binding_strategy: ServiceBindingStrategy
   }

   type ServiceBindingStrategy
     = ParamInjection                  // service as fn param
     | ConstructInBody                 // let client = Client::new()
     | GlobalSingleton                 // lazy_static / once_cell
   ```

2. Async propagation: walk function body for `await` usage, set function
   declaration to `async` if any await found. Data-driven per language
   (Rust: `async fn`, Python: `async def`, Go: goroutine spawn).
3. Service instance construction: LintModel declares the strategy;
   emitter reads it to produce the correct binding pattern.
4. String brace escaping: `format!()` requires `{{`/`}}` for literal
   braces; f-strings require `{{`/`}}`. Rule per language.
5. Internal tests: each rule produces valid expressions.

**Gate:** 17 async + 20 service + 6 string = 43 errors → 0.

**E4: Naming + formatting (quality, not correctness)**

1. Define naming convention rules (already partially in LanguageSpec):

   ```
   type NamingConvention {
     types: CaseStyle          // PascalCase (Rust/Go), PascalCase (Python)
     functions: CaseStyle      // snake_case (Rust/Python), camelCase (Go)
     modules: CaseStyle        // snake_case (Rust/Python), lowercase (Go)
     constants: CaseStyle      // SCREAMING_SNAKE (Rust/Python), PascalCase (Go)
     enum_variants: CaseStyle  // PascalCase (Rust), SCREAMING_SNAKE (Proto)
   }
   ```

2. Define formatting model (partially in LanguageSpec `indentation_width`):

   ```
   type FormatModel {
     indent_width: Int                  // 4 (Rust/Python), tab (Go)
     indent_char: String                // " " or "\t"
     max_line_width: Int?               // 100 (rustfmt default)
     import_grouping: ImportGroupStyle  // std/external/crate (Rust)
     trailing_newline: Bool
   }
   ```

3. Internal tests: rendered names/formatting match convention rules.

**E5: Hermetic integration tests (confirmation, not gate)**

1. `rust_emit_compiles`: compile known `.dag` file → emit Rust → write
   to temp dir with `Cargo.toml` → `cargo check` → assert exit 0.
   Same pattern as existing `python_test_file_syntax_valid` in `pipeline.rs`.
2. `rust_emit_lints_clean`: same pipeline → `cargo clippy -- -D warnings`
   → assert exit 0.
3. `python_emit_compiles`: extend existing `ast.parse` test to full
   corpus. Add `mypy --strict` for type-checked Python.
4. `go_emit_compiles`: emit Go → write temp dir → `go build` → assert
   exit 0.
5. All marked `#[ignore]` — slow, need toolchains. Run manually or in
   CI with toolchain matrix. Self-contained unit tests, not maintained
   processes.

**Gate:** All integration tests pass (redundant — should already pass
if LintModel rules are correct).

**E6: Swappable models + cross-language parity**

1. Define `rust_strict_lint` (clippy pedantic), `python_strict_lint`
   (ruff strict + mypy). Add to language extdeps alongside standard models.
2. CLI flag: `--lint standard | --lint strict` selects model instance.
3. Python codegen: apply same LintModel rules to fix match syntax,
   statement/expression confusion, async patterns.
4. Go codegen: apply LintModel rules for Go-specific patterns
   (implicit interfaces, goroutines, error returns).
5. Cross-language parity test: compile same `.dag` file to all targets,
   all pass their respective integration tests.

**Gate:** Rust + Python + Go all produce valid, linted code from the
same `.dag` source.

### Stream 6 execution order

```
E1 (import derivation)     E2 (type rules)     E3 (expr rules)
     │                          │                    │
     │ 316 errors               │ 183 errors         │ 43 errors
     │                          │                    │
     ├──────────────────────────┼────────────────────┤
     │                          │                    │
     ▼                          ▼                    ▼
E4 (naming/formatting)    ← quality layer, depends on E1-E3 pipeline
     │
     ▼
E5 (hermetic integration) ← confirmation layer, validates E1-E4
     │
     ▼
E6 (swappable + parity)   ← multi-language, depends on E5 passing
```

E1-E3 can run in parallel (independent rule categories). E4 depends on
the emission pipeline changes from E1-E3. E5 depends on E1-E3 clearing
errors. E6 extends to multiple languages and strictness levels.

---

## Workboard: Parallel Lanes

**Lane A (Verification) DONE (PR #227).** Ratchets are in place:
diagnostic (0), emitted Rust errors (1184/1200), L1 (51), keyword
arms (9), complexity (2). Space complexity (Lane A Phase 2) deferred
until Lane B drives error count down.

**Current priority: Lane B (Compiler Output).** 589 Rust codegen
errors block bootstrap regeneration and the committed-binary approach.
Python codegen has parallel structural issues. Both backends need the
same emit pipeline fixes. **Approach: Stream 6 (LintModel)** — model
target-language validity rules as data; emit correct code by
construction, not verification.

**Lanes B and C run in parallel.** They touch different files and have
independent exit criteria.

```
Lane A: Verification ✓           Lane B: Compiler Output       Lane C: Language Design
(DONE — PR #227)                 (make output correct)         (make the language right)
─────────────────────────        ─────────────────────────     ─────────────────────────
✓ Emitted Rust error ratchet     Stream 6: LintModel            Stream 0: Compositional
✓ Complexity ratchet (2)         (correctness by construction)    Parser (R3)
✓ L1 ratchet (51, via script)   ┌─────────────────────┐       · SyntaxSpec extraction
✓ Keyword arm count (9)         │ E1: Import rules(316)│       · parse_item reads spec
                                │ E2: Type rules (183) │       · round-trip smoke test
Deferred:                       │ E3: Expr rules  (43) │
· Space as peer dimension       │ E4: Naming/format    │     Decidability invariant
                                │ E5: Hermetic tests   │       · Audit iteration prims
                                │ E6: Swap + parity    │       · Wire RecursionPattern
                                └─────────────────────┘
                                 Stream 5C: Regen script      Stream 1: L1 Dissolution
                                   · ✓ cargo run (Docker)       · Type constructors → 0
                                   · Regenerate + commit         · Type-name comparisons
                                 Stream 3: Container FF-8        · CollectionKind ✓ dissolved
                                   · Rc<Vec<{0}>> templates
                                   · Atomic with regen
─────────────────────────        ─────────────────────────     ─────────────────────────
Exit: ✓ Done                     Exit: emitted Rust/Python/Go   Exit: parse_item 0 keyword
                                   compiles + lints clean by       arms; decidability is
                                   construction; hermetic tests    structural (Tier 1);
                                   confirm; FF-8 eliminated        L1 ratchet = 0
```

### Lane A status (2026-03-28): DONE

All ratchets delivered in PR #227. The verification gap is closed:
regressions in compiler output, L1 type knowledge, parser keyword
arms, and complexity violations are now visible and mechanically
tracked. Lane B progress is measurable via the emitted Rust error
ratchet (1184 → target 0).

### Cross-lane dependencies

```
Lane A ──→ Lane B:  Emitted Rust ratchet makes Lane B progress
                     measurable. Without it, fixes are unverifiable.

Lane A ──→ Lane C:  Complexity ratchet verifies decidability — the
                     analyzer confirms the structural guarantee. Not
                     a blocking dependency (Tier 1 guarantee holds
                     by construction) but provides observability.

Lane B ──→ Lane B:  Stream 6 (LintModel) is the approach for Lane B.
                     E1-E3 clear errors structurally; E5 confirms.
                     Regen script (5C) unblocks stage0 regeneration.

Lane B ──→ Lane A:  Container sharing (FF-8) feeds space complexity
                     clone cost model. Space analyzer can land first;
                     FF-8 fix makes clone cost O(1) and analyzer
                     confirms.

Lane C ──→ Lane A:  Compositional parser enables round-trip smoke
                     test. Can start on subset before full parser.
```

### Execution order

**Lane A: ✓ DONE** (PR #227)

**Lane B (current priority — 589 errors → 0 via Stream 6 LintModel):**

Prior fixes (PR #229):
- ✓ Fix regen script for Docker (cargo run instead of binary path)
- ✓ Remove serde from NonEmpty wrappers (partial — 316 Deserialize errors remain)
- ✓ Fix generics emission for some types (partial — 122 generic param errors remain)
- ✓ Fix container/generic casing — FreeMonoid not free_monoid
- ✓ Fix callable error recovery — empty name, not "Callable" (Part 1)

Remaining — **approached via LintModel (Stream 6), not one-by-one patches:**

| Phase | Errors cleared | Approach |
|-------|---------------:|----------|
| **E1: Import derivation** | 316 | `LintModel.import_rules` + emitter TypeUsage tracking |
| **E2: Type well-formedness** | 183 | Primitive/algebraic/generic/callable rules from `types.dag` |
| **E3: Expr well-formedness** | 43 | Async propagation, service binding, brace escaping |
| **E4: Naming + formatting** | quality | Convention rules, rustfmt/black/gofmt compliance |
| **E5: Hermetic integration** | confirm | `cargo check` / `clippy` / `py_compile` as `#[test]` |
| **E6: Swappable + parity** | all targets | Rust + Python + Go via same LintModel mechanism |

After E1-E3 (589 → ~0):
7. Regenerate stage0, verify cargo check passes
8. Committed binary approach (never hand-edit generated code again)
9. Container sharing (FF-8, atomic with regen)

**Lane C (runs in parallel, no blocking deps):**
1. Decidability audit (review iteration primitives)
2. SyntaxSpec extraction (Stream 0 Step 1)
3. Wire RecursionPattern into complexity analyzer
4. L1 dissolution (continuous)

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
- **Guarantee enforcement:** Tier 3 machinery (complexity violations,
  L1 ratchet, emitted-Rust errors) must be gated before Phase 5 exit.
  No invariant may exist without a corresponding enforcement mechanism
  from the Guarantee Map.

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
