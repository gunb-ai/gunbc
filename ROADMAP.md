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
Classical logic (`True | False`) is the first modeled layer above
the substrate: the point where the binary distinction inherent in
node/edge existence gets a formal name. Bit is classical logic given
a hardware name. Everything else composes upward from there.

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

### Where We Are (current compiler primitives)

| What | How the compiler knows it | Sites | Status |
|------|--------------------------|-------|--------|
| **Node** | Substrate type | — | Keep |
| **Edge (DAG)** | Substrate (outgoing children) | — | Keep |
| **Conj / Disj** | `connective` enum on Node | 81 dispatch, 114 construction | Bridge — dissolve into edge patterns |
| **Cardinality** | `return_cardinality` enum on Node | 38 dispatch, 142 construction | Bridge — dissolve into edge existence |
| **Bool** | `kernel_types` string list, fabricated `bool_type` node, `"Bool"` → `BooleanAlgebraProfile` map, `type_name == "Bool"` in emit | 5 name-based sites, 1 structural | **Not dissolved** — compiler knows Bool by name |
| **Int, String, Float** | Same as Bool: `kernel_types` list, type algebra profile map, emit default values | ~20 name-based sites each | **Not dissolved** — compiler knows these by name |
| **List, Map, Set** | `container_types` string list, `is_container_type()` | ~10 name-based sites | **Not dissolved** — compiler knows containers by name |

The compiler currently has **2 substrate primitives + 2 bridge enums +
~8 name-known types**. The direction is to dissolve everything except
Node and Edge.

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

### Languages as Coercion Targets

The compiler core does not know about Rust, Go, Python, SPICE, or
Verilog. It does not know about any specific target. Rendering is
**coercion** — finding the minimal complete representation of each
graph segment in the target's native capabilities.

This is the same problem as compiling floating point on a target
with no FPU:
- Target has native FPU → use it (efficient, zero overhead)
- Target has no FPU → synthesize from integer ops (correct, expensive)
- The compiler picks the minimal representation automatically

For graph patterns:
- Target has native branching (Rust `match`, Verilog `case`) → use it
- Target has no branching (pure analog SPICE) → synthesize from mux/
  comparator circuits (correct, more components)
- The cost algebra reports the difference

**The rendering model:**

A language plugin declares two things:
1. **Native capabilities** — what the target does efficiently
2. **Rendering table** — how each structural pattern maps to target
   constructs

The compiler walks the typed graph. For each node, it matches the
structural pattern (edge count + which edges connect) and looks up
the target's rendering. If the target has a native mapping → use it.
If not → synthesize from more primitive capabilities (and the cost
algebra reflects the overhead). If not even synthesizable → compile
error.

| Pattern (Node + Edge) | Rust (native) | SPICE analog | Verilog | English |
|---|---|---|---|---|
| Product (all edges) | struct | subcircuit ports | module ports | "X has Y and Z" |
| Coproduct (one edge) | enum/match | **mux** (synthesized) | case/mux | "X is Y or Z" |
| Cardinality | Option | **tri-state** | tri-state | "optionally" |
| Sequence | let bindings | wire chain | assign chain | "first X, then Y" |
| Function | fn | subcircuit | module | "given X, produce Y" |

**Any causal system qualifies.** The `.dag` graph is directed
relationships between entities — the same structure as circuits,
HDL, natural language, and serialization formats. The minimum: can
you model a directed connection between two things? If yes, you can
render a `.dag` program. The quality varies by how many patterns the
target handles natively vs synthesizes.

**Current reality:** 6,857 lines of language-specific code inside
`src/v2/` and 632 mentions of specific language names across 12
compiler files. These should all be zero.

**Challenge targets** (design validation — if the rendering model
works for these, it works for anything):
- **Verilog** — hardware: products are module ports, coproducts are
  muxes, sequences are assign chains
- **SPICE** — analog: products are subcircuit parameters, coproducts
  are synthesized from comparators + switches, cardinality is tri-state
- **English (Markdown)** — natural language: products are bullet lists,
  coproducts are "either/or", functions are paragraphs

Adding a language = writing `coerce.dag` (coercion rules declaring
the target's basis) + `render.dag` (trivial text output from
target-basis graph) in `dsl/extdeps/languages/`. Zero compiler
changes.

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

The foundation is formal logic, not hardware. Each layer builds on
the one below through composition. Everything reduces to classical
truth — the binary distinction inherent in node/edge existence.

| Layer | What | Built from | Example |
|-------|------|-----------|---------|
| 0 | **Classical logic** (`True \| False`) | Substrate (node existence = truth) | `std/logic.dag` |
| 1 | **Bit** | Classical logic given a hardware name | `std/bit.dag` — `type Bit = Classical` |
| 2 | **Machine words** (`Word32`, `Word64`) | Compositions of Bits | `std/bit.dag` — `type Word64 = Tuple<Bit, ..., Bit>` |
| 3 | **Algebraic types** (`Int`, `String`, `Float`) | Algebraic structures over machine words | `Int = OrderedRing<Word64>` |
| 4 | **Collections** (`List<A>`, `Set<A>`, `Map<K,V>`) | Algebraic structures with laws | `List = FreeMonoid` |
| 5 | **Structural compositions** + bounded iteration | Nodes + edges + algebras | `std/iteration.dag` |
| 6 | **Domain types** | Compositions of Layers 0-5 | Compiler, user programs |

Product/coproduct are not a layer — they are the composition
mechanism: how nodes at any layer combine through edges.

### Type coercion through the stack

Every type must reduce to classical truth through composition.
This is how new types coerce into target languages — the language
doesn't need to know what the type IS, only how to render the
structural patterns it decomposes into.

```
Bloobear                          -- user-defined Layer 6 type
  = Product { x: Wobble, y: Int } -- decomposes to product of fields
    → Wobble decomposes to...     -- each field decomposes further
    → Int = OrderedRing<Word64>   -- Layer 3: algebraic structure
      → Word64 = Tuple<Bit×64>   -- Layer 2: machine word
        → Bit = Classical         -- Layer 1: hardware name
          → True | False          -- Layer 0: classical truth
            → node exists or not  -- substrate
```

The target language renders each structural pattern it encounters
during decomposition: product → struct/subcircuit/bullet-list,
coproduct → enum/mux/"either-or", leaf → literal value. The language
never needs to know what "Bloobear" means — it renders the structure.

**If a new type CAN'T reduce to classical truth** (e.g., fuzzy logic
with truth values between 0 and 1, or quantum superposition), that's
a new Layer 0 — a genuinely different logical foundation. This is a
thesis-level change: the substrate may need to grow, and every
language plugin needs new rendering entries. This should be
extraordinarily rare.

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

### Where We Will Be (target state)

| What | How the compiler knows it | Sites |
|------|--------------------------|-------|
| **Node** | Substrate type | — |
| **Edge (DAG)** | Substrate (outgoing children) | — |
| **Everything else** | `.dag` declarations the compiler reads structurally | 0 name-based sites |

- Product/coproduct, cardinality, and truth are compositional modeling
  above the substrate — edge connectivity patterns, not compiler enums.
- Names are opaque. Inference processes graph structure only.
- Zero language-specific code in the compiler core. Languages are
  coercion targets: `coerce.dag` (rules) + `render.dag` (trivial text)
  in `dsl/extdeps/languages/`. The compiler coerces the typed graph
  into the target's basis; the renderer produces text from the
  already-coerced graph.
- All `.dag` programs are decidable by construction.
- Ownership and complexity proofs wired into the pipeline.
- At least one real program compiles and runs end to end.

### Adding a new base concept

If the architecture is right, adding a new fundamental modeled concept
(a new Layer 0/1 type, beyond Bit/Bool) requires:

- **Add a `.dag` file to `std/`.** Define the type as a composition
  of the substrate primitives (Node + Edge).
- **Zero compiler changes.** The compiler reads structural properties
  from the `.dag` declaration. It doesn't know the concept by name.

If adding the concept requires compiler changes, the architecture has
failed — a structural fact is missing from the substrate.

**Test:** try to model the new concept as a `.dag` type. If you can
→ add the file, done. If you can't → the substrate needs to grow,
which is a thesis-level event requiring deep design review.

Examples: `type Probability = Float where range(0.0, 1.0)` (refinement,
zero compiler changes). `type Qubit = Superposition<Bit>` (new
constructor — would need substrate discussion). `type Signal =
Continuous<Float>` (new collection kind — would need substrate
discussion). The substrate should rarely grow; most concepts compose.

---

## Current State (2026-03-29)

### Dashboard

| Metric | Value | Target | Notes |
|--------|-------|--------|-------|
| .dag files | 87 | — | `dsl/` |
| Self-compile time | 6.47s | <30s | Release. Tokenize 4.87s dominates |
| Self-compile diagnostics | 0 | 0 | Green (pipeline reports 0; bootstrap ratchet allows 3) |
| Files emitted | 40 | — | Rust target |
| `full_dsl_compiles` | FAILS (1 diag) | 0 | `stack.dag` generic fn syntax |
| Bootstrap ratchet (`DIAG_RATCHET`) | 3 | 0 | `dag/syntax.dag` excluded (OOM) |
| L1 ratchet | 70 | 0 | 69 type constructors + 1 comparison |
| Complexity violations (`COMPLEXITY_RATCHET`) | 2 | 0 | 2 DivideAndConquer functions |

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
No parallel file list implementations. Known gap: bootstrap
`prepare_sources` in test harness still uses hardcoded file assembly.

**Work items:**
- [ ] Parser: support `fn foo<T>(...)` generic function syntax
  (`stack.dag` uses this; parser expects `(` but gets `<`)
- [ ] Verify no other .dag files break once stack.dag parses
- [ ] Unify source discovery: `full_dsl_compiles`, bootstrap
  `prepare_sources`, and `strict_complexity_violation_count` all use
  `--source-root` discovery. Currently 4 parallel file-list
  implementations (CLI, full_dsl_compiles, bootstrap, complexity).
- [ ] Add regression tests BEFORE M4/M5 work:
  - Generic fn: `fn foo<T>(x: T) -> T` parses and compiles
  - Single-variant enum: `type X = Y` defines coproduct, not alias
  - `uses` binding: body can reference `fs.read()` from `uses fs`

---

### M2: Users Can Compile .dag to Working Rust

**What:** A user can write `.dag` files and get `cargo check`-clean Rust
output. The compiler produces correct, buildable code — not stubs.

**Gate:** `gunbc compile project/ --target rust && cargo check` passes
on regenerated stage0 and on a non-trivial user project.

**Depends on:** M1

**Work items:**

*Container sharing (FF-8):*
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
- [ ] Stage0 regeneration is automated and CI-verified (regenerate +
  diff = empty). Stage0 .rs files are derived artifacts, never
  hand-edited — the `.dag` source is the single authority.
- [ ] `dag/syntax.dag` inclusion without OOM

*User experience:*
- [ ] CLI: `gunbc compile --source-root ... --target rust` works for
  arbitrary user projects (CLI exists but untested on external input)
- [ ] Error messages: file:line:col with source context (infrastructure
  landed, needs polish for non-compiler-developer audience)

---

### M3: Discovery-Driven Test Generation and Guarantee Receipt

**What:** The compiler generates tests from `.dag` definitions and emits
a machine-readable guarantee receipt every run. Every `.dag` file is
discovered automatically. Generated tests compile, run, and are
committed as derived artifacts (regenerate → diff → empty). The receipt
is the single authority for what is proven, tested, and uncertain.

**Gate:** Guarantee receipt emitted on every compilation. Generated Rust
tests compile and pass. Test freshness in CI. Every service operation
with mock data has a generated test.

**Depends on:** M2 (working codegen baseline)

#### Guarantee receipt

The compiler emits a JSON receipt on every run — the single authority
for what the compilation proved, tested, and left uncertain. Markdown
dashboards are generated FROM the receipt; they are never the source
of truth. If a guarantee is not in the receipt, it does not exist.

```json
{
  "source_digest": "...",
  "compiler_digest": "...",
  "target": "rust",
  "discovered": {
    "dag_files": 87, "services": 42,
    "workflows": 9, "pure_functions": 149
  },
  "structural": {
    "decidability": "proven",
    "name_opacity": "ratcheting:70",
    "parse_item_keyword_arms": 0
  },
  "gated": {
    "all_dsl_files_parse": "pass",
    "full_dsl_compiles": "fail:1",
    "generated_rust_tests": "pass",
    "edge_contract_coverage": { "covered": 812, "uncovered": 4 }
  },
  "report_only": {
    "ownership_coverage": "61/149",
    "emitted_rust_errors": 880
  }
}
```

#### Test generation tracks

**Track 1 — Discovery gates:**
- `all_dsl_files_parse`, `full_dsl_compiles`
- Emitted Rust/Go/Python syntax or compile checks
- Test freshness: regenerate → diff → empty

**Track 2 — Behavioral tests by construct:**

| .dag construct | Generated test | How it works | Status |
|---|---|---|---|
| **Service + `mock_response`** | Mock invocation test | DryRunMode, call operation, assert ok | Working — 6 syntax tests pass |
| **Type (product/coproduct)** | Roundtrip test | Construct → serialize → deserialize → assert equal | Not yet |
| **Pure function (`fn`)** | Property test | Type-driven input gen, assert no panics | Not yet |
| **Workflow (`func`)** | Dry-run test | All services mocked, assert completes | Partial |

**Track 3 — Edge-contract coverage:**
For every edge in every compiled DAG, generate a producer→consumer
harness that executes with synthesized witness values and asserts port
cardinality, coercion, shape compatibility, and error behavior. For
joins/splits/guards, generate cross-products across adjacent ports and
branch outcomes. This is a natural extension of DryRun — wiring,
cardinality, coercion, guards, branching, topological ordering.

**Track 4 — Execution tiers:**
- Tier 1 DryRun: graph wiring, cardinality, coercion, ordering
- Tier 2 Selective Real: hermetic value correctness
- Tier 3 Full Real: controlled integration only

**Track 5 — Differential/parity:**
Same `.dag`, same mocks, same assertion shape across Rust/Go/Python.

#### Obligation-driven test selection

The generator collects proof obligations per construct, discharges
obligations the compiler already proves structurally (type
compatibility, cardinality, acyclicity), and generates tests only for
undischarged obligations. This avoids tautological testing — don't
re-test what the compiler already guarantees by construction.

#### What exists today

- `extract_test_projections` + `emit_operation_test` + `DryRunMode`:
  working pipeline for service mock tests, 6 syntax tests pass
- Name invariance: 9 scrambled-name tests (6 inference + 3 emit)
  covering Rust/Python/Go — all pass in CI
- Parse/emit round-trip smoke test
- Python/Go syntax validation (ast.parse, go vet structure check)
- Ownership analysis wired into pipeline, verified by tests
- Artifact planning wired into pipeline, verified by tests
- `compiler_tests.rs`: reads .dag from disk (no embedded source),
  16 test functions covering tokenize/parse/compile/profile
- `full_dsl_compiles`: discovers all .dag files by scanning `dsl/`
- 184 tests pass, 9 ignored (expensive bootstrap/performance tests)

The current verification surface is broader than "a few syntax tests."
The missing piece is not more ad-hoc tests — it's making the
receipt/status layer authoritative so the roadmap doesn't drift from
reality.

#### Work items

*Guarantee receipt:*
- [ ] Define receipt schema as `.dag` type
- [ ] Compiler emits receipt on every `compile_sources` call
- [ ] CI validates receipt fields against ratchet values

*Discovery gates:*
- [ ] `full_dsl_compiles` promoted from `--ignored` to CI
- [ ] Test freshness gate: regenerate → diff → empty

*Behavioral tests:*
- [ ] Service mock tests compile and pass (not just syntax)
- [ ] Type roundtrip tests
- [ ] Workflow dry-run tests
- [ ] Edge-contract coverage harnesses

*Ratchet promotion (Tier 3 → Tier 2):*
- [ ] Complexity violations in CI (currently 2, target 0)
- [ ] Emitted Rust errors in CI (currently 880, target 0)
- [ ] Ownership coverage in CI (currently not tracked)

*Cross-language:*
- [ ] Python tests pass `python3 -m py_compile`
- [ ] Go tests pass `go vet`
- [ ] Same taxonomy across all three targets

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
`Int = OrderedRing<Word64>` — named types are compositions
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

**Decidability (primitives LANDED, fail-closed NOT YET WIRED).**
Bounded primitives (`fold`, `descend`, `repeat`) in `std/iteration.dag`.
Complexity analyzer: structural descent detection, `CostLog` for
O(n log n). Known gap: `fn spin(n: n)` still compiles — fail-closed
compilation (reject non-descending recursion) is designed but not
wired into the pipeline as a hard error.

**Compositional parser (LANDED).** `SyntaxSpec` in `languages.dag`.
`parse_item` has 0 keyword match arms. Operator precedence, item forms,
and literal keywords all spec-driven. Adding an item type = one entry
in `syntax.dag`.

**Node.name deletion (NEXT — D6).** Identity is the node itself. Text
derived from `source_text_at(span)`. Eliminates ~553 `.name` read
sites, the name registry concept, and scrambled-name tests. See M4.

**ContainerOps (SUPERSEDED by rendering table model).** Was a
stepping stone: type with fields for each container rendering pattern.
The language-as-coercion-target model subsumes it — container rendering
is just one entry in the language's rendering table, not a separate
type. Immediate M2 fix: change Rust container templates to shared
representations (`Rc<Vec<{0}>>`) in `LanguageSpec`.

### Structural Prevention of Invariant Violations

#### The pipeline law

A `String` is legitimate when it carries text. A `String` is wrong
when it chooses behavior. The rule, stage by stage:

| Stage | String rule |
|---|---|
| **Before resolve** | Strings are allowed as source payload (token text, identifiers, keywords) |
| **At resolve** | Strings are consumed to produce edges. Scope maps are resolver-local and die here. |
| **After resolve** | **No semantic decision may depend on free text.** Anything that changes behavior must be an edge/reference, a closed enum, or a typed boundary fact. |
| **In emit** | Only the renderer may produce strings. Shared emit walks the graph and invokes the renderer — it never returns `String`. |

This is the existing invariants restated as an API law: names are
opaque, boundaries must be sufficient, heuristics mean a fact was lost.

#### What's legitimate as String

- Token.text, file paths, module paths (source payload)
- Identifiers during parse, before resolution (frontier artifact)
- Diagnostic messages (human-facing payload)
- TextFile.content and LanguageSpec templates (final rendered text)
- Resolver-local scope maps that die at the resolution boundary

These are text. They don't choose behavior.

#### What must stop being String

| What | Current | Target | Sites |
|---|---|---|---|
| `Node.name` | String field, read everywhere | Deleted. Identity = the node. Text from `source_text_at(span)`. | 1,175 reads |
| `MethodSemantics.method_name` | String, re-dispatched in infer/complexity/emit | Edge to algebra method node in `std/algebra.dag` | 63 dispatch sites |
| `VarBindingKind.parent_enum` | String | Edge to parent type node | Scattered |
| `MatchPattern.Bind.name` | String | Child node with span | Scattered |
| `kernel_types` / `container_types` | `List<String>` | `List<Node>` — edges to definitions | 2 lists |
| `variant_to_enum`, `field_type_names` | `Map<String, String>` | Edges on type definition nodes | 4 maps |
| `builtin_function_registry` | `Map<String, Node>` | Loaded from algebra `.dag` declarations | 30 entries |
| Shared emit return type | `String` (680 concat calls) | Graph walker → renderer (no string return) | 680 sites |
| Transport dispatch | `transport.name == "rest"` | Closed enum on transport node | 13 sites |
| Optional hardcoding | `variant_name == "Some"` | Structural Optional with known layout | 6 sites |

Not every bad String becomes a Node. Some become edges to nodes, some
become closed enums, some become renderer-local spec text. The question
is never "string or DAG?" — it's "does this string carry text, or does
it choose behavior?"

#### Three bypasses to defend against

Even after cleanup, these can reopen the escape hatches:

**1. `source_text_at(span)` as a semantic API.** Deleting `Node.name`
is not enough if infer/complexity/ownership re-read text from spans.
After resolve, source text must be a privileged capability available
only to renderers and diagnostics — not to semantic stages.

**2. String return types on emit functions.** The emitter must not
return `String`. It returns the typed graph; the renderer produces
text. No intermediate string-producing IR is needed because the
compiler's output IS the graph.

**3. "Temporary" stringly side tables.** `Map<String, X>` looks
harmless, but once it crosses a stage boundary it becomes a second
authority. The invariants say: speculative or lossy boundary fact
tables should be deleted rather than carried forward. "Temporary"
without a ratchet means "permanent later."

#### Backend model: emergent graph coercion

Coercion is not a rule engine. It is **emergent from graph structure.**
The compiler reads the source graph and the target's basis, and the
coercion direction falls out of the subgraph relationship — no
hand-written rules for things the graph already tells us.

```
typed graph → structural comparison → coercion direction → target-basis graph → renderer
```

**Three coercion directions, two are structural:**

| Direction | Graph relationship | Cost | Example |
|---|---|---|---|
| **Upcast** (widen) | Source is subgraph of target | **Free** | `Url → String` — Url has every String edge plus more. Forget the extras. |
| **Downcast** (narrow) | Target is subgraph of source | **Needs check** | `String → Url` — target has constraints source doesn't. Runtime validation or explicit `as`. |
| **Sidecast** (lateral) | No subgraph relationship | **Needs .dag process** | `Celsius → Fahrenheit`, `List → Set`, `Int → String` — structural transformation, not deducible. |

Upcasting and downcasting are NEVER hand-written rules. The compiler
compares the source and target graphs: if one is a subgraph of the
other, the direction and cost are determined. Only sidecasts need
explicit `.dag` transformation processes — authored by users or
libraries, not embedded in the compiler.

**Worked examples:**

```
UPCAST (free, structural):
  Url → String
  Source graph: Node "Url" { scheme: String, host: String, ... }
  Target graph: Node "String" (leaf)
  Url IS a String (refinement). Source has all target edges + more.
  Compiler sees: subgraph ✓. Cost: free. No rule needed.

DOWNCAST (checked, structural):
  Float → Int
  Source graph: Node "Float" = Field<Word64>
  Target graph: Node "Int" = OrderedRing<Word64>
  Int has constraints Float doesn't (no fractional part).
  Compiler sees: target has extra constraints. Cost: runtime check.
  Developer must write explicit `as` or `truncate`.

SIDECAST (explicit .dag process):
  Celsius → Fahrenheit
  No subgraph relationship — different structures entirely.
  Requires: fn to_fahrenheit(c: Celsius) -> Fahrenheit { c * 9/5 + 32 }
  This is a .dag function, not a compiler rule.

  List → Set
  Loses information (order, duplicates). Not a subgraph.
  Requires: fn to_set(list: List<T>) -> Set<T> { ... }
  User/library authored. The compiler can't deduce this.
```

**For language targets, the same model applies:**

A language declares its **basis** — which structural patterns it can
represent natively. The compiler compares each source graph segment
against the basis:

| Source pattern | Target basis has it? | What happens |
|---|---|---|
| Product | Rust: struct ✓ | **Identity** — same structure, different syntax |
| Coproduct | Rust: enum ✓ | **Identity** |
| Coproduct | SPICE: no native tagged union | **Sidecast** — needs `.dag` lowering process (mux from switches) |
| Cardinality | Verilog: tri-state ✓ | **Identity** |
| Function | English: paragraph ✓ | **Identity** |

When the source pattern IS in the target's basis → identity (free,
structural). When it's NOT → the language plugin provides a `.dag`
sidecast process for that pattern. Only the non-native patterns need
explicit processes.

**Why this avoids duplicate representations:** coercion rules for
upcasting/downcasting would duplicate what the graph already
expresses. The graph IS the type relationship. Reading it is free.
Only sidecasts — where the relationship genuinely doesn't exist in
the structure — need authored processes.

**Rendering happens AFTER coercion.** The renderer walks the
target-basis graph (already in native patterns) and produces text.
Trivial. All intelligence is in the structural comparison (automatic)
and the sidecast processes (authored in `.dag`).

**The guarantee receipt records the coercion plan:** for each graph
segment, what direction (upcast/downcast/sidecast), what cost, what
the sidecast process was (if any). Deterministic and auditable.

#### End-to-end example: aspiration target

A non-trivial `.dag` program compiled to three targets, showing how
every piece of the roadmap connects. This is what the system looks
like when it works.

```dag
// Source: a temperature converter service
module weather.convert

import std.types { Float, String }

type Celsius = Float where label("°C")
type Fahrenheit = Float where label("°F")

// Sidecast: Celsius and Fahrenheit are both Float refinements
// but not in a subtype relationship — needs explicit process
fn to_fahrenheit(c: Celsius) -> Fahrenheit { c * 9.0 / 5.0 + 32.0 }
fn to_celsius(f: Fahrenheit) -> Celsius { (f - 32.0) * 5.0 / 9.0 }

type TemperatureReading {
  value: Celsius
  location: String
  timestamp: Int
}

type TemperatureReport
  = SingleReading { reading: TemperatureReading }
  | DailyAverage { readings: List<TemperatureReading>, avg: Celsius }
  | Error { message: String }
```

**What the compiler proves (guarantee receipt):**

```json
{
  "discovered": { "types": 4, "functions": 2 },
  "structural": {
    "decidability": "proven (all functions terminate)",
    "complexity": {
      "to_fahrenheit": "O(1) Proven",
      "to_celsius": "O(1) Proven"
    },
    "ownership": {
      "to_fahrenheit": "SoleOwner (all bindings consumed once)",
      "to_celsius": "SoleOwner"
    }
  },
  "coercion_plan": {
    "Celsius → Float": "upcast, free (Celsius is Float refinement)",
    "Float → Celsius": "downcast, needs validation (where label check)",
    "Celsius → Fahrenheit": "sidecast, via to_fahrenheit (user-authored)"
  }
}
```

**Compiled to Rust (identity coercion — all patterns native):**

```rust
// Every structural pattern maps directly:
//   Product → struct, Coproduct → enum, Function → fn
//   Cardinality → Option, Sequence → let bindings
struct TemperatureReading { value: f64, location: String, timestamp: i64 }
enum TemperatureReport {
    SingleReading { reading: TemperatureReading },
    DailyAverage { readings: Rc<Vec<TemperatureReading>>, avg: f64 },
    Error { message: String },
}
fn to_fahrenheit(c: f64) -> f64 { c * 9.0 / 5.0 + 32.0 }
```

**Compiled to SPICE (sidecast for coproduct — mux from switches):**

```spice
* TemperatureReading: subcircuit with 3 ports (product)
.subckt TemperatureReading value location timestamp
.ends

* TemperatureReport: 3-way mux (coproduct → synthesized from switch)
* Selector signal chooses which variant is active
.subckt TemperatureReport sel_0 sel_1 reading_port avg_port err_port
V_mux_ctrl sel_0 sel_1 DC 0
.ends

* to_fahrenheit: subcircuit (function → subcircuit)
.subckt to_fahrenheit c_in f_out
E_scale f_out 0 VALUE={V(c_in)*9.0/5.0+32.0}
.ends
```

**Compiled to English (identity — all patterns have natural mappings):**

```markdown
## Temperature Reading
A temperature reading has:
- a **value** in Celsius
- a **location** (text)
- a **timestamp** (integer)

## Temperature Report
A temperature report is one of:
- a **single reading** containing one temperature reading
- a **daily average** containing a list of readings and an average
- an **error** with a message

## Conversions
- To convert Celsius to Fahrenheit: multiply by 9/5 and add 32.
- To convert Fahrenheit to Celsius: subtract 32 and multiply by 5/9.
```

**What the tests verify (per M3 test tracks):**

| Track | What it checks for this example |
|---|---|
| Discovery | All 3 targets discovered and compiled with 0 diagnostics |
| Behavioral (type roundtrip) | `TemperatureReading` construct → serialize → deserialize → equal |
| Behavioral (function) | `to_fahrenheit(100.0) == 212.0`, `to_celsius(32.0) == 0.0` |
| Edge contracts | `Celsius → to_fahrenheit → Fahrenheit`: sidecast process exists, types match |
| Coercion correctness | Upcast `Celsius → Float` is free. Downcast `Float → Celsius` requires validation. Sidecast `Celsius → Fahrenheit` uses `to_fahrenheit`. |
| Differential/parity | All 3 targets produce structurally equivalent output |
| Guarantee receipt | Receipt matches expectations, no `report_only` gaps |

#### Execution lanes (expected diffs)

**Lane A: Data-driven method/transport dispatch (M2)**

Close 73 escape hatches in emit + 22 in complexity + transport in
resolve. Same pattern as SyntaxSpec — data tables, not match arms.

| File | Change | Sites closed |
|---|---|---|
| `04_method.dag` | `builtin_function_registry()` reads from `std/algebra.dag` nodes instead of string map | ~30 string registrations |
| `complexity.dag` | `method_cost_shape()` reads cost from algebra type field, not `if method == "..."` | 22 if/else branches |
| `05_emit_rust.dag` | Method rendering reads from `runtime.dag` templates | 21 method name comparisons |
| `05_emit_python.dag` | Same | 20 method name comparisons |
| `05_emit_go.dag` | Same | 19 method name comparisons |
| `03_resolve.dag` | Transport node gets enum field, not string name | 1 comparison |
| `05_emit_rust.dag` | Transport dispatch on enum | 3 comparisons |
| `05_emit_python.dag` | Same | 3 comparisons |
| `05_emit_go.dag` | Same | 3 comparisons |
| `05_emit.dag` | Same | 3 comparisons |
| **Total** | | **~145 sites** |

**Lane B: Node.name deletion (M4/D6)**

Close the universal escape hatch. Every `.name` read becomes a
structural property read or `source_text_at(span)` call.

| File | Change | Sites closed |
|---|---|---|
| `00_core.dag` | Delete `name: String` from Node type | Field definition |
| `02_parse.dag` | ~54 Node constructions: remove `name:` field | 54 construction sites |
| `04_infer.dag` | Scope lookups use structural edges, not name strings | ~17 name reads |
| `04_method.dag` | Method identity is algebra node, not name string | ~56 name reads |
| `04_types.dag` | Type identity is structural, not `name ==` | ~22 name reads |
| `05_emit_rust.dag` | Identifiers from `source_text_at(span)` | ~51 name reads |
| `05_emit.dag` | Same | ~18 name reads |
| Other 13 files | Various name reads → structural property reads | ~957 name reads |
| **Total** | | **~1,175 sites across 20 files** |

**Lane C: Coercion engine + language plugin extraction (M5)**

The compiler stops producing strings. `05_emit.dag` becomes the
coercion engine — graph→graph transformation using target-declared
rules. Language-specific code moves out of `src/v2/` into
`dsl/extdeps/languages/` as coercion rule sets + renderers.

| File | Change | Sites closed |
|---|---|---|
| `src/v2/05_emit.dag` | Coercion engine: match patterns, apply rules, produce target-basis graph | ~77 concat → coercion search |
| `src/v2/05_emit_rust.dag` | **DELETE** — Rust coercion rules + renderer move to plugin | 4,121 lines, 309 language mentions |
| `src/v2/05_emit_python.dag` | **DELETE** — Python coercion rules + renderer move to plugin | 1,349 lines, 96 language mentions |
| `src/v2/05_emit_go.dag` | **DELETE** — Go coercion rules + renderer move to plugin | 1,387 lines, 84 language mentions |
| `src/v2/runtime_rust.dag` | **DELETE** — Rust runtime moves to extdep | 5 language mentions |
| `dsl/extdeps/languages/rust/coerce.dag` | **NEW** — Rust coercion rules (graph patterns → Rust-basis patterns) | Single file |
| `dsl/extdeps/languages/rust/render.dag` | **NEW** — Rust renderer (target-basis graph → text, trivial) | Single file |
| Same for python/, go/, verilog/, spice/, english/ | Coercion rules + renderer per target | |
| **Total** | | **6,857 lines + 632 language mentions removed from compiler core** |

**Lane D: Edge-only fact references (M5)**

Replace `List<String>` and `Map<String, X>` metadata with node edges.

| File | Change | Sites closed |
|---|---|---|
| `00_core.dag` | `kernel_types: List<Node>`, `container_types: List<Node>` | 2 string lists |
| `04_emit_info.dag` | `variant_to_enum`, `field_type_names` become node-keyed | 4 string-keyed maps |
| `04_method.dag` | `builtin_function_registry` keyed by node, not string | 1 map |
| `complexity.dag` | Function summary cache keyed by node | 1 map |
| Other files | Remaining `Map<String, X>` → `Map<Node, X>` | 6 maps |
| **Total** | | **14 string-keyed maps** |

#### Lane dependencies

```
Lane A (method/transport dispatch)    independent
Lane B (Node.name deletion)          depends on A
Lane C (graph rendering + plugin extraction) depends on B
Lane D (edge-only facts)             independent, parallel with A/B
```

#### End state (after all lanes complete)

**`src/v2/` contains only language-agnostic compiler code:**

```
src/v2/
  00_core.dag          Node, Edge types (no String on Node, no emit IR)
  01_tokenize.dag      Source text → tokens
  02_parse.dag         Tokens → Node tree
  03_normalize.dag     Structural normalization
  03_resolve.dag       Name resolution (scope dies here)
  04_*.dag             Inference (reads structure, not names)
  05_emit.dag          Coercion engine (graph → target-basis graph, no strings)
  compile.dag          Pipeline orchestration
  complexity.dag       Cost proofs (reads method cost from algebra nodes)
  ownership.dag        Ownership proofs
  trace.dag            Debug tracing
  artifact.dag         Output artifact planning
  languages.dag        LanguageSpec type definitions
```

**No `05_emit_rust.dag`, no `05_emit_python.dag`, no `05_emit_go.dag`.**
Zero mentions of Rust/Python/Go in any compiler file. Zero `concat()`
calls producing target syntax. Zero `if type_name == "..."` branches.

**Language plugins live in `dsl/extdeps/languages/`:**

```
dsl/extdeps/languages/
  rust/
    coerce.dag         Coercion rules (graph patterns → Rust basis)
    render.dag         Renderer (Rust-basis graph → text, trivial)
    emit.dag           Container templates, type maps, sharing policy
    lint.dag           Import rules, naming conventions
    runtime.dag        Runtime function signatures
    naming.dag         Case conventions
  python/
    coerce.dag, render.dag, emit.dag, lint.dag, runtime.dag, naming.dag
  go/
    coerce.dag, render.dag, emit.dag, lint.dag, runtime.dag, naming.dag
  dag/
    syntax.dag         SyntaxSpec for .dag frontend

  # Challenge targets (design validation):
  verilog/
    coerce.dag         Products → module ports, coproducts → mux (Lowered)
    render.dag         Verilog-basis graph → Verilog text
  spice/
    coerce.dag         Products → subcircuit params, coproducts →
                       comparators + switches (Synthesized, expensive)
    render.dag         SPICE-basis graph → SPICE netlist
  english/
    coerce.dag         Products → bullet lists, coproducts → "either/or"
    render.dag         English-basis graph → Markdown
```

**Adding a new language** = add `coerce.dag` (sidecast processes for
non-native patterns) + `render.dag` (trivial text from target-basis
graph) under `dsl/extdeps/languages/`. Zero compiler changes.

**Challenge targets** validate the architecture: if the coercion
engine works for Verilog, SPICE, and English, it works for anything.
These are the hardest targets — they force the compiler to find
minimal representations for patterns that don't map natively (e.g.,
coproducts in pure analog SPICE require synthesizing from
comparators, which the cost algebra reports as Synthesized/expensive).

**Ratchets at zero:**
- Language mentions in `src/v2/*.dag`: 0 (currently 632)
- `node.name` reads: 0 (currently 1,175)
- String-keyed metadata maps: 0 (currently 14)
- Method name dispatch sites: 0 (currently 63)
- Escape hatches total: 0 (currently ~290)

### Exploratory

**Everything is coercion.** The unifying concept across the compiler:
finding the minimal complete representation of a graph segment in a
target domain. This applies at every level:

- **Stage boundaries**: parse → resolve isn't an API call, it's
  coercing the unresolved graph into a resolved one (adding edges).
  Resolve → infer coerces the resolved graph into a typed one.
- **Type compatibility**: `Url` → `String` isn't a conversion
  function, it's coercion along the refinement chain. The graph
  already models the chain; the compiler walks it.
- **Language rendering**: graph → Rust/SPICE/English is coercion from
  structural patterns to the target's native capabilities. The
  compiler finds the minimal representation.
- **Efficiency**: like floating-point emulation on a target with no
  FPU — branching in pure analog SPICE is *possible* (synthesize from
  comparators + switches) but expensive. The cost algebra reflects the
  overhead. Native capabilities are free; synthesized ones have cost.

The `.dag` types.dag header already says this: "coercion insertion is
DAG transformation." The design exists (refinement chains, subtyping
as set inclusion). Implementation: zero. This is the next unifying
abstraction after the substrate primitives.

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
- **Status: wired into pipeline** (`compile.dag` runs `analyze_ownership`
  on every function, emits `SharedError` diagnostics). Verified by
  pipeline tests. Not yet ratcheted — no coverage gate in CI.

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
| Ownership | Full analysis in pipeline, verified by tests | Wired, not ratcheted | Add coverage ratchet, promote to CI |
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
| Bootstrap fixed point | PASSES | PASSES | `bootstrap_fixed_point -- --ignored` |
| Performance | <30s | <30s | `performance_ratchet -- --ignored` |

### CI Gates

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` | Every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | Every change |
| V2 compiler tests | `cargo test -p v2-compiler-tests` | Every change |
| Scrambled-name | `cargo test -p v2-compiler-tests scrambled_name` | Inference changes |

### Required Before Merge

Tier 3 ratchets that must pass before merging, until promoted to CI:

```
scripts/l1-ratchet.sh --check                                              # L1 ≤ ratchet value
cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored              # 0 diagnostics
cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored # ≤ DIAG_RATCHET
cargo test -p v2-compiler-tests bootstrap_fixed_point -- --ignored          # stage0=stage1
```

### Non-Consensual Testing

If a `.dag` file exists in this repo, it is tested. No opt-in, no
hardcoded file lists, no exceptions. The test system discovers files by
scanning the filesystem, not by reading a manifest. `full_dsl_compiles`
is the gate: no PR merges if any `.dag` file fails to compile. See M3
for the full test generation milestone.
