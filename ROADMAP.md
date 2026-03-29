# gunbc Roadmap

## Thesis

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

**1. Names are opaque namespaces.**

Type names (`Int`, `Map`, `List`) are human-readable labels for
structural compositions, not compiler-meaningful identifiers.
`Bit`/bitvectors live in the machine layer; `List`, `Map`, `Set` live
in the algebraic layer with denotational laws. The compiler must not
branch on node names for structural decisions.

**2. Compiler errors are orthogonal to the node graph.**

When inference fails, the result is not a node — it is a structurally
distinct failure. `InferredNode = Resolved { node } | CompilerError
{ message, span }`. Emit never sees error nodes. `Dynamic` and `Error`
unify into `CompilerError`.

**3. Syntactically distinct forms for the same operation normalize
before inference.**

The pipeline has a normalization boundary between resolve and infer.
After normalization: `Call`/`MethodCall` bridging is complete, nodes
carry declared structural properties from `.dag` type definitions, and
parameterized types always carry their declared arity of children.

### Decidability

Every `.dag` program is decidable. The DAG is the only computational
primitive. Recursion, loops, and cyclic-looking patterns are surface
syntax sugar that decomposes into bounded iteration over finite
structure.

The language provides:
- `fold`, `map`, `filter`, `flat_map` — bounded by collection size
- `descend` — bounded by tree depth (structural descent)
- `repeat(bound: N)` — bounded by explicit count

The language does not provide `while(true)`, unbounded `loop`, or
unrestricted recursion. Undecidable programs are structurally
unrepresentable, not detected and rejected.

### Composition Stack

| Layer | What | Location |
|-------|------|----------|
| -1 | Type constructors (Product/Coproduct/Cardinality) | Not yet in std |
| 0 | Logic (`Classical = True \| False`) | `std/logic.dag` |
| 1 | Machine (`Bit`, `Word32`, `Word64`) | `std/bit.dag` |
| 2 | Named compositions (`Int`, `String`, `Char`) | `std/integer.dag`, `std/types.dag` |
| 3 | Collection algebras (`List<A>`, `Set<A>`, `Map<K,V>`) | `std/types.dag` |
| 4 | Structural compositions + bounded iteration | `std/iteration.dag` |
| 5 | Parser/source domain (`Token<Shape>`) | Compiler domain |
| 6 | Compiler domain (records using Layer 4 shapes) | Compiler domain |

See `docs/algebraic-type-spec.md` for the collection algebra,
denotational model, and law layer.

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

- Zero type-world knowledge in the compiler (names are opaque, inference
  processes graph structure only, scrambled-name tests pass)
- Emit is name-opaque: reads `LanguageSpec` + structural declarations,
  no hardcoded `if type_name == "Map" { "HashMap" }` patterns
- One shared emit walker drives all target languages
- Language-specific facts live in `dsl/extdeps/languages/*`
- All `.dag` programs are decidable by construction
- Ownership and complexity proofs wired into the pipeline
- At least one real program compiles and runs end to end
- Compiler-internal structure converges onto `Node` compositions

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

**Work items:**
- [ ] Parser: support `fn foo<T>(...)` generic function syntax
  (`stack.dag` uses this; parser expects `(` but gets `<`)
- [ ] Verify no other .dag files break once stack.dag parses

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

### M3: Compiler Knows Zero Type Names (L1 = 0)

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

### M4: Emitted Code Correct by Construction

**What:** `LintModel` enforces emission correctness. `LanguageSpec`
carries all target-language facts. The emitter has zero hardcoded target
syntax.

**Gate:** No `match render_target` branches in emitter source. LintModel
validates every emitted file.

**Depends on:** M2 (working codegen baseline)

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

### M5: Parse-Emit Symmetry

**What:** The parser and emitter are symmetric views of the same
`LanguageSpec`. Adding a language means adding a spec file, not code.

**Gate:** `parse(spec, emit(spec, graph))` produces structurally
identical graph for all `.dag` files.

**Depends on:** M3 (name-opaque compiler), M4 (spec-driven emit)

**Work items:**
- [ ] Round-trip smoke test on `.dag` subset
- [ ] Statement dispatch (`parse_stmt`) spec-driven (3 keyword arms)
- [ ] Block/record disambiguation from heuristic to spec-driven
- [ ] Second language frontend (validates multi-frontend architecture)

---

### M6: Bit-Graph Model

**What:** Primitives are compositions. `Int = Interpret<Signed, Word64>`.
The compiler knows only Node, Conj/Disj, Cardinality, and Bit.

**Gate:** `is_kernel_type` dissolved. Bit is the only compiler-known
type.

**Depends on:** M5

**Work items:**
- [ ] Layer -1 type constructors in `std/`
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
sites, the name registry concept, and scrambled-name tests. See M3.

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

**Bit-graph type algebra.** The algebraic endgame: `Int` is not a
compiler-known primitive but `Interpret<Signed, Word64>` — a namespace
over a bitvector composition. See M6.

---

## Verification

### Ratchets

| Ratchet | Current | Target | Command |
|---------|---------|--------|---------|
| Self-compile diagnostics | 0 | 0 | `cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored` |
| full_dsl_compiles | 1 | 0 | `cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored` |
| L1 type knowledge | 70 | 0 | `scripts/l1-ratchet.sh --check` |
| Complexity violations | 0 | 0 | Self-compile complexity report |
| Bootstrap fixed point | PASSES | PASSES | `cargo test -p v2-compiler-tests v2_bootstrap_fixed_point -- --ignored` |

### CI Gates

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` | Every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | Every change |
| V2 compiler tests | `cargo test -p v2-compiler-tests` | Every change |
| Scrambled-name | `cargo test -p v2-compiler-tests v2_scrambled_name_inference` | Inference changes |

### Non-Consensual Testing

If a `.dag` file exists in this repo, it is tested. No opt-in, no
hardcoded file lists, no exceptions. The test system discovers files by
scanning the filesystem, not by reading a manifest. `full_dsl_compiles`
is the gate: no PR merges if any `.dag` file fails to compile.
