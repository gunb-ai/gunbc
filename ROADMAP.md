# gunbc Roadmap

## Thesis

### Compiler Substrate: Node and Edge

The compiler has two substrate primitives. Everything above them —
including truth values, type structure, and cardinality — is
compositional modeling in `.dag`.

| Primitive | What it is |
|-----------|-----------|
| **Node** | The universal carrier. Identity, structure, composition. |
| **Edge** | A directed relationship from one node to another. Edges are outgoing only — a node knows what it points to (children), never what points at it (parents). An edge either connects to a target node, or it doesn't exist. There is no "edge to nothing." |

Edges are directed: parent → child. A node sees its outgoing edges
(children) but not its incoming edges (who references it). This is the
D in DAG. Derived properties like fan-out (how many things reference a
binding) are computed by graph traversal, not stored on nodes. The
pipeline walks forward — parse, resolve, infer, emit all follow edges
in the outgoing direction.

### Why Two Substrate Primitives Are Sufficient

All possible states of a slot on a node:

| Edge exists? | Target node exists? | Valid? | What it is |
|---|---|---|---|
| Yes | Yes | Valid | Slot filled — relationship holds |
| Yes | No | **Invalid** | An edge must connect to something. Edge-to-nothing = no edge. |
| No | (N/A) | Valid | Slot empty — no relationship |

No third state. The binary distinction (exists / doesn't exist) is
inherent in both Node and Edge — no separate truth primitive needed.
This is why Bit and Classical logic are modeled as `.dag` types at
Layer 0, not as compiler primitives: node existence already carries
the binary, and `True | False` is a two-variant coproduct expressed
as edge connectivity patterns.

From Node and Edge, all structural properties emerge:

**Product (AND)** — a node where all child edges connect. "A Person
has a name AND an age" means both edges point to nodes.

**Coproduct (OR)** — a node where exactly one child edge connects.
"A Shape is Circle OR Square" means one edge points to a node.

**Cardinality** — an edge that exists or doesn't. Present = connects
to a node. Absent = no edge.

**Bit / Classical logic** — modeled in `.dag` as
`type Classical = True | False` (Layer 0 of the composition stack).
Users write boolean values and the language has full classical logic.
The compiler processes these structurally: a two-variant coproduct
where the truth value is carried by which edge is active.

### How This Looks in Practice

Every `.dag` type is a composition of Node + Edge:

```dag
// Product: a node where all child edges connect (AND)
type SourceSpan {
  file: FilePath        // edge to FilePath node — connected
  start: Int            // edge to Int node — connected
  end: Int              // edge to Int node — connected
}

// Coproduct: a node where exactly one child edge connects (OR)
// True and False are both nodes that exist — the distinction
// is WHICH edge is active (edge connectivity, not a truth primitive)
type Classical = True | False
type Bool = True | False

// Cardinality: an edge that may or may not connect
type AccessToken {
  token: Secret
  scheme: AuthScheme
  expires_at: Timestamp?          // edge connects or doesn't
}

// Recursive coproduct: edges can point back into the structure
type Stack<T>
  = Empty                         // terminal — no child edges
  | Push { top: T, rest: Stack<T> }  // product inside a coproduct variant

// Collection algebras: named compositions
type List<element> = FreeMonoid<element>
type Map<key, value> = PartialFunction<key, value>
```

The compiler sees nodes and edges — not names, not truth values, not
product/coproduct categories. `SourceSpan` and `AccessToken` are both
"node with three child edges" to the compiler. The structural
difference (all-connected vs one-connected) is an edge connectivity
pattern, not a compiler-known enum.

### Current Reality: Substrate + Bridges

Today the compiler has two substrate primitives and three bridges that
should dissolve into the modeled layers:

| What | Role | Status |
|------|------|--------|
| Node | Substrate | Keep |
| Edge (DAG) | Substrate | Keep |
| Conj / Disj | Bridge — `connective` enum on Node | Dissolve: 81 dispatch sites, 114 construction sites. Product/coproduct are edge connectivity patterns, not a compiler enum. |
| Cardinality | Bridge — `return_cardinality` enum on Node | Dissolve: 38 dispatch sites, 142 construction sites. An edge connects or it doesn't. |
| Bit / Bool | Modeled type (Layer 0) | Already correct — not a compiler primitive. Detected structurally as 2-variant coproduct (1 site). |

The bridges exist because the compiler was built incrementally. The
direction is to dissolve them: product/coproduct become edge
connectivity patterns the compiler reads from the graph.
Cardinality becomes edge existence. Bit is already modeled, not
compiler-known.

### Structural Principles

**1. Names are opaque namespaces.**

Type names (`Int`, `Map`, `List`) are human-readable labels for
structural compositions, not compiler-meaningful identifiers. The
compiler must not branch on node names for structural decisions.

**2. Compiler errors are orthogonal to the node graph.**

When inference fails, the result is not a node — it is a structurally
distinct failure. `InferredNode = Resolved { node } | CompilerError
{ message, span }`. Emit never sees error nodes.

**3. Syntactically distinct forms for the same operation normalize
before inference.**

The pipeline has a normalization boundary between resolve and infer.
After normalization: `Call`/`MethodCall` bridging is complete, nodes
carry declared structural properties, and parameterized types carry
their declared arity of children.

### Decidability

Every `.dag` program is decidable. Recursion, loops, and cyclic-looking
patterns are surface syntax sugar that decomposes into bounded iteration
over finite structure.

The language provides:
- `fold`, `map`, `filter`, `flat_map` — bounded by collection size
- `descend` — bounded by tree depth (structural descent)
- `repeat(bound: N)` — bounded by explicit count

The language does not provide `while(true)`, unbounded `loop`, or
unrestricted recursion. Undecidable programs are structurally
unrepresentable, not detected and rejected.

### Composition Stack

| Layer | What | Built from | Location |
|-------|------|-----------|----------|
| 0 | Classical logic, Bit | First modeled layer above substrate | `std/logic.dag`, `std/bit.dag` |
| 1 | Machine words (`Word32`, `Word64`) | Bit compositions | `std/bit.dag` |
| 2 | Named types (`Int`, `String`, `Char`) | Algebraic structures over machine words | `std/integer.dag`, `std/types.dag` |
| 3 | Collections (`List<A>`, `Set<A>`, `Map<K,V>`) | Algebraic structures with laws | `std/types.dag` |
| 4 | Structural compositions + bounded iteration | Nodes + edges + collection algebras | `std/iteration.dag` |
| 5 | Domain types | Compositions of Layers 0-4 | Compiler, user programs |

Every layer is built from Node + Edge. Product/coproduct are not a
layer — they are the composition mechanism itself: how nodes at any
layer combine through edges. Layer 0 (Bit, Classical logic) is the
first modeled composition above the substrate — the point where the
binary distinction inherent in node/edge existence gets a name.

See `docs/algebraic-type-spec.md` for the collection algebra and
denotational model.

### Guarantee Tiers

| Tier | Property | How enforced | Example |
|------|----------|-------------|---------|
| 1 | Structurally enforced | Violations don't compile | `ExpectedToken` enum — missing match arm = Rust error |
| 2 | Tested and gated | CI/test catches regressions | Scrambled-name tests — renamed types produce identical graph |
| 3 | Machinery exists, not gated | Report-only, dangerous | L1 ratchet script — runs but not in CI |
| 4 | Design exists, no machinery | Future | Parse-emit round-trip test |
| 5 | Fundamentally limited | Can't fully prove | Semantic correctness of emitted code |

All invariants should aspire to Tier 1 (unrepresentable violations).
Tier 3 items are the most dangerous — they give the illusion of coverage
without enforcement. Promoting Tier 3 to Tier 2 is always high-priority.

### End Goal

- Two substrate primitives: Node and Edge. Product/coproduct,
  cardinality, and truth are compositional modeling above the
  substrate, not compiler-known categories.
- Names are opaque. Inference processes graph structure only.
- Emit reads `LanguageSpec` + structural declarations, no hardcoded
  target-language knowledge.
- One shared emit walker drives all target languages.
- All `.dag` programs are decidable by construction.
- Ownership and complexity proofs wired into the pipeline.
- At least one real program compiles and runs end to end.

---

## Current State (2026-03-29)

### Dashboard

| Metric | Value | Target | Notes |
|--------|-------|--------|-------|
| .dag files | 87 | — | `dsl/` |
| Self-compile time | 6.47s | <30s | Release. Tokenize 4.87s dominates |
| Self-compile diagnostics | 0 | 0 | Green |
| Files emitted | 40 | — | Rust target |
| `full_dsl_compiles` | FAILS (1 diag) | 0 | `stack.dag` generic fn syntax |
| Bootstrap ratchet | 3 | 0 | `dag/syntax.dag` excluded (OOM) |
| L1 ratchet | 70 | 0 | 69 type constructors + 1 comparison |
| Complexity violations | 0 | 0 | Green |

### Known Invariant Violations

- **FF-8: Bare container clones are O(n).** Rust container templates
  produce `Vec<T>` / `HashMap<K,V>` instead of `Rc<Vec<T>>`. The
  emitter inserts `.clone()` on multi-use bindings — catastrophic in
  loops. Fix: change templates to shared representations. Root-caused
  2026-03-27, fix pending.
- **Option rendering is an emitter heuristic.** Absence-variant
  rendering should be a `LanguageSpec` declaration.
- **General recursion accepted.** `fn spin(n: n)` compiles. Fail-closed
  compilation (reject non-descending recursion) not yet implemented.

### Known Compiler Bugs

- **`uses` variables not bound in scope.** Compiler parses
  `uses fs: Filesystem` but never adds `fs` to `scope.locals`. Any
  body referencing `fs.read()` fails with "undefined variable."
- **Optional exhaustiveness hardcodes `Some`/`None`.** When matching on
  `T?`, the checker uses `vec!["Some", "None"]` as variant names. Inner
  type variants don't satisfy it.
- **Single-variant enums parsed as type aliases.** `type X = Y` where Y
  is a new variant name is treated as a type alias, not a one-variant
  enum definition.

---

## Milestones

### M1: Every .dag File Compiles

**What:** Every `.dag` file in the repo compiles as a unit with zero
diagnostics. No hardcoded file lists, no exceptions.

**Gate:** `cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored`
passes.

**Status:** 1 diagnostic remaining.

**Acceptance condition:** CLI, bootstrap, regeneration script, and all
tests use the same `--source-root` / transitive-import resolution path.
No parallel file list implementations.

**Work items:**
- [ ] Parser: support `fn foo<T>(...)` generic function syntax
  (`stack.dag` uses this; parser expects `(` but gets `<`)
- [ ] Verify no other .dag files break once stack.dag parses
- [ ] All test harnesses use `--source-root` discovery (no hardcoded
  file lists in test code)

---

### M2: Users Can Compile .dag to Working Rust

**What:** A user can write `.dag` files and get `cargo check`-clean Rust
output. The compiler produces correct, buildable code — not stubs.

**Gate:** `gunbc compile project/ --target rust && cargo check` passes
on regenerated stage0 and on a non-trivial user project.

**Depends on:** M1

**Work items:**

*Container sharing (FF-8):*
- [ ] Define `ContainerOps` type in `languages.dag` — rendering patterns
  derived from container templates, not hardcoded
- [ ] Change Rust container templates to shared representations
  (`Rc<Vec<{0}>>`, `Rc<HashMap<{0}, {1}>>`)
- [ ] Update `05_emit_rust.dag` and `runtime_rust.dag` for coherence
- [ ] Land atomically with stage0 regeneration

*Codegen correctness:*
- [ ] Primitive type lowering (`Bool` to `bool`, `Unit` to `()`)
- [ ] Algebraic types lower to stdlib (`FreeMonoid<T>` to `Vec<T>`)
- [ ] `Callable` type renders as `Rc<dyn Fn(...) -> T>`
- [ ] `async fn` emission for service operations
- [ ] Fix `uses` variable scoping (compiler bug)

*Bootstrap:*
- [ ] Regenerate stage0 with `regenerate-stage0.sh`
- [ ] Committed binary approach: stage0 source committed, CI verifies
  regenerate + diff = empty
- [ ] `dag/syntax.dag` inclusion without OOM

*User experience:*
- [ ] CLI: `gunbc compile --source-root ... --target rust` works for
  arbitrary user projects (CLI exists but untested on external input)
- [ ] Error messages: file:line:col with source context (infrastructure
  landed, needs polish for non-compiler-developer audience)

---

### M3: Discovery-Driven Test Generation

**What:** The compiler generates tests from `.dag` definitions. Every
`.dag` file is discovered automatically. Generated tests compile, run,
and are committed as derived artifacts (regenerate → diff → empty).

**Gate:** Generated Rust tests compile and pass. Test freshness gate
in CI. Every service operation with mock data has a generated test.

**Depends on:** M2 (working codegen baseline)

#### What the compiler generates (test taxonomy)

| .dag construct | Generated test | How it works | Status |
|---|---|---|---|
| **Service + `mock_response`** | Mock invocation test | Create service with `DryRunMode(true)`, call operation, assert `result.is_ok()` | Working — `emit_operation_test` generates async fn, 6 syntax tests pass |
| **Type (product/coproduct)** | Roundtrip test | Construct → serialize → deserialize → assert equal | Not yet implemented |
| **Pure function (`fn`)** | Property test | Type-driven input generation, assert no panics, assert return type | Not yet implemented |
| **Workflow (`func`)** | Dry-run test | All services use `mock_response`, assert workflow completes | Partially — DryRunMode wired, no full workflow tests |

#### What exists today

- `extract_test_projections` (`05_emit.dag`): finds all service
  operations with `mock_response` blocks, produces `TestProjection`
  with module, service, operation, params, and mock field inits
- `emit_operation_test` (`05_emit_rust.dag`): generates async test
  function per projection — creates service with DryRunMode, calls
  operation with default params, asserts ok
- `DryRunMode` infrastructure: parser reads `mock_response` blocks,
  emitter generates `if self.dry_run.is_dry_run() { return mock }`
  branches, service constructors accept dry-run flag
- Mock response data lives in `.dag` source alongside the service
  definition — the test and the spec are co-located
- 6 tests validate generated test syntax across Rust/Python/Go
- `full_dsl_compiles` discovers all `.dag` files by scanning `dsl/`

**The gap:** `compiler_tests.rs` is 27K lines of static test fixtures
checked into source — not actually generated by the compiler. The test
projection pipeline works but its output isn't committed or exercised.

#### Work items

*Infrastructure:*
- [ ] `full_dsl_compiles` promoted from `--ignored` to CI on every PR
- [ ] Replace static `compiler_tests.rs` with actual generated output
  from the test projection pipeline
- [ ] Test freshness gate: regenerate → diff → empty

*Test categories:*
- [ ] Service mock tests compile and pass (`cargo test` on generated
  module, not just syntax validation)
- [ ] Type roundtrip tests: generate construct/serialize/deserialize
  assertions for every type with `serde` support
- [ ] Workflow dry-run tests: end-to-end with all services mocked
- [ ] Coverage gate: every discovered service operation with
  `mock_response` has a generated test

*Cross-language:*
- [ ] Python generated tests pass `python3 -m py_compile`
- [ ] Go generated tests pass `go vet`
- [ ] Same test taxonomy across all three targets

---

### M4: Compiler Knows Zero Type Names (L1 = 0)

**What:** The compiler processes graph structure only. Names are opaque.
Inference cannot read them. Adding a type means editing `.dag` files,
not compiler code.

**Gate:** `scripts/l1-ratchet.sh --check` reports 0. Scrambled-name
tests pass (then are deleted — nothing left to scramble).

**Depends on:** M2 (stable bootstrap needed for iterating on compiler
internals)

**Work items:**

*D6: Delete Node.name (~553 sites across 20 files):*
- [ ] Decide name source: span derivation vs dedicated accessor
- [ ] Update 16 `make_*` helpers + 11 accessor functions
- [ ] Audit 60+ direct Node constructions in `02_parse.dag`
- [ ] Update 92 identity checks + 62 scope map operations
- [ ] Update emit + diagnostic layers to use `source_text_at(span)`
- [ ] Delete `Node.name` field, delete scrambled-name tests

*Method dispatch from .dag algebra:*
- [ ] Compiler reads methods from type algebra Nodes in `std/algebra.dag`
  (the definitions already exist — the compiler ignores them)
- [ ] Delete `intrinsic_method_index()` / `runtime_bridge_method_index()`
  string-to-enum maps
- [ ] P5.13 Part B: kernel types as algebraic compositions
  (`Int = OrderedRing<Word64>`, `Float = Field<Word64>`,
  `String = FreeMonoid<Char>`)
- [ ] Refactor `04_method.dag` / `04_infer.dag`: ~60 string branches to
  structural algebra queries

*Type constructor dissolution:*
- [ ] 69 type constructor ratchet sites to 0
- [ ] 1 type-name comparison to 0
- [ ] CollectionKind bridge dissolves when method algebras land

---

### M5: Emitted Code Correct by Construction

**What:** `LintModel` enforces emission correctness. `LanguageSpec`
carries all target-language facts. The emitter has zero hardcoded target
syntax.

**Gate:** No `match render_target` branches in emitter source. LintModel
validates every emitted file.

**Depends on:** M2 (working codegen baseline), M3 (generated tests
verify correctness)

**Work items:**

*LanguageSpec completion (~11 missing fields):*
- [ ] `statement_terminator`, `variable_declaration_keyword`
- [ ] `assignment_operator`, `lambda_syntax`
- [ ] `callable_type_template`, `error_expression`
- [ ] `null_coalesce`, `string_interpolation`
- [ ] `container_bracket`, `tuple_type_template`
- [ ] `indentation_width`

*LintModel wiring:*
- [ ] Wire import rules into emission (rules exist in
  `dsl/extdeps/languages/rust/lint.dag`)
- [ ] Wire naming conventions into emission (rules exist in
  `dsl/extdeps/languages/rust/naming.dag`)
- [ ] Wire formatting model into emission

*Backend parity:*
- [ ] Python backend: match syntax, statement/expression, async
- [ ] Go backend: implicit interfaces, error handling patterns
- [ ] Cross-language test generation (Go/Python emit stub assertions
  today)

*Compiler bug fixes:*
- [ ] Optional exhaustiveness: recognize inner type variants, not just
  `Some`/`None`
- [ ] Single-variant enum parsing: `type X = Y` defines a coproduct,
  not an alias

---

### M6: Parse-Emit Symmetry

**What:** The parser and emitter are symmetric views of the same
`LanguageSpec`. Adding a language means adding a spec file, not code.

**Gate:** `parse(spec, emit(spec, graph))` produces structurally
identical graph for all `.dag` files.

**Depends on:** M4 (name-opaque compiler), M5 (spec-driven emit)

**Work items:**
- [ ] Round-trip smoke test on `.dag` subset
- [ ] Statement dispatch (`parse_stmt`) spec-driven (3 keyword arms)
- [ ] Block/record disambiguation from heuristic to spec-driven
- [ ] Second language frontend (validates multi-frontend architecture)

---

### M7: Dissolve Structural Bridges

**What:** The compiler's remaining bridges (Conj/Disj, Cardinality)
dissolve into the substrate. The compiler reads edge connectivity
patterns from the graph instead of dispatching on enums.
`Int = Interpret<Signed, Word64>` — named types are compositions
over the substrate, not compiler-known concepts.

**Gate:** `connective` field removed from Node (81 dispatch + 114
construction sites). `Cardinality` enum removed (38 dispatch + 142
construction sites). `is_kernel_type` dissolved. No structural enums
remain — the compiler reads the graph.

**Depends on:** M6

**Work items:**
- [ ] Replace `connective: Conj/Disj` with edge connectivity model
  (product = all edges connect, coproduct = one edge connects)
- [ ] Dissolve `Cardinality` enum (edge connects or doesn't)
- [ ] Bit-graph representation for fixed-width types
- [ ] Full structural type algebra with denotational laws

---

## Design Directions

### Committed

**Decidability (LANDED).** Bounded primitives (`fold`, `descend`,
`repeat`) in `std/iteration.dag`. Complexity analyzer: 149 functions,
0 violations. `CostLog` for O(n log n). Structural descent detection
for catamorphisms.

**Compositional parser (LANDED).** `SyntaxSpec` in `languages.dag`.
`parse_item` has 0 keyword match arms. Operator precedence, item forms,
and literal keywords all spec-driven. Adding an item type = one entry
in `syntax.dag`.

**Node.name deletion (NEXT — D6).** Identity is the node itself. Text
derived from `source_text_at(span)`. Eliminates ~553 `.name` read
sites, the name registry concept, and scrambled-name tests. See M4.

**ContainerOps (DESIGNED).** Container rendering patterns derived from
templates, not hardcoded. `ContainerOps` type with fields for every
rendering pattern (`list_empty`, `list_iterate`, `map_wrap`, etc.).
Each emission site calls `container_empty_list(cops)` instead of
`concat("Rc::new(Vec::new())")`. See M2.

### Exploratory

**Unified Sequence (Seq\<T>).** Ordered collections (List, Stack, Queue,
Deque) share the same algebra (FreeMonoid). The access pattern
determines the representation: `push`/`pop` = cons list, `append`/`get`
= array, `enqueue`/`dequeue` = ring buffer. Mixed access is a type
error, not an optimization problem. Open questions: naming (`Seq` vs
`List`), `Bag<T>` as first-class type, iteration across algebras.

**Space complexity as peer dimension.** Time and space use the same
expression walk with different composition operators (sequential:
`add` vs `max`, parallel: `max` vs `add`). Currently `output_size`
is an unpopulated map. Promote to `space: CostExpr` peer to `work`
and `span`.

**Fail-closed compilation.** When descent analysis cannot lower a
recursive function to a bounded primitive, compilation should hard-fail.
Currently produces `DivideAndConquer` classification (soft). The
structural prevention (bounded primitives only) is the real guarantee;
fail-closed is the safety net during transition.

**Bridge dissolution.** Conj/Disj (81 dispatch sites) dissolve into
edge connectivity patterns. Product = all edges connect. Coproduct =
one edge connects. Cardinality (38 dispatch sites) dissolves — an edge
connects or it doesn't. The substrate already carries all structural
information; the bridge enums are redundant. See M7.

---

## Structural Guarantees and Proofs

### What the Compiler Proves Today

The compiler produces structural proofs alongside emitted code. These
run on every compilation — they are not separate analysis passes.

**Complexity analysis** (`complexity.dag`, 1475 lines):
- Computes `work` (sequential cost), `span` (parallel cost), and
  `output_size` per function as symbolic `CostExpr` trees
- Handles: literals, variables, binops, calls, method dispatch (via
  `method_cost_shape` mapping), match/if (max of branches), for-each
  (bounded by collection), recursion (classified + bounded)
- `CostExpr` variants: `CostConst`, `CostAdd`, `CostMul`, `CostMax`,
  `CostSum` (bounded summation), `CostLog` (logarithmic)
- Recursion classification: `LinearRecursion` (catamorphisms),
  `DivideAndConquer` (multiple self-calls), `UnresolvableRecursion`
- Structural descent detection (`is_structural_descent`): recognizes
  match/if over input where all self-calls are inside arms
- Self-compile: 1169 function summaries, 2 violations (both
  `DivideAndConquer` — cost algebra can't express exponentials)
- Certainty tracking: `Proven | Conservative | Unknown`

**Ownership analysis** (`ownership.dag`, 325 lines):
- Single-pass AST walk classifying each variable use as
  `Consumed | Read | Threaded | Projected`
- Branch-aware merging: match/if arms are mutually exclusive (MAX
  consumer count, not sum)
- Produces `OwnershipDecision` per binding: `SoleOwner` (can
  `into_inner`), `SharedError` (compile error), `Unclassified` (bug)
- **Status: fully implemented, NOT wired into pipeline or tests**

**Name invariance** (9 scrambled-name tests):
- Compile two structurally identical programs with different names
- Normalize both, assert byte-identical typed graphs and emitted source
- Covers inference (6 tests) and emission (3 tests: Rust/Python/Go)
- **Status: all pass, not ignored, run in CI**

### What's Proven vs What's Ratcheted

| Guarantee | Mechanism | Status | Gap |
|-----------|-----------|--------|-----|
| Syntax validity | Parser + tokenizer | Proven (every compilation) | None |
| Type soundness | Inference + reconcile | Proven (every compilation) | None |
| Name invariance | 9 scrambled-name tests | Tested (CI) | None |
| Complexity bounds | CostExpr per function | Ratcheted (2 violations of 1169) | Want 0; need exponential cost algebra |
| Decidability | Bounded primitives | Structural (language design) | Fail-closed not wired (general recursion still accepted) |
| Ownership | Full analysis exists | **Not wired** | Integrate into pipeline, add tests |
| Bootstrap stability | Fixed-point test | Tested (manual) | Promote to CI |
| Emitted Rust quality | cargo check + ratchet | Ratcheted (880 errors) | Want 0 (M2 work) |
| Performance | Wall-clock ratchet | Tested (manual, 30s) | Promote to CI |

### Ratchet Direction

Ratchets are checkpoints on the path to structural guarantees. Each
ratchet should trend toward its target value and eventually become
either a Tier 1 guarantee (structurally unrepresentable) or a Tier 2
guarantee (tested and gated in CI). A ratchet that stops moving is a
design signal — it means the current approach can't reach the target
and the machinery needs to change.

---

## Verification

### Ratchets

| Ratchet | Current | Target | Command |
|---------|---------|--------|---------|
| Self-compile diagnostics | 0 | 0 | `strict_compile_diagnostic_count -- --ignored` |
| full_dsl_compiles | 1 | 0 | `full_dsl_compiles -- --ignored` |
| L1 type knowledge | 70 | 0 | `scripts/l1-ratchet.sh --check` |
| Complexity violations | 2 | 0 | `strict_complexity_violation_count -- --ignored` |
| Emitted Rust errors | 880 | 0 | `bootstrap_stage0_to_stage1 -- --ignored` |
| Bootstrap fixed point | PASSES | PASSES | `v2_bootstrap_fixed_point -- --ignored` |
| Performance | <30s | <30s | `performance_ratchet -- --ignored` |

### CI Gates

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` | Every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | Every change |
| V2 compiler tests | `cargo test -p v2-compiler-tests` | Every change |
| Scrambled-name | `cargo test -p v2-compiler-tests v2_scrambled_name_inference` | Inference changes |

### Required Before Merge

Tier 3 ratchets that must pass before merging, until promoted to CI:

```
scripts/l1-ratchet.sh --check                                              # L1 ≤ ratchet value
cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored              # 0 diagnostics
cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored # 0 diagnostics
cargo test -p v2-compiler-tests v2_bootstrap_fixed_point -- --ignored       # stage0=stage1
```

### Non-Consensual Testing

If a `.dag` file exists in this repo, it is tested. No opt-in, no
hardcoded file lists, no exceptions. The test system discovers files by
scanning the filesystem, not by reading a manifest. `full_dsl_compiles`
is the gate: no PR merges if any `.dag` file fails to compile. See M3
for the full test generation milestone.
