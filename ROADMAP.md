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
| **L3: Syntax** | `kind_tag` string dispatch, hardcoded parser branches | How to parse surface syntax | **Active** — compositional parser (R3/Stream 0); item dispatch, operators, literals now spec-driven (PR #226) |

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

Layer 4: Structural compositions + bounded iteration
  descend, repeat, repeat_until                  (std/iteration.dag)
  Span<I>, Tree<A>, DAG<Id,A>, Annotated<A,F>   (structural types TBD)

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

### Implementation: bounded primitives approach

The decidability guarantee emerges from the language's modeling
primitives, not from a per-function checker. Three primitives in
`std/iteration.dag`:

| Primitive | Bound | Declared in |
|---|---|---|
| `fold` | \|collection\| | Methods on List, Set, Map (Layer 3) |
| `descend` | \|tree\| | `std/iteration.dag` (Layer 4) |
| `repeat(N)` | N (explicit) | `std/iteration.dag` (Layer 4) |

These are the ONLY iteration primitives. No `while`, no `loop`, no
general recursion. Recursive syntax desugars to these or is rejected.

**Step 1: Structural descent detection (compile-time).**

When the compiler encounters a recursive function, it analyzes the call
pattern and attempts to lower it to a bounded primitive:

```
Self-call on child of input       → descend (tree fold, O(|tree|))
Self-call inside fold body        → already bounded (fold-of-fold)
Self-call with n - 1              → repeat(n, ...) (O(n))
Mutual recursion on SCC children  → descend over SCC (O(|SCC|))
Self-call with unchanged argument → HARD ERROR (no bounded lowering)
```

This runs after inference (when self-calls are resolved), before
complexity analysis. It produces a `DescentProof` for each recursive
function — the proof that the function terminates.

**Step 2: Fail-closed compilation.**

If the descent analysis cannot lower a recursive function to a bounded
primitive, compilation fails with a hard error. This is the fail-closed
guarantee: programs that escape the structural analysis do not compile.

In a complete implementation, the hard error is unreachable — the
language has no primitive for unbounded recursion, so no program can
trigger it. The error exists as a safety net during the transition.

**Step 3: Complexity analyzer reads descent proofs.**

The complexity analyzer no longer needs to detect or classify recursion.
It reads the `DescentProof` and derives cost directly:

- `descend` proof → O(\|tree\| × per\_node\_cost)
- `repeat(N)` proof → O(N × per\_step\_cost)
- Fold-bounded → already handled by fold CostSum

The 2 current complexity violations (branching recursion in tree-walk
functions) resolve because the descent analysis recognizes that each
self-call processes a child of the input — tree traversal, O(n), not
branching recursion O(2^n).

**Step 4: Cost algebra upstream of language primitives.**

The cost algebra (`CostExpr`) is the upstream authority. A language
primitive cannot exist until its cost class is expressible in the
algebra. This inverts the current relationship (primitives exist →
algebra tries to catch up → Conservative gaps).

Current modeling deficit: `sort_by` exists without `CostLog`. The
algebra cannot express O(n log n), so `sort_by` gets Conservative
O(n). This is backwards — `CostLog` should have been added first.

Fix order (algebra leads, primitive follows):
1. Add `CostLog { base: Int, argument: SizeExpr }` to `CostExpr`
2. `sort_by` declares its cost as `CostMul { n, CostLog { 2, n } }`
3. Analyzer reads the declaration → O(n log n), Proven
4. `Conservative` certainty becomes a dead variant

The principle extends to all future primitives. Before adding any
primitive, answer: "what is its cost class?" If `CostExpr` can't
express it, grow the algebra first. The primitive follows.

| Primitive | Cost class | Algebra needed |
|---|---|---|
| `fold` | O(n × body) | `CostSum` (exists) |
| `descend` | O(\|tree\| × body) | `CostSum` (exists) |
| `repeat(N)` | O(N × body) | `CostSum` (exists) |
| `sort_by` | O(n log n × key) | `CostLog` (**missing**) |
| Future: binary search | O(log n) | `CostLog` (same) |

After `CostLog` lands, every current primitive has a tight bound.
`Conservative` certainty = 0 functions. `Unknown` = hard error.

**Step 5: `trace_pop_frame` O(n) → O(1).**

`trace_pop_frame` uses `take(count - 1)` which copies the list — O(n)
per pop. Replace with a proper `Stack` type in `std/` (cons list) that
has O(1) push/pop. The Stack is itself a bounded structure — it grows
by `push` (adds 1) and shrinks by `pop` (removes 1). Its maximum depth
is bounded by the program's call depth, which is bounded by `descend`'s
tree depth.

### Enforcement (see Guarantee Map)

This is a **Tier 1 (structural)** guarantee, not Tier 2 (tested).
The language makes undecidability unrepresentable because its iteration
primitives are bounded by construction. The complexity analyzer
*confirms* what the language already guarantees — it does not enforce
it. If the analyzer produces `?O(?)`, the bug is in the analyzer.

The fail-closed compilation step is a Tier 2 safety net during the
transition. Once the structural prevention is complete, the hard error
is unreachable and the fail-closed check becomes dead code.

### Non-goal

This does not restrict what users can express — only how they express
it. Recursive types, recursive functions, and long-running processes
are all supported. The language requires the bound to be explicit or
structurally derivable. "Run for 30 trillion years" is fine.
"Run forever" is not expressible.

### What "infinite" means in .dag

Most developers don't care about the difference between "infinite" and
"runs until the heat death of the universe." `.dag` hides this
complexity: `repeat(bound: max_int, ...)` runs for 2^63 - 1 iterations.
At 1 iteration per nanosecond, that's 292 years. At 1 per millisecond,
292 million years. The developer writes practical code. The compiler
gets total analysis. The bound exists even if no one ever reaches it.

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
| Emitted Rust compiles | Stage0 → `cargo check` | ~280 errors; no ratchet tracking reduction |

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

## Current State (2026-03-28)

**TOP PRIORITY: Structural decidability.** Decidability must be a
consequence of the language's modeling primitives, not a per-function
check. The language provides exactly three bounded iteration primitives
(`fold`, `descend`, `repeat`) — no unbounded loop, no general recursion.
Recursive syntax is sugar that the compiler lowers to bounded primitives
or rejects. See "Design Direction: Decidability" and `std/iteration.dag`.

**Current gap:** The compiler accepts general recursion without verifying
structural descent. `fn spin(n: Int) -> Int { spin(n: n) }` compiles.
Fix: fail-closed on non-descending recursion at compile time.
Complexity analyzer now has 0 violations on self-compile (structural
descent detection + CostLog landed).

**PRIORITY 2: Compiler thesis inversion (~362 hardcoded name-based
branches).** The compiler encodes knowledge as code branches instead of
reading data. Three themes: (A) type/method dispatch by string
comparison, (B) emission hardcodes target syntax instead of reading
LanguageSpec, (C) testgen doesn't test compiler. See "Compiler
structural audit" section below. **Fix direction: Boundary Sufficiency
(BS-1 through BS-4).** The data already exists (LanguageSpec is fully
populated, algebra products are correct); stages bypass it. See
"Design Direction: Boundary Sufficiency" for the 4-gap analysis and
execution phases.

**PRIORITY 3: Test generation (Theme C).** `generated_tests.rs` is a
static snapshot, not generated tests. Rust test emission works but
Go/Python emit stub assertions. No test verifies emitted code is
*correct*, only that it exists. Fix direction: golden-output tests
(compile `.dag` → `cargo check` → run generated tests), cross-language
test parity, and test generation as a compiler pipeline output.

**Diagnostic quality: LANDED.** Typed `CompilerDiagnostic` enum (14
variants), `ErrorNode` with causal DAG edges, `SourceSpan` carries
file path from tokenizer, `NewlineIndex` for O(log n) byte→line:col.
Renderer shows file:line:col, source context with caret spans,
topo-sorted output (roots first, cascades grouped). Old string-keyed
`diagnostic_node()` infrastructure fully deleted.

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
- Emitted Rust error ratchet: 872 errors (down from 1242)
- L1 ratchet: 51 (delegates to scripts/l1-ratchet.sh)
- Keyword arm count: 9 (exact match)
- Complexity ratchet: 2 violations (tree traversal misclassified as branching recursion — resolves when structural descent detection lands)

**Stage0 regeneration: BLOCKED on 872 codegen errors.** Fixes applied
(PR #229): serde removal, generic type params, container casing, type
map routing, callable error recovery. Remaining: callable rendering,
cross-module imports, cascading type mismatches.

**DSL compilation:** Added `\{` `\}` escape sequences. 775 brace
templates escaped across 18 DSL files. Remaining ~1 parse diagnostic
(block/record disambiguation). Compiler correctly fails closed.

**Diagnostic quality: LANDED.** `CompilerDiagnostic` typed enum,
`ErrorNode` with causal edges, file:line:col rendering, source
context with caret spans. Old `diagnostic_node()` deleted.

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
- **Structural decidability — TOP PRIORITY (bounded primitives + fail-closed)**
- **Compiler thesis inversion — PRIORITY 2 (~362 name-based branches)**
- **Test generation — PRIORITY 3 (golden output, cross-language parity)**
- **Stream 0 (compositional parser) — LANDED (PR #226)**
- **Guarantee enforcement (all Tier 3 → Tier 2) — Lane A done (PR #227)**
- **Diagnostic quality — LANDED (typed CompilerDiagnostic, ErrorDAG, file:line:col renderer)**

---

## Backlog: review.dag → Binary

Deferred pipeline. Not the current architectural priority (Stream 0
parse-emit symmetry takes precedence). When revisited: Stage 1 parser
(3 remaining diagnostics) → Stage 2 Rust codegen (~280 errors) →
Stage 3 domain wiring → Stage 4 feature parity with shell runtime.
See git history for full stage breakdown.

---

## Active Work (Phase 5)

| Stream | Branch | Focus | Exit criteria |
|--------|--------|-------|---------------|
| **Stream 0: Compositional Parser (R3)** | `cool-ant-90` | **LANDED (PR #226).** Spec-driven item dispatch, operators, literals. Next: parse-emit round-trip, second frontend. | ~~`parse_item` has 0 keyword match arms~~ **DONE**; adding item type = 0 parser edits **DONE**; round-trip test; second SyntaxSpec |
| **Stream 1: L1 Type Dissolution** | `l1-type-dissolution` | P5.7 predicates, P5.13 kernel decls, type constructors, type-name comparisons, CollectionKind bridge | L1 ratchet 420 → 0 |
| **Stream 2: Expression Model & Frontend** | *(unassigned)* | P5.1 token coherence, P5.5 residual enum cleanup, `assemble_stage0` fixups | Structural model maturity |
| **Stream 3: Container Sharing** | `perf/v2-tokenizer-root-cause` | Rust container templates → `Rc<Vec<{0}>>` etc. + emitter + runtime + stage0 regen | Eliminate O(n) clone class (FF-8) |
| **Stream 4: Guarantee Enforcement** | *(unassigned)* | Wire Tier 3 machinery into gates; add Tier 4 ratchets as design directions land | Complexity ratchet, L1 ratchet in CI, keyword arm ratchet, round-trip smoke test |
| **Stream 5: Compiler Correctness** | *(unassigned)* | Fix emitted Rust errors (~280→0); space complexity tracking; regeneration script | Generated stage0 passes `cargo check`; space bounds in complexity report |
| **Stream 6: Diagnostic Quality** | `warm-lynx-757` | **LANDED.** Typed CompilerDiagnostic (14 variants), ErrorNode/ErrorDAG, SourceSpan carries file, NewlineIndex, topo-sort renderer with source context | ~~Every error has file:line:col~~ **DONE**; ~~cascades trace to root~~ **DONE** |
| **Stream 7: Name Elimination** | *(unassigned)* | Resolution produces structural edges, not string-keyed scopes. Names consumed at resolution boundary, dead after. Inference/emit never read name strings. | L1 name-read ratchet = 0; scope maps keyed by Node ref, not String |

### Diagnostic Quality Design

**Problem.** The DSL is unusable until diagnostics are actionable.
Today: byte offsets, no file name, no line:column, no source context,
printed via `Debug`. The C++ problem: one root cause produces N
unrelated-looking errors because cascade suppression doesn't exist.

**Design principle: root cause by construction.**

Errors are a DAG. Each error is a node in that DAG. Root-cause errors
have no parents. Cascade errors carry explicit parent pointers — they
are structurally linked, not heuristically suppressed. The output is
the **minimal actionable set**: every error the user must independently
fix. Errors that would be eliminated by fixing a root cause are
cascades, not independent errors. Errors that share a category but
have different root causes are independent — all shown.

**Invariant review checklist (verified against INVARIANTS.md):**

| Invariant | How this design satisfies it |
|-----------|----------------------------|
| **Performance** | Newline index: O(n) build, O(log n) lookup. Error DAG topo-sort: O(V+E). Errors in Vec, id = index → O(1) causality lookup. No rescans. |
| **No duplicate representations** | Line:col computed at render time from `SourceSpan` + newline index — not stored. File path originates in tokenizer, flows forward in `SourceSpan`. One source of truth per fact. |
| **No case enumeration for open sets** | `CompilerDiagnostic` is a closed enum (compiler error categories are closed). Adding a variant is a compile error at all match sites. |
| **No fallbacks that fabricate** | If a stage can't determine causality, it emits an independent error — never guesses a parent. |
| **Explicit boundary contracts** | `ErrorDAG` is the boundary type between pipeline and renderer. Typed variants, not string-keyed properties. |
| **Facts flow forward** | File path enters at tokenization (where the file is known), carried in `SourceSpan` through every stage. Never reconstructed. |
| **Heuristics indicate lost structure** | Replaces string-keyed property extraction (`diagnostic_severity()` matching on `p.name == "severity"`) with typed variant fields. |

**Compositional error model.**

The current `diagnostic_node()` stuffs severity, module_name, and
category into string-keyed Node properties — the same anti-pattern
the reviewer flagged (string-keyed global maps). The fix: model
compiler errors the same way `std/errors.dag` models domain errors —
as typed compositions with required fields.

```
// SourceSpan gains a file field. File originates at tokenization
// (the only place that knows which file is being read) and flows
// forward through every pipeline stage. Line:col are NOT stored —
// they are derived at render time from the newline index.
type SourceSpan {
  file: FilePath
  start: Int
  end: Int
}

// Newline index — built once per source file, O(n). Byte offset →
// line:col is O(log n) binary search on the offset table.
// This is render-time infrastructure, not stored in the error.
type NewlineIndex {
  file: FilePath
  offsets: List<Int>        // byte offset of each newline
  source: String            // original source text (for context)
}

// Error variant — each variant carries its own required fields.
// You cannot construct an error without a span (which carries file).
// The set of variants is closed — adding one is a compile error
// at every match site.
type CompilerDiagnostic
  = UnresolvedName     { name: String, span: SourceSpan }
  | TypeMismatch       { expected: String, got: String, span: SourceSpan }
  | ArityMismatch      { name: String, expected: Int, got: Int, span: SourceSpan }
  | VariantNotFound    { variant: String, type_name: String, span: SourceSpan }
  | FieldNotFound      { field: String, type_name: String, span: SourceSpan }
  | NonExhaustiveMatch { missing: List<String>, span: SourceSpan }
  | CircularDependency { modules: List<String>, span: SourceSpan }
  | DuplicateModule    { name: String, span: SourceSpan }
  | MissingAnnotation  { fn_name: String, what: String, span: SourceSpan }
  | ParseError         { message: String, span: SourceSpan }

// Error DAG node — wraps a diagnostic with its causal parents.
// Errors stored in Vec<ErrorNode>; id = Vec index → O(1) lookup.
// caused_by is empty for root causes, non-empty for cascades.
type ErrorNode {
  diagnostic: CompilerDiagnostic
  caused_by: List<Int>         // indices into ErrorDAG.errors
  module_name: String
}

// Pipeline result carries the error DAG, not a flat list.
type ErrorDAG {
  errors: List<ErrorNode>      // id = position in list
}
```

**Propagation rule.** When a pipeline stage encounters an input that
is already a `CompilerError`, it emits a new `ErrorNode` with
`caused_by` pointing to the input error's index — not an independent
error. If causality is unknown, the error is emitted as a root cause
(empty `caused_by`) — never a guessed parent. The `cascade_error`
category is deleted; causality is structural.

**Display contract: minimal actionable set.**

The renderer computes which errors are independently actionable:
- **Root causes** (empty `caused_by`): always shown.
- **Cascade errors**: shown only if they require independent user
  action (e.g., a cascade that reveals a *second* problem beyond the
  root cause). Suppressed if fixing the root cause would eliminate
  them — determined structurally from the `caused_by` edges, not by
  heuristic.
- **Independent errors** that share a category but have different root
  causes: all shown. 50 independent `unresolved_name` errors from 50
  missing imports = 50 actionable items.

Display order is topological: roots first, then dependents grouped
under their root.

```
error[unresolved_name] src/v2/04_infer.dag:127:5
 │ unknown type `Foo`
 │
127 │   let x: Foo = bar()
 │          ^^^
 │
 ├─ error[type_mismatch] src/v2/05_emit.dag:84:12
 │   expected `Foo`, got compiler error
 │
 │  84 │   let result: Foo = process(x)
 │   │               ^^^
 │
 ╰─ error[field_not_found] src/v2/05_emit.dag:91:8
    no field `name` on compiler error

   91 │   x.name
    │     ^^^^

error[unresolved_name] src/v2/03_resolve.dag:42:3
 │ unknown module `std.missing`
 │
42 │   import std.missing { Thing }
 │          ^^^^^^^^^^^

2 root errors, 5 total (3 cascades from root #1)
```

**Performance contract.**

| Operation | Bound | Notes |
|-----------|-------|-------|
| Newline index construction | O(n) per file | Single scan of source bytes |
| Byte offset → line:col | O(log n) | Binary search on newline offsets |
| Error DAG construction | O(1) per error | Vec push + parent index |
| Causality lookup | O(1) | Vec index, not search |
| Topo-sort for display | O(V+E) | V = error count, E = causality edges |
| Source context extraction | O(1) per error | Newline index → line start/end offsets |

No operation is worse than linear in input size. No operation rescans
the full source or full error list.

**What this eliminates:**
- String-keyed property extraction (severity, module_name, category
  are typed fields, not string lookups).
- Cascade storms — one missing import no longer produces N independent
  errors. Cascades exist but are structurally linked to their root.
- Disconnected errors — every non-root error says exactly *which*
  root error caused it.
- Byte-offset-only output — `SourceSpan` carries file, renderer
  computes line:col from newline index.
- Stored derived data — line:col never stored, always computed.

**Implementation path (ordered by dependency):**

| Step | What | Status |
|------|------|--------|
| D1 | Add `file: FilePath` to `SourceSpan`, thread through tokenizer | **DONE** |
| D2 | `NewlineIndex`: per-file offset table, O(n) build, O(log n) lookup | **DONE** |
| D3 | `CompilerDiagnostic` (14 variants), `ErrorNode`, `ErrorDAG` | **DONE** |
| D4 | Replace 26 `diagnostic_node()` calls with typed constructors | **DONE** |
| D5 | Migrate pipeline from `List<Node>` to `List<ErrorNode>` (46 type decls) | **DONE** |
| D6 | Delete old `diagnostic_node()` + 9 accessor functions | **DONE** |
| D7 | Error DAG renderer: topo-sort, file:line:col, source context, carets | **DONE** |

All steps landed on branch `warm-lynx-757`.

### Stream 7: Target Compositional Model

**Problem.** The compiler has 45 types in `00_core.dag`. The thesis
says "everything is a Node." In practice, 10 satellite types
duplicate Node's name+span fields, ExprData carries 9 String
identifier payloads outside the graph, and names survive past
resolution because scope maps are string-keyed. The fragmentation
produces ~730 name/string operations across the codebase, of which
~260 are structural violations (the compiler branches on name
content).

**Root cause.** The compiler was built incrementally. Module, Import,
and ServiceConfig dissolved into Node. Field, Variant, Param,
ResourceUse, FieldInit, NamedArg, MatchArm, FieldBinding,
OperationDef, and CapabilityDef did not. The dissolution PATTERN
exists and works — it was not applied universally.

#### Target Model

The compiler needs exactly these types:

```
Token { text, span, shape }       -- tokenizer → parser boundary

Node {                             -- THE universal structural type
  span: SourceSpan                 -- identity + source text
  children: List<Node>             -- ALL structural composition
  connective: Connective           -- Conj / Disj / None
  collection_kind: CollectionKind  -- container shape
  params: List<Node>               -- parameters (NOT List<Param>)
  inferred: InferredNode?          -- resolved type (set by infer)
  return_cardinality: Cardinality  -- output multiplicity
  uses: List<Node>                 -- resource deps (NOT List<ResourceUse>)
  body: Node?                      -- computation body
  transport: Node?                 -- transport binding
  properties: List<Node>           -- metadata (NOT List<FieldInit>)
  type_annotation: Node?           -- explicit type
  is_self_recursive: Bool
  has_non_tail_self_call: Bool
  match_pattern: MatchPattern?     -- pattern discriminator
  expr_data: ExprData              -- expression operator (NO String fields)
}
```

**No `name` field. No `id` field. No name registry.**

A node IS its own identity (reference equality / structural
position). The source text at a node's span IS its "name" — derived
at render time via `source_text_at(span)`, never stored. The
`NewlineIndex` (already built per file) provides the source text.

- **Identity**: the node itself. Two nodes are the same iff they
  are the same node. No labels needed.
- **Rendering text**: `source[span.start..span.end]`. Derived from
  span, not stored. Emit extracts text when it needs to produce
  an identifier in the output.
- **Binding resolution**: edges from reference-site node to
  binding-site node. The binding site's span gives you the text
  when emit needs it.
- **Synthetic nodes** (compiler-generated, no source text): empty
  span. Emit handles them structurally (e.g., generated type
  names come from the type definition's structural position, not
  from a stored string).

This eliminates:
- `Node.name: String` (current — 730 read sites)
- `Node.id: NameId` (proposed intermediate — unnecessary)
- `NameRegistry` (unnecessary — span IS the registry)
- Scrambled-name tests (nothing to scramble)

```

ExprData                           -- pure operator tag
  = NoExprData
  | ExprLiteral { value: LiteralValue }
  | ExprError { kind: ExprErrorKind, message: String }
  | ExprVar { binding_kind: VarBindingKind? }
  | ExprFieldAccess { summary: FieldSummary? }
  | ExprCall { call_semantics: CallSemantics? }
  | ExprMethodCall { method_semantics: MethodSemantics? }
  | ExprMatch | ExprIf | ExprLet | ExprRecordLit
  | ExprListLit | ExprBinOp { op: BinOpKind }
  | ExprUnaryOp { op: UnaryOpKind }
  | ExprLambda { semantics: LambdaSemantics? }
  | ExprStringInterp | ExprBlock | ExprCast
  | ExprForEach | ExprIndex | ExprSlice | ExprReturn
```

**What changed from current:**

| Current | Target | Why |
|---------|--------|-----|
| `params: List<Param>` | `params: List<Node>` | Param dissolved — name+span are Node's |
| `uses: List<ResourceUse>` | `uses: List<Node>` | ResourceUse dissolved |
| `properties: List<FieldInit>` | `properties: List<Node>` | FieldInit dissolved |
| `ExprVar { name: String }` | `ExprVar` (unit) | Identifier is Node.name |
| `ExprFieldAccess { field: String }` | `ExprFieldAccess` | Field ref is children[1] |
| `ExprCall { func: String }` | `ExprCall` | Func ref is children[0] or Node.name |
| `ExprMethodCall { method: String }` | `ExprMethodCall` | Method ref is a child |
| `ExprLet { name: String }` | `ExprLet` | Binding name is Node.name |
| `ExprForEach { variable: String }` | `ExprForEach` | Loop var is Node.name |
| `ExprLambda { params: List<String> }` | `ExprLambda` | Param names are child Nodes |
| `ExprRecordLit { type_name: String? }` | `ExprRecordLit` | Type ref is a child |

**Deleted types (10):** Field, Variant, Param, ResourceUse, FieldInit,
NamedArg, MatchArm, FieldBinding, OperationDef, CapabilityDef. All
dissolve into Node following the existing Module/Import pattern.

**Retained structural enums:** Connective, Cardinality,
CollectionKind, ExprData (unit variants), MatchPattern, LiteralValue,
TokenShape, BinOpKind, UnaryOpKind, InferredNode, NodeType. These
are closed structural properties — not satellite types.

**Retained semantic enums:** IntrinsicMethod, RuntimeBridgeMethod,
MethodSemantics, CallSemantics, VarBindingKind, FieldSummary,
LambdaSemantics. These are populated by resolution/inference and
consumed by emit — they carry structural facts, not names.

#### What Each Stage Models

**Tokenize** (bytes → tokens):
- Input: raw source + filename
- Output: `List<Token>` — (text, span, shape) tuples
- Names: produces them from source text. Legitimate.

**Parse** (tokens → Node tree):
- Input: token sequence
- Output: Module Node with children (imports, items, expressions)
- Each identifier token becomes a child Node with a span. The
  span IS the name — `source_text_at(span)` recovers the text.
- Intermediates: **NONE in target model.** Parser produces Nodes
  directly. No Field/Variant/Param/OperationDef.

**Resolve** (Node tree → edges + ordering):
- Input: Nodes with spans, children, connective
- Reads: `source_text_at(node.span)` to match imports to modules,
  names to exports. This is the ONLY stage that reads text.
- Output: same Nodes + structural edges (reference → definition) +
  dep_order
- **Names die here.** Resolve reads source text, produces edges.
  Everything downstream follows edges.

**Infer** (edges → types):
- Input: resolved Node tree with structural edges
- Reads: Node.children, connective, edges (NEVER source text)
- Output: same Nodes + `Node.inferred` set on every expression +
  semantic enums populated (MethodSemantics, CallSemantics, etc.)
- Method dispatch via data tables loaded at resolution, not
  string comparison in inference.

**Emit** (typed graph → target code):
- Input: fully typed Nodes with semantics
- Reads: Node.inferred, expr_data semantics, children
- Output: TextFile (target-language source)
- Identifier text: `source_text_at(binding_site.span)` — follows
  edge to binding site, extracts text from source at that span.
  Never makes a rendering DECISION based on name content.

#### Audit Summary (2026-03-28)

~730 name/string operations. ~260 are violations.

**Sources (parser, ~80 embeddings):** `expect_name()` → String →
8 ExprData fields (29 sites) + 8 AST struct types (50+ sites).

**Violations by file:**

| File | Violations | Root cause |
|------|-----------|------------|
| `04_method.dag` | 56 (100%) | Method dispatch by string comparison |
| `05_emit_rust.dag` | 51 (37%) | Function dispatch, Rc wrapping, variant special-casing |
| `04_types.dag` | 22 (39%) | Type identity checks by name |
| `05_emit.dag` | 18 (53%) | Refined/Tuple handling |
| `04_infer.dag` | 17 (13%) | Special-case name comparisons |
| `00_core.dag` | 14 (100%) | `is_kernel_type`, transport/config dispatch |
| `04_access.dag` | 10 (100%) | Redundant with structural properties |
| `ownership.dag` | 6 | fold/init detection by name |

Go and Python emitters: 0 violations. They model correct emit.

**Legitimate name reads (~470):** scope lookup during resolution
(~120), structural copying (~125), emit rendering (~175),
diagnostics (~45).

#### Execution

| Phase | What | Dissolves | Ratchet |
|-------|------|-----------|---------|
| D1 | **Satellite type dissolution.** Param, ResourceUse, FieldInit → `List<Node>`. Delete types. | 3 types, ~85 consumer sites | Satellite types: 10 → 7 |
| D2 | **Parser produces Nodes directly.** `expect_name()` returns leaf Node. Field/Variant/OperationDef/CapabilityDef created as Nodes in parser, never as intermediates. | 4 types, ~80 parser sites | Satellite types: 7 → 3 |
| D3 | **ExprData String dissolution.** String fields → children. ExprData variants become unit. Parser stores identifiers as child Nodes. | 9 String fields, ~210 consumer sites | ExprData Strings: 9 → 0 |
| D4 | **Remaining dissolution.** NamedArg, MatchArm, FieldBinding → Node. MatchPattern names → Node.name. | 3 types, ~70 consumer sites | Satellite types: 3 → 0 |
| D5 | **Edge model.** Resolution produces structural edges (use → definition). Method dispatch via data tables. Inference walks edges. | ~120 scope lookups + ~165 name comparisons | Name reads in inference: current → 0 |
| D6 | **Delete Node.name.** Emit extracts identifier text via `source_text_at(binding_site.span)`. No stored name field. Delete scrambled-name tests (nothing to scramble). | ~175 emit name reads + `Node.name` field | Node has no name field |

D1-D4 are the type dissolution (existing pattern applied
universally). D5 is the architectural change (resolution boundary
kills names). D6 is the final step: Node.name ceases to exist.

The L1 ratchet (currently 52) tracks name-read sites in the compiler.
This model drives L1 to 0.

**On scrambled-name tests.** The current scrambled-name tests are
validation — they test that inference doesn't branch on names. The
target model makes this **structural**: Node has no name field.
Inference can't read the name because there is no name. The
scrambled-name tests become unnecessary by construction — delete
them when `Node.name` is removed. Until then, they remain as the
ratchet.

**Exit criteria:** 0 satellite types. ExprData has 0 String fields.
Node has no `name` field — identity is the node itself, text is
derived from span. Inference has no access to name text. Emit
extracts text via `source_text_at(span)`. Scrambled-name tests
deleted (invariant holds by construction). Diagnostic span precision
is automatic (every Node has a span).

### Stream 0: Compositional Parser — Status

**Goal:** The parser and emitter are symmetric. Both read from
spec data. Adding a language = adding a spec file, not code.

**Step 1: Extract .dag SyntaxSpec from hardcoded parser. — DONE (PR #226)**
`SyntaxSpec`, `ItemForm`, `BodyKind`, `OperatorSpec` types in
`languages.dag`. Instance data in `dsl/extdeps/languages/dag/syntax.dag`:
- `dag_item_forms`: 9 item forms (type, fn, func, service, resource, data, extern, pattern, interface)
- `dag_operators`: 16 operators with Pratt binding powers
- `dag_keyword_literals`: true/false/none/null → LiteralValue

**Step 2: Make parser generic over SyntaxSpec. — DONE (PR #226)**
`parse_item` reads from `dag_item_forms` via `find_item_form` lookup.
`parse_item_by_form` eats keyword generically, dispatches on `BodyKind`.
`infix_bp` reads from `dag_operators`. `parse_primary` reads from
`dag_keyword_literals`. 30+ `ShKw*` variants → single `ShKeyword`.
70+ keyword predicates deleted. 7 `*_after_kw` body parsers extracted.

**Step 3: Validate symmetry with emission. — NEXT**
Ensure `LanguageSpec` and `SyntaxSpec` share the same fact tables where
applicable (item tags, operator symbols, type templates). Define the
round-trip invariant test: `parse(spec, emit(spec, graph)) ≅ graph`.

**Step 4: DSL-side workarounds for current parser limitations.**
Fix the remaining 3 parse diagnostics in the DSL files (not in the parser):
- `filesystem.dag`: `and` → `&&`
- `auth/patterns.dag`: rewrite `{ token: value }` match arms
- `std/patterns.dag`: defer generics on patterns until spec-driven parser

**Step 5: Second language frontend.**
Define a second `SyntaxSpec` (e.g., a subset of Python or a simplified
frontend) to validate that the architecture supports multiple frontends.

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

---

## Workboard: Parallel Lanes

**Lane A (Verification) DONE (PR #227).** Ratchets are in place:
diagnostic (0), emitted Rust errors (1184/1200), L1 (51), keyword
arms (9), complexity (2). Space complexity (Lane A Phase 2) deferred
until Lane B drives error count down.

**Current priority: Lane B (Compiler Output).** The 1184 cargo check
errors in regenerated stage0 are the single deficit blocking
bootstrap regeneration and the committed-binary approach.

**Lanes B and C run in parallel.** They touch different files and have
independent exit criteria.

```
Lane A: Verification ✓           Lane B: Compiler Output       Lane C: Language Design
(DONE — PR #227)                 (make output correct)         (make the language right)
─────────────────────────        ─────────────────────────     ─────────────────────────
✓ Emitted Rust error ratchet     Stream 5A: Codegen fixes      Stream 0: Compositional
✓ Complexity ratchet (2)           (1184 → 0 errors)             Parser (R3)
✓ L1 ratchet (51, via script)    ┌─────────────────────┐       · SyntaxSpec extraction
✓ Keyword arm count (9)          │ E0425 (541): generic │       · parse_item reads spec
                                 │   type params <T>    │       · round-trip smoke test
Deferred:                        │ E0433 (404): serde   │
· Space as peer dimension        │   NonEmpty wrappers  │     Decidability invariant
                                 │ E0220 (140): derived │       · Audit iteration prims
                                 └─────────────────────┘       · Wire RecursionPattern
                                 Stream 5C: Regen script
                                   · ✓ cargo run (Docker)     Stream 1: L1 Dissolution
                                   · Regenerate + commit        · Type constructors → 0
                                                                · Type-name comparisons
                                 Stream 3: Container FF-8       · CollectionKind dissolves
                                   · Rc<Vec<{0}>> templates
                                   · Atomic with regen
─────────────────────────        ─────────────────────────     ─────────────────────────
Exit: ✓ Done                     Exit: cargo check 0 errors    Exit: parse_item 0 keyword
                                   on generated stage0;           arms; decidability is
                                   committed binary (not          structural (Tier 1);
                                   source); FF-8 eliminated       L1 ratchet = 0
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

Lane B ──→ Lane B:  Regen script (5C) unblocks emitted Rust fixes
                     (5A). 5C first, then 5A is automated.

Lane B ──→ Lane A:  Container sharing (FF-8) feeds space complexity
                     clone cost model. Space analyzer can land first;
                     FF-8 fix makes clone cost O(1) and analyzer
                     confirms.

Lane C ──→ Lane A:  Compositional parser enables round-trip smoke
                     test. Can start on subset before full parser.
```

### Execution order

**Lane A: ✓ DONE** (PR #227)

**Lane B (current priority — 996 errors → 0):**
1. ✓ Fix regen script for Docker (cargo run instead of binary path)
2. ✓ Remove serde from NonEmpty wrappers (138 → 0 serde errors)
3. ✓ Fix generics emission — `<T>` on struct/enum defs (~130 errors)
4. ✓ Fix container/generic casing — FreeMonoid not free_monoid
5. ✓ Fix callable error recovery — empty name, not "Callable" (Part 1)
6. Callable rendering (Part 2) — zero-param fn() → Rc<dyn Fn() -> T>.
   Structural identification works (inferred != none predicate) but
   rendering change causes ~186 cascading type mismatches. Needs full
   emit pipeline to agree: struct fields, function sigs, call sites.
7. Remaining E0425 (388): `Tuple`, `Bool` type names unresolved
8. Regenerate stage0, verify cargo check passes
9. Committed binary approach (never hand-edit generated code again)
10. Container sharing (FF-8, atomic with regen)

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

### Phase 5 Milestones (2026-03-28)

- **Diagnostic ratchet: 0.** Zero type errors or warnings on self-compile.
  (Note: stage0 regeneration currently blocked by pre-existing
  04_resolve.dag parse diagnostic unrelated to Stream 0.)
- **Fixed-point: PASSES.** Stage0 → stage1 → stage2 converges.
- **Self-compile: 6.47s** (release). Tokenize 4.87s, Parse 78ms,
  Resolve 1ms, Reconcile 244ms, Emit 1.27s.
- **Stream 0 landed (2026-03-28, PR #226).** `SyntaxSpec` types,
  data-driven item/operator/literal dispatch. 148/148 tests pass.

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
| R3 | `02_parse.dag` | D → B+ | **B+ (PR #226)** — spec-driven dispatch landed; statement/expression dispatch still hardcoded |
| R4 | `03_resolve.dag` | A → A | Done |
| R5 | `04_infer.dag` | D → B+ | Phase 1 |
| R6 | `05_emit*.dag` | D → B+ | Phase 4 (done) |
| R7 | `complexity.dag` | B → A | Phase 4 |
| R8 | `ownership.dag` | A- → A | Phase 3 |
| R9 | `compile.dag` | B- → A | Phase 3 |

### R3 Detail: Compositional Parser Model

**Landed (2026-03-28, PR #226).** The parser reads `SyntaxSpec` data.

**What shipped:**
- `SyntaxSpec` type with `ItemForm`, `BodyKind`, `OperatorSpec` in `languages.dag`
- `.dag` syntax spec instance: `dsl/extdeps/languages/dag/syntax.dag`
- `parse_item` → `find_item_form` lookup → `parse_item_by_form` dispatch on `BodyKind`
- `infix_bp` → `find_operator_bp` lookup from `dag_operators` table
- `parse_primary` literal dispatch → `dag_keyword_literals` map lookup
- `TokenShape`: 30+ `ShKw*` → single `ShKeyword`; 70+ predicates deleted
- 7 `*_after_kw` body parsers extracted from existing parse functions

**Exit criteria (met):**
- `parse_item` has 0 keyword-specific match arms
- Adding a new item type with standard body = 1 file edit (`syntax.dag`)
- `infix_bp` has 0 hardcoded match arms

**Remaining (deferred):**
- Statement dispatch (`parse_stmt`) — 3 keyword arms, stable, small
- Expression keyword dispatch (match/if/for/let/return/fn) — structural forms, not data-drivable
- Block/record disambiguation — still heuristic
- Parse-emit round-trip test
- Second language frontend

---

## Backlog

### Language Features

| Item | Why deferred |
|------|--------------|
| Full linear type checking | Ownership proof started; full proof beyond current migration |
| `[when]` string comparison | Only boolean fields supported; blocks conditional service dispatch |
| `[when]`/`[after]` inside `for` | Bracket clauses only on top-level step bindings |
| Multiple `uses` clauses | Only one per `func`; workaround: use `shell.Exec.Run` |
| `fixture`/`test` blocks | **Unblocked by PR #226** — add `ItemForm` entry in `syntax.dag` with `BlockBody` |

### Desired Parser Features (2026-03-28)

| Feature | DSL workaround | Proper fix |
|---------|----------------|-----------|
| `uses Resource(mode: X)` parameterized resources | Drop `uses` clause | Compositional parser: `uses` accepts arbitrary config |
| `[after X, when X.field]` multi-clause brackets | Implicit data-flow ordering | Bracket clause accepts comma-separated constraints |
| `[when]` string comparison `x == "foo"` | `match` + `shell.Exec.Run` | Bracket clause accepts arbitrary boolean expressions |
| Multiple `uses` clauses per func | `shell.Exec.Run` for secondary resources | `uses net: Network, fs: Filesystem` |
| `fixture`/`test` blocks | Comment out; tests via cargo test | **Unblocked:** add `ItemForm` to `syntax.dag` |
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
