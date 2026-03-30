# gunbc Roadmap

## Architecture (summary)

Two substrate primitives: **Node** and **Edge**. Everything else —
types, truth values, cardinality, product/coproduct — is compositional
modeling in `.dag`. Languages are coercion targets. Testing is
compilation.

Full thesis: [docs/architecture.md](docs/architecture.md)
Compiler laws and coercion model: [docs/compiler-laws.md](docs/compiler-laws.md)
Testing strategy: [docs/testing-strategy.md](docs/testing-strategy.md)

---

## Current State (2026-03-30)

### Dashboard

| Metric | Value | Target | Notes |
|--------|-------|--------|-------|
| .dag files | 90 | — | `dsl/` (+3 transport extdeps) |
| Self-compile time | 6.47s | <30s | Release. Tokenize 4.87s dominates |
| Self-compile diagnostics | 0 | 0 | Green (pipeline reports 0; bootstrap ratchet allows 3) |
| Files emitted | 40 | — | Rust target |
| `full_dsl_compiles` | PASSES (0 diag) | 0 | Fixed: generic fn, fold, node scoping, filter, pattern uses |
| Bootstrap ratchet (`DIAG_RATCHET`) | 3 | 0 | `dag/syntax.dag` excluded (OOM) |
| L1 ratchet | 70 | 0 | 69 type constructors + 1 comparison |
| Complexity violations (`COMPLEXITY_RATCHET`) | 2 | 0 | **BLOCKED: test OOMs (SIGKILL).** Cannot identify violations or wire fail-closed gate until fixed. |

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
- **Variadic arguments not supported.** Arguments are children (nodes),
  so N args is structurally natural. But `04_resolve.dag:336` enforces
  strict arity (`expected == actual`). Should be free from the modeling.
  Blocks concat consolidation as regular variadic function.
- **Builtin function registry is a parallel authority.** 24 standalone
  functions in `builtin_function_registry()` duplicate algebra method
  signatures. Fix: convert to method syntax, delete registry. (Lane A
  acknowledged bridge with deletion point.)

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

## Governance

### Invariant Ownership Matrix

Every sustainability invariant has an owning milestone, a current
escape hatch, and a gate. If an invariant is not in this table, it
is not owned.

| Invariant | Current escape hatch | Owning milestone | Gate |
|---|---|---|---|
| No duplicate representations | `node.name`, `List<String>` fact lists, stage0 hand-edits | M4 (name deletion), M5 (edge-only facts) | L1 ratchet = 0, language mentions = 0 |
| No case enumeration for open sets | `if method == "fold"`, `if type_name == "Int"` | M2 (data-driven dispatch), M4 (name deletion) | Method dispatch sites = 0 |
| No fallbacks that fabricate | `Dynamic` compat, `LitNull` sentinels, warning-permissive gate, `try_unwrap` clone fallback | **M2 (new sub-track)** | Zero error sentinels reaching emit, zero warning-gated semantic gaps |
| Heuristics indicate lost structure | `concat()` in emitter, string-based type identity | M5 (coercion engine) | Zero `concat()` in emitter, zero string return types |
| No parallel implementations | 4 source-discovery paths, stage0 hand-editable | M1 (single discovery), M2 (regen CI) | ONE discovery implementation |
| Boundary sufficiency | `node.name` as proxy for structural facts | M4 (name deletion) | Scrambled-name tests pass, then deleted |
| Single-authority metadata | Ratchet values in roadmap AND code | M3 (guarantee receipt) | Receipt is single authority; dashboard derived |

### No-Fabrication Sub-Track (owned by M2)

The "no fallbacks that fabricate" invariant has the most scattered
debt. These are concentrated work items, not spread across other
milestones:

- [ ] Remove `Dynamic` as universal compatibility in `node_type_equals`
- [ ] Remove `LitNull` sentinel nodes from inference (23 parser sites
  are OK — error recovery. 14 inference/emit sites are not.)
- [ ] Promote `access_error` / `inference_error` from Warning to Error
  so `compile_sources` gates correctly (currently emit runs on known
  inference gaps)
- [ ] Remove callable-to-value fabrication in `lookup_in_scope`
- [ ] Delete `try_unwrap` clone fallback — ownership proof or fail

### Stage Boundary Contract Table

Each stage boundary carries exactly the facts the next stage needs.
If a fact is missing, the downstream stage compensates with a
heuristic — which is an invariant violation.

| Boundary | Producer → Consumer | Guarantee | Forbidden escape | Test |
|---|---|---|---|---|
| **Tokenize → Parse** | `List<Token>` | Every token has text + span + shape | Reading raw source in parser | Token coverage tests |
| **Parse → Resolve** | `Node` tree with spans | Structure faithful to source, identifiers as child nodes | — | Parse smoke tests |
| **Resolve → Infer** | `Node` tree + structural edges | Names consumed, edges produced. Scope dies here. | Reading `node.name` after resolve | Scrambled-name tests |
| **Infer → Emit** | Typed graph (`.inferred` on every node) | No error sentinels in typed graph. Types are structural. | `Dynamic` compat, `<error:*>` strings, `LitNull` in emit input | Zero error sentinels test |
| **Emit → Renderer** | Target-basis graph (coerced) | All patterns in target basis. No source-language concepts. | `concat()` producing target syntax, `if target == ...` | Zero language mentions in `05_emit.dag` |
| **Renderer → Output** | `TextFile` | Valid target-language text | — | `cargo check` / `python3 -m py_compile` / `go vet` |

### Forbidden Moves

These are never acceptable, regardless of milestone pressure:

- **No new `Map<String, X>` after resolve.** Resolver-local scope maps
  die at the boundary. New string-keyed metadata crossing a boundary
  is a new duplicate representation.
- **No new fact table without a same-change consumer.** A boundary
  fact table that no downstream stage reads in the same PR is
  speculative metadata — delete it until the consumer exists.
- **No new validation pass where a type change can enforce the
  contract.** If you're writing `assert(x.is_valid())`, refactor the
  upstream type so invalid states are unrepresentable.
- **No new stringly metadata after resolve.** Anything that changes
  behavior must be an edge, a closed enum, or a typed boundary fact.

### Temporary Bridge Rules

Every compatibility bridge names its owner, delete trigger, and latest
milestone. "Temporary without a ratchet means permanent later."

| Bridge | Owner | Delete trigger | Latest milestone |
|---|---|---|---|
| `connective: Conj/Disj` enum | M7 | Edge connectivity model replaces enum | M7 |
| `return_cardinality` enum | M7 | Edge existence replaces enum | M7 |
| `node.name: String` | M4 | `source_text_at(span)` + edges replace all reads. Infrastructure landed (B0-B4), rendering reads migrated. Blocked: synthetic node identity needs M4 type dissolution. | M4 |
| `kernel_types: List<String>` | M4 | `List<Node>` edges to type definitions | M4 |
| `container_types: List<String>` | M4 | `List<Node>` edges to type definitions | M4 |
| `05_emit_rust/python/go.dag` in `src/v2/` | M5 | Moved to `dsl/extdeps/languages/` plugins | M5 |
| `builtin_function_registry()` | M4 | Convert ~260 standalone calls to method syntax, delete registry | M4 |
| `COMPLEXITY_RATCHET = 2` | M2 | Fail-closed compilation → 0 violations | M2 |
| `DIAG_RATCHET = 3` | M2 | `dag/syntax.dag` OOM fix → 0 diagnostics | M2 |

---

## Milestones

### M1: Every .dag File Compiles

**What:** Every `.dag` file in the repo compiles as a unit with zero
diagnostics. No hardcoded file lists, no exceptions.

**Gate:** `full_dsl_compiles` scans ALL `.dag` files in the repo
(`dsl/` AND `src/v2/`) and compiles them as a unit with 0 diagnostics.
Currently only scans `dsl/`.

**Status:** 1 diagnostic remaining (`stack.dag` generic fn syntax).

**Acceptance condition:** ONE source-discovery implementation used
everywhere. CLI (`--source-root`), `full_dsl_compiles`, bootstrap,
and complexity tests all use the same transitive-import resolution.
Remove legacy `--source-dir` flag. Delete `prepare_sources` curated
assembly. Delete manifest-based approaches (BOOTSTRAP.md still
mentions converging on a manifest — contradicts "scan, not manifest").

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

**Gate:** `gunbc compile dsl/examples/weather/ --target rust && cargo check`
passes on the committed example project (not a subjective "non-trivial"
claim — one specific fixture).

**Depends on:** M1

**Work items:**

*Fail-closed decidability:*
- [ ] Reject non-descending recursion as hard compile error (currently
  produces `DivideAndConquer` classification, soft). `fn spin(n: n)`
  must not compile.
- [ ] Diagnostic: "recursive function X has no structural descent —
  use `fold`, `descend`, or `repeat` instead"
- [ ] CI gate: `complexity_violation_count == 0` (currently 2)
- **BLOCKER:** `strict_complexity_violation_count` test OOMs (SIGKILL
  in container). Cannot identify the 2 violations or wire fail-closed
  gate. Fix OOM before proceeding.


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
- [ ] Committed example project: `dsl/examples/weather/` (the
  aspiration target from this roadmap). Gate is one exact command:
  `gunbc compile --source-root dsl/examples/weather --target rust &&
  cargo check`
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

Full design: [docs/testing-strategy.md](docs/testing-strategy.md)

**What exists today:**
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

**Work items:**

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

*Ratchet promotion (Tier 3 → Tier 2 — ALL live ratchets):*
- [ ] Complexity violations in CI (currently 2, target 0)
- [ ] Emitted Rust errors in CI (currently 880, target 0)
- [ ] Ownership coverage in CI (currently not tracked)
- [ ] Bootstrap fixed-point in CI (currently manual)
- [ ] Performance ratchet in CI (currently manual, 30s)

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
- [x] Decide name source: span derivation via `source_text_at(source, span)`
- [x] `source_text_at` infrastructure (B0) + test proving span→text recovery
- [x] Thread source_text through pipeline: SourceFile → ResolvedModule →
  TypeEnv → InferScope → TypedModule → emit (B2/B2.5)
- [ ] Migrate emit rendering reads to `source_text_at` (B3 — REVERTED:
  parser item spans point to keyword tokens, not identifiers. Needs
  identifier span stored separately before B3 can proceed.)
- [ ] Migrate resolve type lookups to `source_text_at` (B4a — REVERTED:
  same span issue)
- [x] Synthetic name dissolution: tuple field constants centralized,
  module/import markers moved to property values (B1b/B1c)
- [x] `extern fn` syntax deleted (dead code, wrong model)
- [ ] Update 17 `make_*` helpers + 11 accessor functions
- [ ] Update remaining ~256 Node constructions to drop `name:`
- [ ] Migrate synthetic node identity to structural (see audit below)
- [ ] Delete `Node.name` field, delete scrambled-name tests

*D6 blocker: Synthetic node audit.*
Synthetic nodes = compiler-fabricated with `no_span()` / `zero_span`.
`source_text_at` cannot recover text for them. Each family needs
either a deletion point (becomes real .dag declaration) or a reason
it is truly compiler-owned. Dangerous = permanent semantic authority.

| Synthetic family | Count | Status | Deletion point |
|---|---|---|---|
| Kernel type constants (`int_type`, `string_type`, etc.) | 6 | Bridge | M4: kernel types become .dag declarations loaded from `std/types.dag` |
| `leaf_node(name: ...)` | 68 L1 sites | Bridge | M4: type identity from declaration edges, not fabricated leaves |
| Algebra method fields (`algebra_method_field`) | ~50 | Bridge | M4: methods read from `std/algebra.dag` declaration nodes |
| Tuple children (`"first"`, `"second"`) | 2 | Bridge | M4: Tuple becomes .dag type definition |
| Optional skeleton (`Some`, `None`, `value`) | 3 | Bridge | M4: Optional becomes .dag type definition |
| Module/import markers | 3 | Bridge (B1c) | Moved to property values; structural markers deferred |
| `error_type` / `none_type` | 2 | Compiler-owned | Permanent: error sentinels are compiler infrastructure |
| `container_node` / `callable_node` / `map_node` | ~15 L1 | Bridge | M4: type constructors → .dag declarations |

Rule: **synthetic node with zero span = red flag** that the compiler
still needs `.name` for semantics. Acceptable only as bridge with
clear M4 deletion point.

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

### M5: Coercion Engine + Language Plugin Extraction

**What:** The compiler stops producing target syntax. `05_emit.dag`
becomes the coercion engine. Language-specific code moves out of
`src/v2/` into `dsl/extdeps/languages/` as coercion rule sets +
renderers. LintModel enforces emission correctness. Zero language
mentions in compiler core.

**Gate:** Zero `match render_target` branches. Zero language mentions
in `src/v2/*.dag`. `05_emit_rust/python/go.dag` deleted from compiler
core. LintModel validates every emitted file.

**Depends on:** M2 (working codegen baseline), M3 (generated tests
verify correctness), M4 (name-opaque compiler)

Execution lanes: [docs/compiler-laws.md](docs/compiler-laws.md#execution-lanes)

**Work items:**

*Coercion engine (Lane C):*
- [ ] `05_emit.dag` walks typed graph, matches structural patterns,
  invokes language-declared coercion + renderer
- [ ] Delete `05_emit_rust.dag` (4,121 lines) → `rust/coerce.dag` +
  `rust/render.dag` in `dsl/extdeps/languages/`
- [ ] Delete `05_emit_python.dag` (1,349 lines) → `python/coerce.dag`
- [ ] Delete `05_emit_go.dag` (1,387 lines) → `go/coerce.dag`
- [ ] Delete `runtime_rust.dag` → `rust/runtime.dag` extdep
- [ ] Reconcile→emit boundary cleanup (INVARIANTS.md Root Cause A
  debt: field access style, Rc wrapping, variant→enum mapping)

*LanguageSpec completion (~11 missing fields):*
- [ ] `statement_terminator`, `variable_declaration_keyword`
- [ ] `assignment_operator`, `lambda_syntax`
- [ ] `callable_type_template`, `error_expression`
- [ ] `null_coalesce`, `string_interpolation`
- [ ] `container_bracket`, `tuple_type_template`
- [ ] `indentation_width`

*LintModel wiring:*
- [ ] Wire import rules, naming conventions, formatting model

*Edge-only fact references (Lane D):*
- [ ] 14 `Map<String, X>` metadata maps → structural edges

*Compiler bug fixes:*
- [ ] Optional exhaustiveness: structural, not `Some`/`None` hardcoded
- [ ] Single-variant enum parsing

*Challenge targets (design validation):*
- [ ] Verilog `coerce.dag` + `render.dag`
- [ ] SPICE `coerce.dag` + `render.dag`
- [ ] English/Markdown `coerce.dag` + `render.dag`

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
compilation is designed but not wired as a hard error. Owned by M2.

**Compositional parser (LANDED).** `SyntaxSpec` in `languages.dag`.
`parse_item` has 0 keyword match arms. Operator precedence, item forms,
and literal keywords all spec-driven. Adding an item type = one entry
in `syntax.dag`.

**Node.name deletion (NEXT — D6).** Identity is the node itself. Text
derived from `source_text_at(span)`. Eliminates ~553 `.name` read
sites, the name registry concept, and scrambled-name tests. See M4.

### Exploratory

**Everything is coercion.** The unifying concept across the compiler:
finding the minimal complete representation of a graph segment in a
target domain. Applies at every level: stage boundaries (parse →
resolve is coercing unresolved graph into resolved one), type
compatibility (`Url` → `String` is coercion along the refinement
chain), language rendering (graph → Rust/SPICE/English). See
[docs/compiler-laws.md](docs/compiler-laws.md#backend-model-graph-coercion).

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
and [docs/testing-strategy.md](docs/testing-strategy.md) for the full
test generation strategy.
