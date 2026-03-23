# gunbc Roadmap

## Architectural Thesis

**Node and DAG are the only compiler primitives.**

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules, and emits target code. All
domain knowledge — types, cardinality, containers, optionality, and
target-language facts — lives in `.dag` definitions, not in the compiler
implementation.

### Three Structural Principles

These principles refine the thesis based on root-cause analysis of the
current invariant violations (2026-03-23). Every active violation traces
back to one of these principles being underdeveloped.

**1. Names are opaque namespaces.**

Type names (`Int`, `Map`, `Optional`, etc.) are human-readable labels for
structural compositions, not compiler-meaningful identifiers. The compiler
must not branch on node names for structural decisions. `Int` is a
namespace for `List<List<bit>>`; `String` is a namespace for `List<Int>`.
At every level above the fundamental unit, names are opaque.

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
and both are failures, not types. The `.dag` language has no legitimate
"genuinely polymorphic" position today (no generics); when it does,
type variables will be a distinct structural concept.

**3. Syntactically distinct forms for the same operation normalize before
inference.**

The pipeline has a normalization boundary between resolve and infer.
After normalization: `Call`→`MethodCall` bridging is complete, nodes
carry their declared structural properties from `.dag` type definitions,
and parameterized types always carry their declared arity of children.
Infer receives a fully-normalized graph and processes one form per
semantic operation — no divergent code paths for the same concept.

### Dissolution Layers

The thesis dissolves compiler knowledge in three layers:

| Layer | What dissolves | Compiler stops knowing | Measured sites | Status |
|-------|----------------|------------------------|---------------:|--------|
| **L1: Types** | Name-checking, `node_is_*`, type constructors, `.connective` reads | What `Optional`, `List`, `Map`, `Int`, etc. mean — the compiler processes graph structure; names are opaque namespaces | ~392 | **Active — `BuiltinTypeKind` deleted, predicates centralized; name opacity and `InferredNode` wrapper are next** |
| **L2: Expressions** | `ExprData` semantic knowledge, 12+ full ExprData walks | What `if`, `for`, `match`, `let`, etc. mean | 12 walks | Future — after bootstrap and shared emit |
| **L3: Syntax** | `kind_tag` string dispatch, hardcoded parser branches | How to parse surface syntax like `if cond { body }` | ~200 string checks | Future — data-driven parser |

L1 is the urgent layer. Its endgame is not "replace name checks with
property checks" (that still puts domain knowledge in the compiler). The
endgame is: the compiler processes graph structure and reads structurally
declared properties from `.dag` type definitions. Names are opaque.
Inference cannot read them. The L1 ratchet evolves from "count name
comparisons" to "scrambled-name tests pass."

The compiler retains structural vocabulary: `Conj`/`Disj` (product vs
coproduct — graph primitives, not domain), children/parent traversal,
and cardinality. Everything above that — what `Int` means, what `Map`
means, what `Optional` means — is namespace, defined in `.dag`.

### Algebraic Type Vision

The long-term endgame: every type is a structural composition from a
single fundamental unit, and properties like cardinality, optionality,
and arity fall out of the definitions — not from compiler-injected
properties.

```
// Level 0: Fundamental unit
type Bit = True | False                           // |[Bit]| = 2

// Level 1: Fixed-width compositions
type Byte = Tuple<Bit, Bit, Bit, Bit, Bit, Bit, Bit, Bit>  // |[Byte]| = 2^8

// Level 2: Named compositions (opaque namespaces)
// Int is List<List<Bit>> — a namespace, not a compiler-known concept
// String is List<Int> — another namespace

// Level 3: Algebraic structures
// Optional<T> = T | Unit  (coproduct — cardinality = |T| + 1)
// List<T> = Nil | Cons { head: T, tail: List<T> }  (free monoid)
// Map<K,V> = List<Tuple<K, V>>  (finite function space)
// Set<T> = List<T> where unique  (powerset restriction)
```

At each level, the compiler sees only graph structure. Names are
opaque. Cardinality is a structural consequence, not a property.
`Optional<T>` IS a coproduct — the compiler processes coproducts
generically; it doesn't need an "optional" concept.

This requires generics in the `.dag` language (Phase 3+). Until then,
the compiler uses an explicit arity bridge (`kernel_types` pattern)
that gets deleted when real algebraic declarations exist. Every
bridge is a short-term regression acknowledged as such — its purpose
is to move structure upstream so it can be choked out when the real
declarations land.

Phase timeline for this vision:

| Phase | What's reachable | What's still a bridge |
|-------|-----------------|----------------------|
| Phase 1 | `InferredNode`, normalization, path dedup. Arity bridge for known types. Algebraic spec design doc. | Arity hardcoded in bridge (`Map→2`, `List→1`, `Optional→1`) |
| Phase 2 | Gist end-to-end. Arity bridge keeps types structurally complete. | Same bridge; no new emit heuristics needed because inference is sound. |
| Phase 3 | Generics land (at least enough for parameterized type declarations). Algebraic specs become real `.dag` declarations. Arity bridge deleted. | None — real declarations replace the bridge. |
| Phase 4 | Shared emit reads `LanguageSpec` + structural declarations. Emit becomes name-opaque. | None |
| Phase 5 | L1=0. Scrambled-name tests pass. Connective dissolution starts. | None |
| Beyond | Bit-graph model. Primitives as compositions. Full structural type algebra. | None |

The bit-graph model is aspirational and post-Phase 5. The pragmatic
intermediate (Phases 1-3) is: primitives and containers get real `.dag`
declarations with structural definitions; the compiler reads structure
from those declarations; names are opaque. This is sufficient for the
thesis without requiring the full algebraic foundation up front.

L3 is larger than previously acknowledged: `02_parse.dag` (3,938 lines)
dispatches on `TokenKind` entirely through a `kind_tag(token) -> String`
function, then compares strings everywhere (`check(tag: "KwFn")`,
`expect(tag: "LBrace")`, etc.). Adding a new token kind requires finding
every string comparison by hand with no compiler-enforced exhaustiveness.

---

## How To Read This Roadmap

This file now has one canonical schedule and three supporting
decompositions.

- **Phases are the canonical execution order.** If another section seems
  to imply a different ordering, the phase plan wins.
- **`M*` tracks** describe cross-cutting architecture migrations that span
  more than one phase.
- **`R*` targets** describe the desired end state of specific compiler
  modules once the naming cleanup lands.
- **`S*` passes** describe technical refactors that cut across phases and
  tracks.

The repo is still in the middle of a rename/relocation cleanup. Some
sections refer to current filenames, and some refer to target filenames.
Use this map:

| Old file | Current file (M1 complete) | Meaning |
|----------|---------------------------|---------|
| `04_reconcile.dag` | `04_infer.dag` | Stage 4 is infer/typecheck, not "reconcile" |
| `06_pipeline.dag` | `compile.dag` | Compiler driver/orchestrator, not a sixth stage |
| `07_complexity.dag` | `complexity.dag` | Proof/report layer, not a numbered stage |
| `07_ownership.dag` | `ownership.dag` | Proof/obligation layer, not a numbered stage |
| `08_artifact.dag` | `artifact.dag` | Artifact planning layer, not a numbered stage |
| `09_trace.dag` | `trace.dag` | Runtime/debug contract, not a numbered stage |

M1 naming cleanup is complete. All files use their target names. Last
stale reference (`v2.compiler.pipeline` in `05_emit_rust.dag`) fixed
2026-03-23.

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
- Container and wrapper types have real `.dag` algebraic declarations;
  cardinality and arity fall out of structure, not compiler knowledge
- Emit is name-opaque: shared emit reads `LanguageSpec` + structural
  declarations for type→target-identifier mapping (Phase 4); no hardcoded
  `if type_name == "Map" { "HashMap" }` patterns
- One shared emit walker drives all target languages through a common
  compiler-owned spine
- Language-specific facts live in `dsl/extdeps/languages/*`; program-
  dependent lowering lives in compiler-owned adapters
- Ownership and complexity proofs are wired into the compile pipeline
- At least one real program (`gist`) compiles and runs end to end
- v1 is archived (Phase 3: feature-gated in P3.5 **done**; fully removable
  once Phase 3 confirms v2 parity — "archived" means the `v1-bootstrap`
  feature flag and all v1 crates can be deleted without breaking any
  non-bootstrap workflow)
- Compiler-internal structure converges onto `Node` compositions

---

## Completed Milestones

Status labels:

- **tree-green**: verified on the current tree (`cargo test` passes)
- **prior-branch**: achieved on a previous green branch; not yet verified on this tree
- **structural**: code-level change is landed and visible; downstream gate may not pass yet

| Milestone | Gate | Status | Date |
|-----------|------|--------|------|
| Self-compile pipeline | v2 processes its own `.dag` through all 5 core stages | tree-green | 2026-03 |
| Bootstrap A5 | v1 -> stage0 -> stage1 (`cargo check`) | prior-branch | 2026-03 |
| Fixed point A6 | stage1 output == stage2 output (byte-identical) | prior-branch | 2026-03 |
| A7 Phase 1 | Self-compile reached 0 `cargo check` errors | prior-branch | 2026-03 |
| TypeExpr -> Node | 8 `TypeExpr` variants deleted | tree-green | 2026-03 |
| Expr -> Node | 21 `Expr` variants deleted, `ExprData` now lives on `Node` | tree-green | 2026-03 |
| Transport dissolution | `TransportBinding` deleted | tree-green | 2026-03 |
| Node/TypedNode unified IR | W1-W13 complete, 129 tests passing | tree-green | 2026-03 |
| Performance audit | tokenize+parse down to ~24ms | tree-green | 2026-03 |
| OOM fix | `node_type_deps` cycle detection stabilized | tree-green | 2026-03 |
| M1 naming cleanup | All non-stage files renamed to target names | structural | 2026-03 |
| Stage0 build | 18 build errors fixed, stage0 compiles cleanly | prior-branch | 2026-03 |
| Stage0 parse | 5 parser ambiguities fixed in v2 source | tree-green | 2026-03 |
| Gist pipeline | 11-file gist closure compiles with 0 diagnostics | tree-green | 2026-03 |
| V1 feature-gate | v1 crates gated behind `v1-bootstrap` cargo feature | tree-green | 2026-03 |
| Diagnostic reduction | 395 → 197 via tuple naming, error cascade, branch compatibility | prior-branch | 2026-03 |
| Diagnostic ratchet 0 | 197 → 0 via 4 root-cause fixes (map types, data scope, lookup returns, cascade suppression) | prior-branch | 2026-03 |
| RenderTarget extraction | Moved from `00_core.dag` to `artifact.dag` (orchestration, not kernel) | tree-green | 2026-03 |
| Emit metadata extraction | `emit_info` removed from ResolvedGraph; emit builds EmitGraphInfo locally | tree-green | 2026-03 |
| BuiltinTypeKind deletion | Enum and `builtin_type_kind()` fully removed from `00_core.dag` | tree-green | 2026-03 |
| RuntimeBridgeMethod enum | String-keyed bridge dispatch replaced with closed enum in core; round-trip through `runtime_bridge_method_name` remains (P1.10) | tree-green | 2026-03 |
| L1 centralized predicates | `node_is_optional`, `node_is_map`, `node_is_container` and type predicates live in `04_types.dag` (`infer_types`); emit imports them; `classify_type_structure` replaces direct `.connective` reads in emit | tree-green | 2026-03 |
| Rc policy extraction | `type_needs_rc`, `data_lookup_needs_rc_wrap`, `rc_wrapped` removed from core/reconcile/shared emit; live only in `05_emit_rust.dag` | tree-green | 2026-03 |
| Kernel types single authority | `kernel_types` in `00_core.dag` is the only source; deleted `kernel_type_names()`, `is_primitive_name()`, `build_primitive_set()` | tree-green | 2026-03 |
| Complexity match cost | `MatchCostAccum` in `cost_of_expr`; single pass over match arms (no 2^depth re-evaluation) | tree-green | 2026-03 |
| Resolve bounded OOM | `resolve_node_bounded` stops re-resolving already-resolved lookups; trusts topological binding order | tree-green | 2026-03 |

---

## Current State (2026-03-23 Audit, reconciled with branch review)

**Bootstrap note:** On this tree, `cargo test -p v2-compiler-tests --features v1-bootstrap v2_strict_compile_diagnostic_count -- --ignored` and `v2_bootstrap_fixed_point` **fail**: stage0 compile of v2 `.dag` sources reports 44 errors (`if` branches resolve to incompatible list element types across infer/parse/resolve/complexity). Workspace tests excluding `v2-compiler-tests` pass. Non-ignored v2-compiler-tests pass (115/115). Re-run the ignored gates after stage0 self-compile is green again.

**What "prior-branch" means:** Several milestones (diagnostic ratchet 0, bootstrap A5/A6/A7, stage0 build) were achieved on earlier green branches but the current tree has regressed stage0 self-compile. The `.dag` source changes are present, but the bootstrap ignored test gates do not pass. These milestones re-verify once root cause fixes (P1.9, P1.14, P1.15) land and stage0 self-compile is green again.

**Root-cause audit (2026-03-23):** All ~66 live invariant violations
trace to three root causes. Fixing root causes eliminates downstream
symptoms structurally; fixing symptoms individually is whack-a-mole.

| Root Cause | Violations | Core issue |
|---|---:|---|
| **I: Type nodes structurally incomplete** | ~32 | Parameterized types sometimes lose children (bare `leaf_node(name: "Map")` instead of `map_node(key:, value:)`). Downstream: `normalize_type_name` heuristic, leaf-vs-structured comparison, emit `"_"` placeholders, `classify_type_structure` in emit, Go `interface{}` holes. |
| **II: Error/Dynamic are names, not structure** | ~18 | Inference failures are smuggled through the type namespace as nodes named `"Error"` or `"Dynamic"`. Downstream: `node_type_equals` treats Error==anything, emit string-checks for error types, cascade suppression by name, permissive type compatibility. |
| **III: Divergent inference paths** | ~17 | `ExprCall` and `ExprMethodCall` compute the same operations through independent code paths with different logic. Downstream: duplicated map/flat_map/fold typing, asymmetric `map_insert`/`map_merge` handling, 4x bridge method name maps, inline string method checks bypassing classifiers, testgen parallel mock extraction. |

### Compositional Audit

| Area | Lines | Fns | Current state | Key issues |
|------|------:|----:|---------------|------------|
| `00_core.dag` | 800 | 28 | Target-agnostic; `BuiltinTypeKind` deleted | `runtime_bridge_method_name` in core plus per-emitter maps duplicate enum→identifier strings (P1.10). Several enum-to-string / string-to-enum paired functions (`config_property_*`, `transport_kind*`) risk drift. |
| `01_tokenize.dag` | 507 | 18 | Clean syntax leaf | No structural issues. Errors represented as `Unknown` tokens. |
| `02_parse.dag` | 3,932 | 176 | L3 debt hotspot | `kind_tag` dispatches on `TokenKind` via ~200 string comparisons. Service/resource syntax correctly dissolves into `Node`. 6 fabrication sites (empty capability on bad input, `"_"` status key, dummy EOF tokens). This is the primary L3 target. |
| `03_resolve.dag` | 459 | 12 | Cleanest authority boundary | `kernel_type_names()` deleted; callers import `kernel_types` from core. `resolve_node_bounded` trusts topological order (no redundant re-resolve of bindings; fixes diamond-shaped OOM). One defensive `None => []` fallback. |
| `04_infer.dag` | 4,442 | 110 | Core inference; P2.6 split in progress | Multiple ExprData walks. Dynamic fabrication → `CompilerError` (P1.9). Duplicated Call/MethodCall paths → deduplicated after normalization stage (P1.14/P1.15). Imports predicates and method classifiers from split modules. |
| `04_types.dag` | 456 | 40 | Split from infer (`infer_types`) | `node_is_error_type` / `node_is_dynamic` predicates exist but will be **deleted** when P1.9 lands `InferredNode` wrapper. `node_type_equals` / `node_type_compatible` permissive rules dissolve with the wrapper. |
| `04_env.dag` | 53 | 3 | Split from infer (`infer_env`) | `TypeEnv`, `TypeBinding`, merge/lookup/cycle helpers. |
| `04_method.dag` | 192 | 6 | Split from infer | `classify_reconciled_intrinsic_method`, `classify_runtime_bridge_method` (single string-to-enum entry points). |
| `05_emit.dag` | 804 | 35 | Shared helpers/context only | `classify_type_structure()`, `build_emit_context()`, name converters, `EmitContext`. Does not own tree traversal. |
| `05_emit_rust.dag` | 3,777 | 158 | Most complete backend | 19/19 intrinsic methods. 56 fabrication sites (21 are `compile_error!` -- correct fail-loud). 14 `"_"` type placeholders, 6 `todo!()`, 8 `panic!()`. `rust_bridge_fn_name` duplicates bridge naming (P1.10). |
| `05_emit_go.dag` | 1,280 | 70 | Incomplete backend | 19/19 intrinsic methods (exhaustive match; no `_ => none`). `go_bridge_method_name` duplicates bridge naming (P1.10). 13 `interface{}` type holes (silent type erasure). 1 `/* unhandled expr */` wildcard. |
| `05_emit_python.dag` | 1,256 | 70 | Incomplete backend | 19/19 intrinsic methods (exhaustive match; no `_ => none`). `py_bridge_method_name` duplicates bridge naming (P1.10). 2 `_unimplemented()` placeholders. |
| `05_emit_*` (cross) | — | — | 13 structurally identical function patterns across 3 backends | ExprData dispatch (21 arms x3), TCO (~90% identical x3), service/transport emission (identical 4-way dispatch x3). |
| `complexity.dag` | 1,216 | 33 | Good proof layer | Pipeline-wired. `intrinsic_method_cost_shape` is exhaustive. `cost_of_expr` uses `MatchCostAccum` (single-pass branch costs; avoids 2^depth blowup). |
| `ownership.dag` | 309 | 6 | Good proof layer | Pipeline-wired. `walk_expr` is 1 ExprData walk. String dispatch on `"fold"` for special accumulator threading. |
| `compile.dag` | 277 | 13 | Honest boundary shape | Returns complexity, ownership, and artifact plan. `front_end_sources` extracts shared frontend path. Complexity/ownership stages are order-independent. |
| `artifact.dag` | 113 | 2 | Consumed types only | `RenderTarget`, `Artifact`, `ArtifactPlan`, `default_artifact_plan` consumed by `compile.dag`. Speculative boundary verification types removed (P1.11 **done**). |
| `trace.dag` | 221 | 13 | Completely disconnected from pipeline | No other `.dag` file imports from it. Type-only schema with pure helpers. Should not grow until a consumer exists. |

### Active Ratchets

#### Phase-Blocking Ratchet: Diagnostics

`src/v2/tests/src/lib.rs` enforces `DIAG_RATCHET = 0` when the v1-bootstrap
path can compile v2 sources through stage0. **PHASE 2 GATE MET** on a green
bootstrap branch; if stage0 compile fails, the ignored diagnostic test fails
before the ratchet runs (see bootstrap note under Current State).

Journey: 2797 → 395 → 197 → 0. Root causes eliminated:
- RC-A: `method_receiver_element_node` for Map<K,V>, map/flat_map/fold return type propagation
- RC-B: Suppress variant/field lookup cascade diagnostics on error/leaf types
- RC-C: Imported data declarations added to scope via `merge_scope_from_imports`
- RC-D: `lookup` return type fixed from receiver to `Optional<element>`

#### Architectural Ratchet: L1 Type Knowledge Dissolution

Scripted audit via `scripts/l1-ratchet.sh`. The script and this table
measure the same categories. Run `scripts/l1-ratchet.sh --check` to
verify the ratchet (current cap: 374).

| Category | Script variable | Count | What the compiler still "knows" |
|----------|----------------|------:|----------------------------------|
| `.connective` direct access | `connective_field_count` | 17 | Product vs coproduct read from Node field |
| `Conj` / `Disj` references | `conj_disj_count` | 47 | Connective shape matching (includes parse, which must produce them) |
| Type constructors | `constructor_count` | 140 | `leaf_node`, `optional_node`, `container_node`, `tuple_node`, etc. |
| Type-name comparisons | `typename_count` | 32 | `.name == "Optional"`, `"Map"`, `"Dynamic"`, etc. |
| `node_is_*` predicate calls | `predicate_count` | 116 | Centralized type-specific dispatch helpers |
| `classify_type_structure` calls | `classify_count` | 22 | Structural classification (replaces raw `.connective` reads in emit) |
| `builtin_type_kind()` calls | `builtin_count` | 0 | **Deleted** |
| **Total** | | **374** | |

Progress since last audit: `BuiltinTypeKind` enum and `builtin_type_kind()`
are fully deleted. `classify_type_structure()` replaces direct `.connective`
reads in emit. `node_is_optional`, `node_is_map`, `node_is_container` are
centralized in `04_types.dag` (`infer_types`, imported by infer and emit).

The `node_is_*` count rose from 43 to 116 because scattered inline checks
were replaced with calls to the centralized predicates. This is correct
L1 migration behavior: concentrate knowledge into fewer predicates first,
then dissolve those predicates into structural graph traversal.

L1 acceptance (updated per thesis amendments):

- `BuiltinTypeKind` deleted **done**
- `builtin_type_kind()` deleted **done**
- `InferredNode` wrapper: compiler errors are orthogonal to node structure
- `node_is_*` predicates deleted — replaced with structural graph traversal
- `optional_node()`, `container_node()`, `pair_node()` deleted
- `normalize_type_name` deleted — unnecessary when types are structurally complete
- `classify_type_structure` deleted from emit — unnecessary when nodes carry structure
- `connective` field removed from `Node` (last structural primitive, furthest out)
- Scrambled-name tests pass: inference produces identical results regardless
  of type names (enforced by not passing name registry to infer)
- Fixed point still holds

---

## Canonical Execution Order

Use this as the source of truth for sequencing.

| Order | Phase | What it does | Blocking gate |
|-------|-------|--------------|---------------|
| 1 | Phase 1 | Fix regressions, fix root causes (InferredNode, normalization, path dedup), arity bridge, algebraic type spec | Regressions fixed; R.C. I (arity bridge), II (InferredNode), III (normalization + dedup) landed; P1.10, P1.12, P1.19-21 done; no new emit heuristics |
| 2 | Phase 2 | `gist` end-to-end through emitted Rust | `gist` builds and runs correctly; arity bridge holds (no bare type nodes reach emit) |
| 3 | Phase 3 | Compile bundle, ownership/artifact wiring, v1 retirement, generics for parameterized type declarations | v2 compiles everything v1 still matters for; algebraic `.dag` declarations replace arity bridge |
| 4 | Phase 4 | Shared emit spine, `LanguageSpec` authority, emit name-opacity | New backend = language facts + compiler-owned adapter; emit reads `LanguageSpec` for type→target mapping, no hardcoded type names |
| 5 | Phase 5 | L1=0, connective dissolution prep, L2/L3 preparation | **L1=0 gate**: scrambled-name tests pass; no arity bridges remain; no `node_is_*` predicates; `normalize_type_name` deleted; `classify_type_structure` deleted from emit |

Important clarifications:

- **No phase regresses the thesis.** Every change either moves toward
  name opacity and structural completeness, or is an explicit short-term
  bridge with a named deletion point. If a fix adds compiler type
  knowledge (emit heuristic, name check, property check), it must be
  traced upstream and the upstream fix must be on the Phase 1 workboard.
  "Temporary" without a deletion phase means "permanent later."
- Phase 1 is the only intentionally overlapping phase. **Diagnostics are
  at 0.** Regressions (R1-R4), root cause fixes (P1.9, P1.14, P1.15,
  P1.17), and invariant items (P1.10, P1.12) block Phase 2. L1
  dissolution continues in parallel toward Phase 5's L1=0 gate.
- The three root causes (I: incomplete types, II: error-as-name, III:
  divergent paths) are the organizing principle for Phase 1. Each root
  cause fix eliminates its downstream violation cluster.
- **Bridges are acknowledged regressions.** The arity bridge (P1.17) is
  hardcoded compiler knowledge — a short-term regression that replaces
  ~20 worse violations (emit `"_"` fabrication). It is deleted in Phase 3
  (P3.7) when real `.dag` declarations land. Every bridge has a named
  deletion point.
- `M*`, `R*`, and `S*` are support structures for this phase order, not
  competing schedules.

### Sequential Execution Checklist

After completing each phase, run the verification commands and confirm
the "you are here" state. If any check fails, the phase is not done.

**After Phase 1 — "The compiler is sound"**
```
cargo test --workspace --exclude v2-compiler-tests          # green
cargo clippy --all-targets -- -D warnings                   # green
cargo test -p v2-compiler-tests --features v1-bootstrap     # green
cargo test -p v2-compiler-tests v2_testgen_emits_valid_rust # green
scripts/l1-ratchet.sh --check                               # total <= ratchet (374)
```
State: 0 regressions. `InferredNode` wrapper landed. Normalization stage
exists. Arity bridge enforced. No silent/fail-open fabrication on the
bootstrap-critical Rust emit path (Go `interface{}` holes, Python
`_unimplemented()`, and Go unhandled-expr wildcard remain and are
tracked as Phase 4 violations). Testgen verified. Algebraic type spec
written. ~66 violations reduced to ~0 through root cause fixes. Fixed
point holds.

**After Phase 2 — "One real program works"**
```
# All Phase 1 checks, plus:
cargo test -p v2-compiler-tests --features v1-bootstrap \
  v2_gist_full_pipeline -- --ignored                        # green
```
State: The gist program compiles to Rust, builds, and runs in dry-run
mode. Emitted test files present for service modules. The compiler
produces a working program, not just a compiling one.

**After Phase 3 — "The compiler owns its domain"**
```
# All Phase 2 checks, plus:
cargo test -p v2-compiler-tests --features v1-bootstrap \
  v2_bootstrap_fixed_point -- --ignored                     # green
# v1 deletion proof: build and test without v1-bootstrap feature:
cargo test --workspace --exclude v2-compiler-tests \
  --no-default-features                                     # green
cargo test -p v2-compiler-tests                             # green (no --features)
```
State: Generics landed. Algebraic `.dag` declarations exist for
`Optional`, `List`, `Map`, `Set`. Arity bridge deleted. v1 fully
removable — the feature-off proof above demonstrates that removing
`v1-bootstrap` does not break any non-bootstrap workflow. Compile
bundle has authoritative shape with ownership and artifact planning.

**After Phase 4 — "Adding a backend is easy"**
```
# All Phase 3 checks, plus:
# New backend (DAG) emits serialized typed graph
# Emit has zero hardcoded type-name → target-identifier mappings
```
State: Shared emit spine serves all backends. `LanguageSpec` is the
single authority. Generated tests work across backends. DAG backend
emits a serialized artifact. Emit is name-opaque.

**After Phase 5 — "The compiler is a generic graph processor"**
```
# All Phase 4 checks, plus:
cargo test -p v2-compiler-tests v2_scrambled_name_inference  # green
scripts/l1-ratchet.sh --check                                # L1 = 0
```
State: L1=0. Scrambled-name tests pass. No `node_is_*` predicates. No
`normalize_type_name`. No `classify_type_structure` in emit. The
compiler processes graph structure only. Ready for L2 work.

**Scrambled-name test definition:** The test compares **inferred
structure** (the typed graph after inference), not emitted artifacts.
Concretely: take a program, run it through inference with real names,
record the structural decisions (which nodes get which types, which
children, which connective shapes). Then scramble all type names
(consistently across declarations and references) and re-run inference.
The two sets of structural decisions must be identical. Emit is excluded
from this test because emit legitimately reads names for target-language
identifiers — that is name-rendering, not name-dependent inference.

---

## Phase 1: Soundness, Root Causes, and L1 Dissolution

This phase fixes all regressions, eliminates the three root causes of
invariant violations, and continues L1 dissolution toward name opacity.

### Execution Principles

- **Regressions first.** No regression survives into the next work item.
- **Fix root causes, not symptoms.** Each root cause fix eliminates its
  downstream violation cluster. Point-fixing symptoms grows them back.
- **L1 is the thesis in action.** Every L1 step reduces the distance to
  "inference doesn't read names."

### Phase 1 Workboard

#### Tier 0: Regressions (fix immediately, no regressions in this PR)

| ID | Item | Root Cause | Fix |
|----|------|-----------|-----|
| R1 | RC3 safety net: 30 lines of emit heuristic compensating for reconcile losing Optional through chained field access (`emit_rust` 1591-1620) | I | Fix `FieldSummary` propagation in inference; delete emit compensation |
| R2 | Anonymous record tuple index: hardcoded 0-3, falls back to `"0"` for index >= 4 (`emit_rust` 1574-1578) | I | **Stopgap:** emit `compile_error!()` for index >= 4 (fail-loud, not a root cause fix). Real fix is the backlog item "Anonymous record target resolution" which should produce proper field access for any arity. |
| R3 | Duplicate map/flat_map/fold type refinement: ~40 lines in both `ExprCall` (1765-1803) and `ExprMethodCall` (1919-1935) paths | III | Extract shared helper; both paths call it |
| R4 | `map_insert` key type hardcoded `"String"` (`infer` 1781) | III | Read key type from receiver's map children |

#### Tier 1: Immediate invariant fixes (< 1 day each)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.12 | Emit catch-all fail-closed | Quick | `emit_typed_item` catch-all emits `compile_error!()` instead of `// unhandled` |
| P1.10 | Collapse parallel bridge name maps | **Blocking** | Delete `runtime_bridge_method_name` (dead in core); collapse 4 parallel enum→string tables to one per-target rendering without shared string intermediary |
| P1.19 | Testgen: collapse parallel mock extraction | Quick | Rust emitter's `extract_mock_props`/`starts_with_prefix` duplicates shared emit's `has_mock_prefix`/`extract_test_projections`. Delete the Rust-only copy; `emit_test_file` must consume `TestProjection` from shared emit. |
| P1.20 | Testgen: kill fabrication sites | Quick | `emit_simple_expr` wildcard (`todo!()`) and `emit_data_value_json` wildcard (`"null"`) silently produce wrong test/mock code. Replace with `compile_error!()` and diagnostic respectively. `Default::default()` fallback when no mock data exists must emit `compile_error!()` or skip the test entirely. |
| P1.21 | Testgen: verification gate | **Blocking** | Add a test that compiles a service module with mock data, extracts the test files from the emitted bundle, and asserts they are non-empty and syntactically valid Rust. Replaces the archived `v2_crate_cargo_test`. Without this, testgen can regress silently. |
| P1.13 | Dead code cleanup | **Done** | Dead functions removed from `00_core.dag` and `complexity.dag` |
| P1.11 | Delete speculative artifact types | **Done** | Only consumed plan types remain in `artifact.dag` |

#### Tier 2: Root cause fixes (structural, each eliminates a violation cluster)

| ID | Item | Root Cause | Status | Notes |
|----|------|-----------|--------|-------|
| P1.9 | `InferredNode` wrapper | II | **Blocking** | Introduce `InferredNode = Resolved { node } \| CompilerError { message, span }`. Unify `Error` and `Dynamic` into `CompilerError`. Scope: `infer_expr` returns `InferredNode`; `Node.return_type` becomes `InferredNode?` (where error types propagate through expressions). Type node children remain `List<Node>` (missing children = arity violation, Root Cause I, different problem). Eliminates ~18 downstream name-checking violations. Delete `node_is_error_type`, `node_is_dynamic`. |
| P1.14 | Normalization stage | III | Planned | New pass between resolve and infer. Unifies `Call`→`MethodCall` bridging, enforces arity completeness (via arity bridge — see P1.17), marks parser error-recovery nodes with `CompilerError`. Property population from `.dag` declarations deferred to Phase 3 (requires generics for parameterized type declarations). |
| P1.15 | Deduplicate inference paths | III | Planned | After P1.14, a single code path handles each semantic operation. Shared `refine_collection_result_type` helper. `map_insert`/`map_merge` handled uniformly. |
| P1.17 | Arity bridge | I | Planned | Hardcode arity for known parameterized types (`Map→2, List→1, Optional→1, Set→1`) in the same pattern as `kernel_types`. Normalization enforces that type nodes carry the declared number of children. **Explicit short-term bridge** — deleted when real `.dag` algebraic declarations exist (Phase 3). Every bare `leaf_node(name: "Map")` becomes a construction error. Dissolves ~20 Root Cause I violations. |
| P1.18 | Algebraic type spec (design doc) | — | Planned | Write set-theoretic structural definitions for `Optional`, `List`, `Map`, `Set`, and primitives as a design document. Not compilable yet (requires generics), but pins the algebra and serves as the blueprint for Phase 3 declarations. |

#### Tier 3: L1 dissolution (toward name opacity — ongoing, NOT Phase 1 exit requirements)

These items start in Phase 1 and continue toward Phase 5's L1=0 gate.
They do NOT block Phase 2. They are tracked here because Phase 1 is
where the foundational work (arity bridge, InferredNode) enables them.

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.16 | Scrambled-name tests | Planned | Rename all types to arbitrary strings (declaration + references), run through infer, verify identical structural decisions. Verifies the property wall. |
| P1.4 | L1 Optional/cardinality | Planned | Structural graph traversal replaces `node_is_optional`. Optionality is a structural composition (coproduct with None variant), not a name. |
| P1.5 | L1 Containers | Planned | Structural graph traversal replaces `node_is_container`, `node_is_map`. Element type is first child — no "container" concept needed. Fix bare leaf vs parameterized inconsistency (Root Cause I). |
| P1.6 | L1 Primitives | Planned | Primitives dissolve with the bit-graph model. `Int`, `String`, etc. are namespaced compositions opaque to the compiler. `.dag` declarations carry any facts emit needs. |
| P1.7 | L1 Connective dissolution | Planned | Last structural primitive. Remove `connective` from `Node` only after all consumers use structural graph traversal. |
| P1.8 | Delete residual type primitives | Planned | Delete constructors, predicates, and builtin classifiers after consumers switch. |

#### Completed

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.1 | Naming cleanup (M1) | **Done** | All non-stage files renamed; RenderTarget moved to artifact.dag |
| P1.2 | Infer cleanup via data tables | **Partial** | Emit metadata extracted. Method handling (remaining emit-side method dispatch centralization) is picked up by P4.2 (shared emit fold + target adapters). |
| P1.3 | Diagnostics ratchet -> 0 | **Done** | DIAG_RATCHET = 0. Four root-cause fixes. |

### P1.9 Design: `InferredNode` Wrapper

The compiler produces errors — it should never need to rediscover them
by string-checking node names. `Error` and `Dynamic` are both "inference
couldn't determine this." They are failures, not types.

```
type InferredNode {
  Resolved { node: Node }
  CompilerError { message: String, span: SourceSpan }
}
```

Inference returns `InferredNode`. If a child fails, the parent propagates
`CompilerError`. Emit never sees error nodes. `node_type_equals` and
`node_type_compatible` never encounter errors — the wrapper is resolved
before comparison.

**What this deletes:**
- `error_type_node()` — replaced by `CompilerError { ... }`
- `node_is_error_type(n)` — replaced by pattern match on `InferredNode`
- `node_is_dynamic(n)` — same (Dynamic = CompilerError)
- `node_type_equals` Error==anything rule (line 246 in `04_types.dag`)
- `node_type_compatible` Error/Dynamic==anything rules (lines 215-216)
- All 18 emit sites that check for error/Dynamic by name
- All `"_"` type placeholders that compensate for error-typed nodes

**Verification:** `grep -rn '"Error"\|"Dynamic"' src/v2/04_types.dag
src/v2/04_infer.dag src/v2/05_emit*.dag` returns zero results related
to type-level error/dynamic checking (diagnostic messages are fine).

**Migration boundary: types and APIs that change when P1.9 lands.**

The representation change touches every layer that currently reads
`Node.return_type` or passes type nodes through inference results.
Mechanically:

| Layer | Type/API affected | Change |
|-------|-------------------|--------|
| `00_core.dag` | `Node.return_type: Node?` | Becomes `InferredNode?` — `Some(Resolved{node})` on success, `Some(CompilerError{..})` on failure, `None` when unset |
| `00_core.dag` | `make_expr_node(return_type: Node?)` | Parameter becomes `InferredNode?` |
| `00_core.dag` | `make_expr_error_node()` | Deleted or returns `CompilerError` directly instead of fabricating a `Node{name:"Error"}` |
| `00_core.dag` | `FieldSummary.field_type: Node?` | Becomes `InferredNode?` |
| `04_types.dag` | `error_type_node()` | Deleted |
| `04_types.dag` | `node_is_error_type(n)`, `node_is_dynamic(n)` | Deleted — callers pattern-match `InferredNode` |
| `04_types.dag` | `node_type_equals`, `node_type_compatible` | Error/Dynamic special cases deleted; these functions take `Node` only (never `InferredNode` — unwrap first) |
| `04_types.dag` | `child_return_type_or_name()` | Returns `InferredNode` or is deleted |
| `04_infer.dag` | `infer_expr()` return type | Returns `InferredNode` instead of `Node` |
| `04_infer.dag` | ~15 sites fabricating `leaf_node(name: "Dynamic")` | Return `CompilerError` |
| `05_emit.dag` | `FieldSummary` consumers | Must unwrap `InferredNode` before emit; error field types produce `compile_error!()` |
| `05_emit_rust.dag` | ~9 sites checking `"Error"`/`"Dynamic"` by name | Deleted — emit never sees error nodes |
| `05_emit_go.dag` | `interface{}` type holes from error nodes | Resolved — emit receives concrete types or `compile_error!()` |
| `complexity.dag` | `cost_of_expr` reads `return_type` | Unwrap `InferredNode`; skip cost computation for `CompilerError` |
| `ownership.dag` | `walk_expr` reads `return_type` | Unwrap `InferredNode`; skip ownership for `CompilerError` |
| Serialization | Stage0 IR boundary | `InferredNode` must round-trip through v1 interpreter values (same `_variant` pattern as other sum types) |

Ordering constraint: the InferredNode wrapper changes the type of
`Node.return_type`, which touches `00_core.dag`. Every `.dag` file that
reads `return_type` needs mechanical updates. This should be done as a
single atomic commit that updates all consumers, not incrementally.

### P1.14 Design: Normalization Stage

New pass: `parse → resolve → **normalize** → infer → emit`

Normalization has two scopes, split across phases:

- **Phase 1 normalization** (P1.14): Call/MethodCall unification, arity
  enforcement via the hardcoded arity bridge (P1.17), and parser
  error-recovery tagging with `CompilerError`. This scope uses
  hardcoded knowledge of known parameterized types (`Map→2`, `List→1`,
  `Optional→1`, `Set→1`).

- **Phase 3 normalization** (P3.6/P3.7): Declaration-driven property
  population and generic slot substitution. The arity bridge is deleted
  and replaced by reading arity from real `.dag` algebraic declarations.
  This scope requires generics (P3.6) to exist first.

Phase 1 normalization job:
- Unify `ExprCall`→`ExprMethodCall` for known method patterns (the
  bridge rewrite that currently lives inside `infer_expr`)
- Enforce arity completeness: parameterized types always carry their
  declared number of children (no bare `leaf_node(name: "Map")`)
- Mark parser error-recovery nodes with `CompilerError`

Distinct from resolve (which validates names and ordering) and from infer
(which determines types and produces diagnostics). Normalize is a
structural graph rewrite: same graph in, structurally-normalized graph
out. It should be small — a single-pass graph rewrite, not a full
analysis.

### Remaining Diagnostic Correctness Items

Diagnostics reached 0. These inference gaps remain:

| Fix | Status | Notes |
|-----|--------|-------|
| Enumerate return type | Done | `List<Tuple<Int, T>>` now flows through inference |
| Fold accumulator threading | Done | `fold_accumulator_type` follows init-arg type |
| Callable/function-value type | Done | Callable type representation exists |
| Structured `ErrorCategory` | Done | Error classification moved off ad hoc strings |
| `map_insert` / `map_merge` result typing | R4 | Bare `Map` leaf in wrong places; key hardcoded to `"String"` |
| Chained field access | R1 | Depends on correct `FieldSummary` propagation |
| Tighten `node_type_equals` | P1.9 | Dissolved by `InferredNode` wrapper — error/Dynamic rules deleted, not tightened |
| `normalize_type_name` heuristic | P1.5 | Dissolved when parameterized types are always structurally complete |

### Phase 1 Exit Criteria

- All regressions (R1-R4) fixed
- `InferredNode` wrapper landed; no error/Dynamic sentinels in the type graph
- Normalization stage exists and Call→MethodCall bridging is unified
- Arity bridge enforced: parameterized types always carry declared children;
  no bare `leaf_node(name: "Map")` reaches inference or emit
- Parallel bridge name maps collapsed (P1.10)
- Emit catch-all fail-closed (P1.12)
- Testgen: parallel mock extraction collapsed onto `TestProjection` (P1.19),
  fabrication sites killed (P1.20), verification gate passing (P1.21)
- Algebraic type spec (design doc) written, pinning the structural
  definitions for `Optional`, `List`, `Map`, `Set`, and primitives
- No silent/fail-open fabrication on the bootstrap-critical Rust emit
  path — every `"_"` placeholder, silent `todo!()`, and `Default::default()`
  fallback in `05_emit_rust.dag` is replaced with `compile_error!()` or
  traced upstream. Go `interface{}` (13 sites), Python `_unimplemented()`
  (2 sites), and Go `/* unhandled expr */` (1 site) are Phase 4 scope.
- No new emit heuristics introduced — every emit-side type-knowledge
  regression is traced upstream and fixed in inference or normalization
- `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` passes
  (As of 2026-03-23, stage0 self-compile reports 44 errors — `if`-branch
  list element type mismatches across infer/parse/resolve/complexity. These
  must be resolved by root cause fixes before this gate can pass.)
- Fixed point still holds after every structural change
- Phase 2 may start once all of the above are met; L1 dissolution
  continues in parallel toward Phase 5's L1=0 gate

---

## Phase 2: Gist End to End

**Gate:** `gist.dag` plus its transitive dependencies compile to Rust,
`cargo build` succeeds, and the emitted program runs correctly in dry-run
mode.

### Current Status

- Service operation bodies are already real in `05_emit_rust.dag`
- `main.rs` workflow dispatch is already emitted
- The remaining blocker is verification through a built stage0 binary

### Why This Is Still Blocked

The v1 interpreter path cannot handle the full multi-module compile
through `compile_sources` because of lowered lambda scoping issues. That
means the real verification path is the stage0 binary, not the v1
interpreter.

The current acceptable path is:

1. Build stage0 via `v2_bootstrap_fixed_point`
2. Use the resulting binary to compile `gist`
3. Build and run the emitted Rust crate in dry-run mode

### Phase 2 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P2.1 | Gist pipeline test | **Done** | 11-file gist closure compiles with 0 diagnostics, 4 files emitted |
| P2.2 | Service operation bodies | Done | reqwest, `Command`, auth injection, dry-run mocking already landed |
| P2.3 | `main.rs` workflow dispatch | Done | Workflow subcommands and dispatch match arms already land |
| P2.4 | Multi-module extdep imports | **Done** | Verified via gist pipeline test; all 11 modules with transitive imports resolve |
| P2.5 | Emitted crate build/run | Needs verification | Test cleans up output; needs infrastructure to preserve and build emitted crate |
| P2.6 | `04_infer.dag` decomposition | **Partial** | **Started:** `04_method.dag`, `04_env.dag`, and `04_types.dag` exist and are wired from `04_infer.dag`. **Remaining:** shrink `04_infer.dag` (~4,442 lines, ~110 fns) and finish boundary cleanup. **Duplicated inference paths (R3/P1.15) and `InferredNode` wrapper (P1.9) are Phase 1 prerequisites** — they shrink infer substantially before Phase 2 decomposition continues. |

### Current Emitted Bundle Shape

Today the emitted Rust crate is already conceptually the right bundle:

```text
output_dir/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── v2_rt.rs
│   ├── gist.rs
│   ├── github_api.rs
│   ├── git.rs
│   └── ...
├── tests/
│   ├── github_api_test.rs   (generated from service mock data)
│   └── ...
```

That bundle comes out of `compile.dag` plus the Rust emitter.

### Phase 2 Exit Criteria

- `cargo test -p v2-compiler-tests v2_gist_full_pipeline -- --ignored` passes
- The emitted gist crate builds and runs in dry-run mode
- Emitted test files are present in the bundle for service modules with mock data
- No v1-only post-processing step is required to make the crate buildable

---

## Phase 3: Compile Contract, Pipeline Completion, and v1 Retirement

**Gate:** v2 compiles everything that still matters from v1, ownership is
pipeline-wired, artifact planning is real, and v1 is no longer on the
critical path.

This phase owns the compile contract work: M2, M3, and M4, plus R8 and
R9.

### Phase 3 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P3.1 | Verify parity with remaining v1 paths | **Prior-branch** | Root causes identified on a previous green branch (tuple field naming, if-branch type unification). Stage0 self-compile currently has 44 errors; parity re-verifies when stage0 is green. |
| P3.2 | Ownership wiring + authoritative compile bundle | Preparatory (ahead of Phase 3 gate) | `compile_sources` now returns `complexity`, `ownership`, and `artifact_plan`, and emit dispatch follows the planned artifact target; unsupported obligations/reporting still need consolidation |
| P3.3 | Artifact planning above emit | Preparatory (ahead of Phase 3 gate) | Default single-artifact planning now runs between infer and emit through the real artifact contract. Speculative boundary types (`BoundaryContract`, `verify_boundaries`, `ArtifactReport`) deleted in P1.11; re-add only when a real consumer lands end-to-end. Real partitioning and per-artifact orchestration remain. |
| P3.4 | Runtime shim dissolution | Mostly done | `runtime_rust.dag` (220 lines) already IS the `.dag` runtime template — the emitter calls `rust_runtime_source()` and writes `v2_rt.rs`. **Remaining:** (1) Delete the v1 legacy `v2_runtime_shim.rs` (336 lines, bootstrap-only) once v1 retires. (2) If Go/Python backends need runtime intrinsics (equivalent of `v2_rt`), add `runtime_go.dag` / `runtime_python.dag` following the same pattern. (3) Verify no `todo!()` stubs remain in `runtime_rust.dag` for functions the emitted crate actually calls. |
| P3.5 | Feature-gate v1 | **Done** | v1 crates gated behind `v1-bootstrap` feature; `cargo test -p v2-compiler-tests` runs 0 tests without feature |
| P3.6 | Generics (parameterized type declarations) | Planned | See P3.6 Design below. At least enough for `type Optional<T> = Some { value: T } \| None`, `type List<T> = Nil \| Cons { head: T, tail: List<T> }`, `type Map<K,V> = List<Tuple<K, V>>`. The algebraic specs from P1.18 become real `.dag` declarations. |
| P3.7 | Delete arity bridge | Planned | Once P3.6 lands, the hardcoded arity bridge (P1.17) is replaced by the compiler reading arity from the `.dag` declarations. Delete the bridge. Cardinality starts falling out of structure. |

### P3.6 Design: Generics as Compositional DAG Slots

Generics are not a separate system — they are the DAG composition model
applied to type declarations. A type parameter is a **slot**: a named
position in a type's DAG structure where another type can be composed in.

```
type Optional<T> = Some { value: T } | None
type List<T> = Nil | Cons { head: T, tail: List<T> }
type Map<K, V> = List<Tuple<K, V>>
```

`List<Int>` means: take the `List` DAG, fill slot `T` with the `Int`
DAG. This is structural composition — the same thing nodes already do
with children. The only new pieces are:

1. Declaring which positions are slots (in the `.dag` declaration)
2. Resolving slot references in bodies at composition time

**Slot names are meaningful namespace identifiers**, not abstract
single-letter variables. Slots are named positions in the type's DAG
structure — the same concept as named children in a product type:

```
type Map<key, value> = List<Tuple<key, value>>
type List<element> = Nil | Cons { head: element, tail: List<element> }
type Optional<inner> = Some { value: inner } | None
type Set<element> = List<element>
```

`key`, `value`, `element`, `inner` are lowercase because they are
structural positions, not type names. When you write `Map<String, Int>`,
the composition is positional: first slot (`key`) gets `String`, second
(`value`) gets `Int`. If the compiler needs the element type of a `List`,
the answer is structural: whatever was composed into the `element` slot.

**Slot representation.** A slot appears in a type declaration body as a
`TypeVar` leaf node — a leaf whose name matches a declared slot name.
The declaration Node records its slots via its `params` field: each
`Param` has `name` (the slot name, e.g. `"key"`) and `type_expr` (a
`TypeVar` marker node). This reuses the existing `Param` structure —
for function params, `type_expr` is a declared type; for type slots,
`type_expr` is the unfilled slot marker. Same structural shape,
different role determined by context (type declaration vs function
declaration). When the compiler encounters `List<Int>`, it looks up
`List`'s declaration, finds one slot `element`, and produces a concrete
Node graph with every `TypeVar` named `element` replaced by the `Int`
Node.

**Where substitution happens.** The normalization stage (P1.14, already
planned). Normalization already enforces arity completeness; with
generics, it also performs slot substitution. By the time inference sees
the graph, all slots are filled with concrete types. Inference never
encounters TypeVar nodes — they are resolved structurally before
inference runs.

**What changes per pipeline stage:**

| Stage | Change |
|-------|--------|
| **Parse** | `parse_type_def` learns `<Name, ...>` after the type name. Records slot names on the declaration Node. `finish_type_expr_from_name` generalizes: any name can accept `<Arg, ...>` (delete the hardcoded `List/Set/Map` checks at lines 1104-1115). |
| **Resolve** | Validates that slot names are unique within a declaration. Validates that `TypeVar` references in the body match declared slot names. |
| **Normalize** | Performs slot substitution: `List<Int>` → look up `List` declaration → walk body, replace `TypeVar("T")` with `Int` Node → produce concrete type. Enforces arity: `List` with 0 or 2 args is a compile error. Handles recursive references (`List<T>` in `Cons.tail`). |
| **Infer** | No change — receives fully-substituted concrete types. |
| **Emit** | No change — type nodes already carry their children. |

**Recursive types.** `type List<T> = Nil | Cons { head: T, tail: List<T> }`
— the `List<T>` in `tail` is a self-reference with the same slot
binding. Substitution produces `List<Int>` in `tail` when the outer
`List<Int>` is composed. The existing `is_self_recursive` flag on Node
already tracks this; normalization extends it to slot-aware recursion.

**Nested composition.** `List<Map<String, Int>>` — the `Map<String, Int>`
arg is itself a composition. Substitution is recursive: resolve inner
compositions first, then substitute into the outer slot. This falls out
of the normalization pass naturally (post-order traversal).

**What this deletes:**
- The arity bridge (P1.17 / P3.7) — arity is read from the declaration
- Hardcoded generic parsing in `finish_type_expr_from_name` (lines
  1104-1115 in `02_parse.dag`)
- `container_node()`, `map_node()`, `optional_node()` constructors in
  `04_types.dag` — replaced by slot substitution from `.dag` declarations
- `container_property()`, `map_type` property injection — properties
  come from the `.dag` declaration, not from hardcoded constructors

**Existing plumbing that helps:**
- `children: List<Node>` already holds type arguments at use sites
- `params: List<Param>` records slots on declarations — `Param.name` is
  the slot name, `Param.type_expr` is the `TypeVar` marker; same field
  already holds function params (role is contextual)
- Parser already handles `Name<Arg>` and `Name<Arg1, Arg2>` reference
  syntax (just hardcoded to known names)

**Open design questions (resolve in P1.18 algebraic spec):**
- Named composition syntax (`Map<key: String, value: Int>`) — more
  explicit, avoids positional arity-order errors, but adds syntax.
  Positional (`Map<String, Int>`) is the minimum viable. Decide whether
  named composition is Phase 3 scope or later.
- Higher-kinded slots (`type Functor<F<_>>`) — out of scope for Phase 3;
  note as a "Beyond" item if needed.
- Constraint syntax — likely unnecessary for the DAG model. Since
  everything is a Node, key hashing/equality is structural (the DAG
  structure itself determines comparison, not type-specific logic). No
  type is "unhashable." Constraints may emerge later if the model needs
  them, but they are not a Phase 3 concern.

### Key Decisions for Phase 3

- The compile result stops being just `files + diagnostics`
- Ownership becomes a first-class pipeline output, not a side analysis
- Artifact planning becomes part of the real compile flow, not a side
  module with stringly targets
- Unsupported proof or validation obligations must surface explicitly

### Phase 3 Exit Criteria

- The compile bundle has one authoritative typed shape
- Ownership is included alongside complexity in the pipeline output
- Artifact planning runs between infer and emit in the primary compile path
- v1 is no longer required for normal compilation
- Algebraic `.dag` declarations exist for `Optional`, `List`, `Map`, `Set`
- Arity bridge (P1.17) is deleted — compiler reads arity from declarations
- No short-term bridges remain from Phase 1

---

## Phase 4: Shared Emit, Projections, and Backend Boundaries

**Gate:** adding a new backend means writing language facts plus a
compiler-owned adapter, with no changes to the shared compiler core.

This phase owns M5, M6, and M7. M8 follows only after the Phase 4
contract is real.

### Design Rules for Phase 4

- Shared emit owns traversal
- Compiler-owned target adapters own program-dependent lowering
- `dsl/extdeps/languages/*` stays declarative
- Generated tests are first-class outputs, not Rust-only emitter details
- The DAG backend remains a compile target; execution stays in a runtime

### Phase 4 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P4.1 | `LanguageSpec` becomes the single authority | Planned | Shared emit already imports extdep language tables; remaining duplication must collapse into one contract. **This is how emit becomes name-opaque:** `LanguageSpec` + structural declarations provide the type→target-identifier mapping; emit no longer hardcodes `if type_name == "Map" { "HashMap" }`. |
| P4.2 | Shared emit fold + target adapters | Planned | Highest-risk refactor; Rust/Python/Go still own full tree dispatch today |
| P4.3 | Generated tests as first-class projection | Planned | **Prereqs: P1.19, P1.20, P1.21.** All emitters consume `TestProjection` from shared emit. Each backend owns only the test-syntax rendering (Rust `#[tokio::test]`, Go `func Test*`, Python `def test_*`). No backend owns mock extraction. Go/Python test generation is new work gated on shared emit fold (P4.2). |
| P4.4 | DAG backend/runtime boundary | Planned | Today the compiler only emits source code (Rust/Go/Python). A DAG backend would emit the **typed graph itself** as a serialized artifact (e.g., JSON representation of the `ResolvedGraph`), executed by an external runtime — not the compiler. This keeps the compiler pure: DAGs in, artifacts out. The "canonical artifact" is a well-defined serialization of the post-infer typed graph. Design: add `Dag` to `RenderTarget`, implement `emit_dag(typed: ResolvedGraph) -> EmitResult` that serializes the graph, define the schema. Runtime execution is a separate system (not in the compiler). |
| P4.5 | Typed backend plumbing and CLI surface | Mostly done | Backend selection is already typed: `RenderTarget = Rust \| Python \| Go` (closed enum in `artifact.dag`), `compile_sources` takes `target: RenderTarget`, `emit_artifact` matches exhaustively. **Remaining:** (1) Add `Dag` variant to `RenderTarget` for P4.4. (2) CLI surface for the v2 compiler binary itself (not the emitted program) should parse `--target rust\|python\|go\|dag` and produce the typed `RenderTarget` — straightforward. |
| P4.6 | Equivalence validation | Planned | Self-compile and gist must still converge after shared emit lands |

### Current Phase 4 Risks

- Shared emit is still helper-only; traversal is still per target.
  13 structurally identical function patterns exist across 3 backends:
  ExprData dispatch (21 arms x3), TCO walker (~90% identical x3),
  service/transport emission (identical 4-way dispatch x3).
- Go/Python **intrinsic** method emission is exhaustive (19/19 arms, no
  `_ => none`). Remaining backend gaps: **`interface{}` erasure**,
  Python `_unimplemented()` / Go unhandled expr wildcard, and **P1.10**
  duplicate runtime-bridge **name** maps across three emitters plus core.
- Go emitter uses `interface{}` as a type hole in 13 sites where the
  compiler has lost type information. These are fabrication sites that
  should be tracked alongside Python's `_unimplemented()` and Rust's
  `compile_error!()` as backend-specific type-loss indicators.
- `LanguageSpec` exists, but emit does not yet read it as the single
  source of truth.
- Generated tests are Rust-only and unverified. The Rust emitter has its
  own mock extraction parallel to shared emit's `TestProjection` (fixed by
  P1.19). Three fabrication sites produce wrong test code silently (fixed
  by P1.20). The verification test is archived — no gate prevents regression
  (fixed by P1.21). Go/Python have no test generation at all (new work in
  P4.3 after shared emit fold).

### Phase 4 Exit Criteria

- No backend owns a whole-tree `ExprData` dispatcher
- No backend owns a separate whole-tree TCO walker
- `LanguageSpec` is the single authority for language facts
- Emit is name-opaque: type→target-identifier mapping reads from
  `LanguageSpec` and structural declarations, no hardcoded type names
- Generated tests are first-class artifact outputs
- The DAG backend emits a serialized typed graph (JSON or equivalent)
  without embedding an interpreter in the compiler stages; a separate
  runtime can execute the artifact

---

## Phase 5: L1=0, Convergence, and L2 Preparation

**Gate:** L1 dissolution is complete. The compiler has zero type-world
knowledge. Names are opaque. Scrambled-name tests pass. The bootstrap
architecture is stable enough for deeper dissolutions.

This phase gates on L1=0. It is intentionally later — it should happen
after naming, pipeline, shared emit, and algebraic declarations are all
stable.

### Phase 5 Workboard

Phase 5 has two tracks that can run in parallel: **L3 dissolution**
(parser, P5.0-P5.1) and **L1 final deletions** (P5.6-P5.10). The
structural dissolutions (P5.2-P5.5) are independent of each other and
can interleave with either track.

#### Track A: L3 dissolution (parser)

| ID | Item | Depends on | Est. scope | Notes |
|----|------|-----------|-----------|-------|
| P5.0 | `kind_tag` string dispatch elimination | — | ~200 sites in `02_parse.dag` | Replace `kind_tag(token) -> String` + string comparisons with structural match on `TokenKind` enum. Largest single item in Phase 5. |
| P5.1 | Token dissolution | P5.0 | ~507 lines (`01_tokenize.dag`) | Replace `Token` / `TokenKind` structures with `Node` compositions. Only tractable after P5.0 removes string dispatch. |

#### Track B: Structural dissolutions (independent, any order)

| ID | Item | Depends on | Est. scope | Notes |
|----|------|-----------|-----------|-------|
| P5.2 | Module/import dissolution | — | ~459 lines (`03_resolve.dag`) | Dissolve `Module`, `Import`, and `ImportNames` into `Node` compositions |
| P5.3 | Diagnostic / compile-output dissolution | — | Moderate | Dissolve `Diagnostic`, `Severity`, `CompileResult`, and `TextFile` where it is still valuable. `InferredNode` (P1.9) already handles error representation. |
| P5.4 | Service/support type dissolution | — | Small | Verify which service-layer types still need to move. Service nodes already use `Node` composition for operations/transports. |
| P5.5 | Residual semantic enum cleanup | P5.2-P5.4 | Small | Move remaining compiler-only semantic types toward `.dag` or `Node`-based representation. Depends on prior dissolutions to identify what's left. |

#### Track C: L1 final deletions (the L1=0 gate)

| ID | Item | Depends on | Est. scope | Notes |
|----|------|-----------|-----------|-------|
| P5.6 | Scrambled-name tests (full suite) | Phase 4 (emit name-opacity) | Test suite | All compiler stages from infer onward produce identical output regardless of type names. This is the L1=0 verification gate. |
| P5.7 | Delete `node_is_*` predicates | P5.6 passing | 82 call sites | Replaced by structural graph traversal. No predicate checks type identity. |
| P5.8 | Delete `normalize_type_name` | P5.6 passing | 17 sites | Unnecessary when types are always structurally complete (arity enforced since Phase 1, declarations since Phase 3). |
| P5.9 | Delete `classify_type_structure` from emit | P5.6 passing, Phase 4 (shared emit) | 22 call sites | Unnecessary when nodes carry structure directly. |
| P5.10 | Connective dissolution assessment | P5.7-P5.9 | Design decision | Evaluate whether `Conj`/`Disj` can dissolve or remain as the compiler's last structural primitive. Depends on whether products/coproducts are derivable from the algebraic type definitions. |

### Phase 5 Exit Criteria

- **L1=0:** scrambled-name tests pass; no `node_is_*` predicates; no
  `normalize_type_name`; no `classify_type_structure` in emit; no type-name
  comparisons in inference; no arity bridges
- Target filenames from M1 are fully normalized
- Compiler-internal structure is consistently `Node`-centric
- Each convergence step survives re-bootstrap and fixed-point verification
- The compiler is in a clean place to start real L2 work

### Beyond Phase 5: Bit-Graph Model

The full algebraic vision — primitives as compositions from `Bit`,
`Int = List<List<Bit>>`, `String = List<Int>` — is post-Phase 5 work.
It requires the algebraic type system to be mature enough that the
compiler genuinely processes only graph structure and the fundamental
unit. This is the theoretical endgame, not a near-term deliverable.

---

## Cross-Cutting Reference

The sections below support the phase plan above. They do not override it.

### Structural Pass Order (`S*`)

| Pass | Primary phase | Meaning |
|------|---------------|---------|
| S1 | Done | Theme 4: `kernel_types` / `is_kernel_type` are single-authority in core |
| S2 | Done | Theme 6: pipeline owns compilation only; artifact/trace are honest side systems |
| S3 | Done | Theme 3: known-method resolution is centralized; complexity follows semantics |
| S3.5 | Phase 1 | Extract emit metadata out of infer/reconcile |
| S4 | Phase 1 | Move Rust-only ownership/render policy out of core + infer |
| S5 | Phase 4 | Fuse duplicated `ExprData` walks behind shared fold machinery |
| S6 | Phase 4 | Shared emit dispatch with per-target leaves |
| S7 | Phase 1 / Phase 4 | Remove fabrication fallbacks and finish residual string-keyed cleanup |

### Compositional Refactor Targets (`R*`)

These are written in post-M1 names.

| ID | Module | Current -> Target | Primary phase | Note |
|----|--------|-------------------|---------------|------|
| R1 | `00_core.dag` | C -> A | Phase 1 / 3 | Remove emit/pipeline-only types from core |
| R2 | `01_tokenize.dag` | A -> A | Done | No structural refactor required |
| R3 | `02_parse.dag` | C -> B+ | Phase 5 (L3) | `kind_tag` string dispatch (~200 sites) is the primary L3 debt; service/resource lowering is already clean |
| R4 | `03_resolve.dag` | A -> A | Done | No structural refactor required |
| R5 | `04_infer.dag` | D -> B+ | Phase 1 | Bootstrap-critical infer cleanup |
| R6 | `05_emit*.dag` | D -> B+ | Phase 4 | Shared traversal plus target adapters |
| R7 | `complexity.dag` | B -> A | Phase 4 | P1.13 dead code cleanup **done**; convert into a fold consumer |
| R8 | `ownership.dag` | A- -> A | Phase 3 | Wire ownership into the pipeline |
| R9 | `compile.dag` | B- -> A | Phase 3 | Complete orchestration and typed backend flow |

Practical notes:

- **R5 is the bootstrap-critical refactor.** It is the first high-value
  cleanup inside the current infer/reconcile hotspot.
- **R6 is the highest-risk refactor.** Do it only after the compile
  contract and naming cleanup are stable enough to support it.
- **R8 and R9 are Phase 3 work.** They should not wait for deep
  convergence.

### Architecture Migration Tracks (`M*`)

| ID | Track | Primary phase | Depends on | Outcome |
|----|-------|---------------|------------|---------|
| M1 | Stage/module naming cleanup | Phase 1 | none | Target filenames and stage naming are coherent |
| M2 | Compile bundle + projection contracts | Phase 3 | M1 | One authoritative compile result shape |
| M3 | Artifact planning above emit | Phase 3 | M2 | `infer -> plan -> emit` is real |
| M4 | Proof/obligation derivation contract | Phase 3 | none | Proofs/tests/reports share one contract and unsupported is explicit |
| M5 | Generated tests as first-class projection | Phase 4 | M3, M4 | Generated tests become artifact outputs, not a Rust side path |
| M6 | Shared emit spine + target adapters | Phase 4 | M3 | Shared traversal plus compiler-owned adapters |
| M7 | DAG backend/runtime boundary | Phase 4 | M2, M3, M4 | Canonical DAG artifact with runtime kept downstream |
| M8 | Mixed-backend artifact boundaries | Late Phase 4 / later | M3, M5, M7 | Typed boundary plans and generated validation across artifacts |

---

## Business Feature Track: Agent Workflow Vertical Slice

This track stays parallel to compiler convergence. Its job is to prove one
real business integration without forking the architecture.

### Guardrails

- Do not block all product value on perfect compiler convergence
- Keep the first integration narrow, typed, and auditable
- Use the first real integration to pressure-test compiler/runtime
  contracts
- Do not build a parallel ad hoc system around compiler gaps

### Preferred First Integration

The first target remains the Cursor cloud agent API / Composer 2 surface.
The exact upstream API shape must be re-verified against current docs when
implementation starts. This roadmap item is about the integration shape,
not freezing an external API contract in advance.

### Business Track Timing

- AG1 modeling can start once Phase 2 proves the compiler can emit a real
  program
- Modeling work can overlap with late Phase 2 if it does not need the full
  compile path yet
- AG2 and AG3 should not outrun the compiler contract they depend on

### AG Workboard

| ID | Item | Status | Acceptance |
|----|------|--------|------------|
| AG1 | Model the cloud agent API in `.dag` | Planned | One typed lifecycle covers credentials, request payload, agent/run handle, optional follow-up handle, result payload, and cleanup |
| AG2 | Run one end-to-end happy path | Planned | `auth upsert -> launch -> follow-up -> delete` works end to end and is auditable |
| AG3 | Record the integration challenges | Planned | Real friction points are written down, classified, and fed back into the main roadmap |

### Generated Validation Expectations

The first workflow should carry generated validation from day 0.

Generated unit-style validation:

- Missing key returns `NeedsManualProvision`
- Invalid key fails explicit validation
- Valid key returns a ready handle
- Launch, follow-up, and delete request shaping are correct

Generated integration-style validation:

- `auth upsert -> launch -> follow-up -> delete` succeeds against mocked
  responses
- Cleanup invalidates any local state/handles created for the workflow
- Follow-up after delete fails in a controlled, typed way
- Repeated delete is either idempotent or returns an explicit expected
  error

Review bar:

- Tests must prove meaningful contract behavior, not tautologies
- At least one negative-path case exists for auth validation and
  post-delete behavior
- Failures are human-legible without reading generator internals
- Anything already proven structurally by the compiler should move into
  compile-time proof, not remain as a tautological runtime test

Out of scope for the first workflow:

- PR creation/review/follow-up management
- Repository discovery beyond what the happy path needs
- Artifact download flows unless the happy path proves they are required

---

## Backlog

Items below are real, but they are not on the critical path for the
current phase order.

### Language Features

| Item | Why deferred |
|------|--------------|
| General generic syntax | Now planned as P3.6 (compositional DAG slots). Phase 3 scope covers parameterized type declarations; higher-kinded types and constraints are post-Phase 3. |
| Full linear type checking | Ownership proof work has started, but full proof remains beyond the current migration |
| Widen V5 | The conservative version covers current hot paths |

### Compiler Improvements

| Item | Why deferred |
|------|--------------|
| Anonymous record target resolution | Must fail closed, but is not blocking active phases. R2 is the fail-loud stopgap; this item is the real fix (proper field access for any arity). |
| Collection intrinsic semantics in shared IR | Worth doing after shared emit is real |
| Generated self-hosting tests and stage contracts | Valuable once the compile contract settles |
| TCO backend contract | Should be cleaned up during/after shared emit extraction |
| SCC-aware return type resolution | Not currently blocking bootstrap |

### Open Invariant Violations (grouped by root cause)

All ~66 violations trace to three root causes. Each root cause has a
structural fix in the Phase 1 workboard above. Symptoms dissolve when
root causes are fixed.

#### Root Cause I: Type nodes structurally incomplete (~32 sites)

Dissolved by: P1.5 (arity enforcement), P1.14 (normalization), P1.16
(scrambled-name tests verify no name-dependent inference).

| ID | Violation | Where | Dissolved by |
|----|-----------|-------|-------------|
| I-1 | `"_"` for Map with no children | `emit_rust` 764 | P1.5: Map always has key/value children |
| I-2 | `"_"` for collection element | `emit_rust` 1959-1962 | P1.5: containers always have element child |
| I-3 | `"_"` for lambda param | `emit_rust` 2001 | P1.14: normalize populates param types |
| I-4 | `"_"` for fold accumulator | `emit_rust` 2033, 2170-2173 | P1.15: unified fold inference |
| I-5 | `"_"` for sort_by element | `emit_rust` 2189-2190 | P1.5: receiver element type always present |
| I-6 | RC3 safety net | `emit_rust` 1591-1620 | **R1**: fix FieldSummary propagation |
| I-7 | Anon record index cap → `"0"` | `emit_rust` 1574-1578 | **R2**: `compile_error!()` for index >= 4 |
| I-8 | `normalize_type_name` heuristic | `04_types` 165-177 | P1.5: structurally complete types make this unnecessary |
| I-9 | `node_type_equals` leaf==structured | `04_types` 267-272 | P1.5: bare leaves for parameterized types don't exist |
| I-10 | `node_type_compatible` empty children → true | `04_types` 226-239 | P1.5: children always present |
| I-11 | `node_type_compatible` name-only fallback | `04_types` 242 | P1.5 + P1.7: structural comparison |
| I-12 | `classify_type_structure` in emit (13 calls) | `emit_rust` multiple | P1.5/P1.7: nodes carry structure directly |
| I-13 | `field_type_names` composite string key | `04_infer` / `emit_rust` | P1.5: field types on nodes directly |
| I-14 | Two composite key formats (`\|` vs `::`) | `04_infer` / `05_emit` | Same |
| I-15 | Go `interface{}` type holes (13 sites) | `emit_go` | P1.5: concrete type nodes |
| I-16 | Lambda params default to `Dynamic` | `04_infer` ~6 sites | P1.9 + P1.14: `CompilerError`, not Dynamic |
| I-17 | Fold acc defaults to `Dynamic` | `04_infer` 1684, 1886 | P1.15: unified fold inference |
| I-18 | `bare_map_node()` for `empty_map()` | `04_infer` 1817 | P1.5: typed empty map from context |
| I-19 | `child_return_type_or_name` falls to `.name` | `04_types` 27-29 | P1.5: `return_type` always set |
| I-20 | `method_receiver_element_node` returns receiver for odd shapes | `04_types` 403-417 | P1.5: structural element extraction |

#### Root Cause II: Error/Dynamic are names, not structure (~18 sites)

**Dissolved by P1.9 (`InferredNode` wrapper).** All sites below are
eliminated when error/Dynamic become `CompilerError` outside the node
graph.

| ID | Violation | Where |
|----|-----------|-------|
| II-1 | `node_type_equals` Error == anything | `04_types` 246 |
| II-2 | `node_type_compatible` Error == anything | `04_types` 215 |
| II-3 | `node_type_compatible` Dynamic == anything | `04_types` 216 |
| II-4 | Dynamic/Error → `compile_error!()` in type emission | `emit_rust` 762-763 |
| II-5 | Lambda scope → `Dynamic` fabrication | `emit_rust` 1974 |
| II-6 | Field type checks against `"Dynamic"`/`"Error"` strings | `emit_rust` 2491, 2510, 2595 |
| II-7 | `ExprMatch` result = first arm only | `04_infer` 1986-1988 |
| II-8 | `ExprIf` result = then only | `04_infer` 2042 |
| II-9 | `ExprListLit` element = first only | `04_infer` 2089-2096 |
| II-10 | `Optional == Unit` name shortcut | `04_types` 217-218, 249-250 |
| II-11 | `node_is_error_type` checks in emit | `emit_rust` 1990, 2243 |
| II-12 | `emit_typed_item` catch-all → comment | `emit_rust` 559 |
| II-13 | Resolve builtin → `Unit` for unknown | `04_method` 187-191 |
| II-14 | Unknown method → result = receiver type | `04_infer` 1912-1914 |

#### Root Cause III: Divergent inference paths (~17 sites)

Dissolved by: P1.14 (normalization), P1.15 (deduplication), and P1.19
(testgen parallel extraction).

| ID | Violation | Where |
|----|-----------|-------|
| III-1 | Duplicate fold init/acc inference | `04_infer` 1670-1684 vs 1872-1887 |
| III-2 | Duplicate element typing | `04_infer` 1686-1718 vs 1888-1901 |
| III-3 | Duplicate map/flat_map/fold result refinement | `04_infer` 1765-1803 vs 1919-1935 |
| III-4 | `map_insert`/`map_merge` only in Call bridge | `04_infer` 1777-1796 |
| III-5 | `map_insert` key hardcoded `"String"` | `04_infer` 1781 |
| III-6 | String method names in `ExprCall` bridge | `04_infer` 1670+ |
| III-7 | 4x `RuntimeBridgeMethod` → string maps | `00_core`, `emit_rust/go/python` |
| III-8 | `runtime_bridge_method_name` dead in core | `00_core` 235-266 |
| III-9 | `py_bridge_method_name` diverges (`"with_update"` vs `"with"`) | `emit_python` 693 |
| III-10 | `ownership.dag` `"fold"` string dispatch | `ownership` 2 sites |
| III-11 | Inline string checks duplicate classifiers | `04_infer` |
| III-12 | Builtin vs bridge typing disagrees | `04_method` |
| III-13 | Testgen: `extract_mock_props`/`starts_with_prefix` duplicates shared `has_mock_prefix`/`extract_test_projections` | `emit_rust` 3028-3036 vs `05_emit` 121-140 |

#### Testgen fabrication (Phase 1, P1.20)

| ID | Violation | Where | Dissolved by |
|----|-----------|-------|-------------|
| TG-1 | `emit_simple_expr` wildcard → `todo!()` in generated test mock setup | `emit_rust` 1090, used at 3321 | P1.20 |
| TG-2 | `emit_data_value_json` wildcard → `"null"` in mock JSON data | `05_emit` 413 | P1.20 |
| TG-3 | `Default::default()` fallback when no mock data or `first` returns `None` | `emit_rust` 3092, 3096 | P1.20 |
| TG-4 | `TestProjection` type and `extract_test_projections` defined but never called (dead abstraction) | `05_emit` 115-140 | P1.19 |

#### Phase 4 violations (deferred, not blocking)

| Violation | Where | Dissolved by |
|-----------|-------|-------------|
| Triple `emit_typed_expr` 22-arm parallelism | `emit_rust/go/python` | P4.2: shared dispatch |
| Triple TCO walk parallelism | `emit_rust/go/python` | P4.2: shared TCO dispatcher |
| Triple service/transport emission | `emit_rust/go/python` | P4.2: shared service emit |
| Go `interface{}` type erasure (13 sites) | `emit_go` | P1.5 + P4 |
| Python `_unimplemented()` (2 sites) | `emit_python` 1132, 1145 | P4 |
| Go `/* unhandled expr */` wildcard | `emit_go` 613-614 | P4 |
| `02_parse.dag` `kind_tag` string dispatch | `02_parse` (~200 sites) | P5 (L3) |

#### Resolved

| Violation | Status |
|-----------|--------|
| `classify_reconciled_intrinsic_method` string ladder | **Done** |
| `classify_runtime_bridge_method` string ladder | **Done** |
| Go/Python intrinsic `_ => none` | **Done**: exhaustive 19-arm matches |

### Review follow-ups (branch reconciliation)

| ID | Item | Notes |
|----|------|-------|
| **F4** | Parser `item_kind` (body without type annotation → `FnItem`) | Verify gist closure: no un-annotated data defs misclassified |
| **F6** | Unused `runtime_bridge_method_name` import in `04_infer.dag` | **Done** |
| **F7** | Data constants `.clone()` from `lazy_static` | Optional: avoid redundant clones on `Copy` types if profiling warrants |

---

## Verification

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` | After every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | After every change |
| V2 compiler tests (with bootstrap) | `cargo test -p v2-compiler-tests --features v1-bootstrap` | After every change |
| Diagnostics ratchet | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_strict_compile_diagnostic_count -- --ignored` | End of Phase 1 |
| Fixed point | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_bootstrap_fixed_point -- --ignored` | After any `.dag` change that affects bootstrap output |
| Gist pipeline | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_gist_full_pipeline -- --ignored` | End of Phase 2 |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | After any `.dag` change (goal: 0) |
| Testgen gate | `cargo test -p v2-compiler-tests v2_testgen_emits_valid_rust` | After P1.21; verifies generated test files are non-empty and syntactically valid |
| Scrambled-name tests | `cargo test -p v2-compiler-tests v2_scrambled_name_inference` | After P1.16; verifies name opacity |

**Scrambled-name test design:** Rename all type names to arbitrary strings
(consistently across declarations and references), run through inference,
compare **inferred structure** (typed graph shapes: which nodes carry which
types, children, connective shapes). If inference depends on `"Map"` being
called `"Map"`, the structural decisions diverge and the test breaks.
Scoped from infer onward — parse and resolve legitimately work with real
names. Emit is excluded because it legitimately reads names for target-
language identifiers (name-rendering, not name-dependent inference).
Inference receives nodes with opaque names and no name registry.

Manual Phase 2 smoke still exists in addition to the automated test:
build the emitted gist crate and run it in dry-run mode. There is not yet
a dedicated `v2_gist_end_to_end` test in the tree, so the roadmap should
not pretend that one exists.

**Review-queue discipline:** Prefer **scoped commits** (one invariant or
theme per commit) on automation branches, per `CLAUDE.md`, so CI and
blame stay attributable when diffs are large.
