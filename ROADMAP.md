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

M1 naming cleanup is complete. All files now use their target names.

---

## End Goal

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules defined in `.dag`, and emits
target artifacts. Adding a type, expression, language, transport, or
runtime contract should mean editing `.dag` files, not compiler code.

Concrete acceptance:

- Zero type-world knowledge in the compiler (L1 complete): names are
  opaque namespaces; inference processes graph structure only; scrambled-
  name tests pass
- Compiler errors are orthogonal to nodes: `InferredNode` wrapper;
  no error/Dynamic sentinels in the type graph
- One shared emit walker drives all target languages through a common
  compiler-owned spine
- Language-specific facts live in `dsl/extdeps/languages/*`; program-
  dependent lowering lives in compiler-owned adapters
- Ownership and complexity proofs are wired into the compile pipeline
- At least one real program (`gist`) compiles and runs end to end
- v1 is archived
- Compiler-internal structure converges onto `Node` compositions

---

## Completed Milestones

| Milestone | Gate | Date |
|-----------|------|------|
| Self-compile pipeline | v2 processes its own `.dag` through all 5 core stages | 2026-03 |
| Bootstrap A5 | v1 -> stage0 -> stage1 (`cargo check`) | 2026-03 |
| Fixed point A6 | stage1 output == stage2 output (byte-identical) | 2026-03 |
| A7 Phase 1 | Self-compile reached 0 `cargo check` errors | 2026-03 |
| TypeExpr -> Node | 8 `TypeExpr` variants deleted | 2026-03 |
| Expr -> Node | 21 `Expr` variants deleted, `ExprData` now lives on `Node` | 2026-03 |
| Transport dissolution | `TransportBinding` deleted | 2026-03 |
| Node/TypedNode unified IR | W1-W13 complete, 129 tests passing | 2026-03 |
| Performance audit | tokenize+parse down to ~24ms | 2026-03 |
| OOM fix | `node_type_deps` cycle detection stabilized | 2026-03 |
| M1 naming cleanup | All non-stage files renamed to target names | 2026-03 |
| Stage0 build | 18 build errors fixed, stage0 compiles cleanly | 2026-03 |
| Stage0 parse | 5 parser ambiguities fixed in v2 source | 2026-03 |
| Gist pipeline | 11-file gist closure compiles with 0 diagnostics | 2026-03 |
| V1 feature-gate | v1 crates gated behind `v1-bootstrap` cargo feature | 2026-03 |
| Diagnostic reduction | 395 → 197 via tuple naming, error cascade, branch compatibility | 2026-03 |
| Diagnostic ratchet 0 | 197 → 0 via 4 root-cause fixes (map types, data scope, lookup returns, cascade suppression) | 2026-03 |
| RenderTarget extraction | Moved from `00_core.dag` to `artifact.dag` (orchestration, not kernel) | 2026-03 |
| Emit metadata extraction | `emit_info` removed from ResolvedGraph; emit builds EmitGraphInfo locally | 2026-03 |
| BuiltinTypeKind deletion | Enum and `builtin_type_kind()` fully removed from `00_core.dag` | 2026-03 |
| RuntimeBridgeMethod enum | String-keyed bridge dispatch replaced with closed enum in core; round-trip through `runtime_bridge_method_name` remains (P1.10) | 2026-03 |
| L1 centralized predicates | `node_is_optional`, `node_is_map`, `node_is_container` and type predicates live in `04_types.dag` (`infer_types`); emit imports them; `classify_type_structure` replaces direct `.connective` reads in emit | 2026-03 |
| Rc policy extraction | `type_needs_rc`, `data_lookup_needs_rc_wrap`, `rc_wrapped` removed from core/reconcile/shared emit; live only in `05_emit_rust.dag` | 2026-03 |
| Kernel types single authority | `kernel_types` in `00_core.dag` is the only source; deleted `kernel_type_names()`, `is_primitive_name()`, `build_primitive_set()` | 2026-03 |
| Complexity match cost | `MatchCostAccum` in `cost_of_expr`; single pass over match arms (no 2^depth re-evaluation) | 2026-03 |
| Resolve bounded OOM | `resolve_node_bounded` stops re-resolving already-resolved lookups; trusts topological binding order | 2026-03 |

---

## Current State (2026-03-23 Audit, reconciled with branch review)

**Bootstrap note:** On this tree, `cargo test -p v2-compiler-tests --features v1-bootstrap v2_strict_compile_diagnostic_count -- --ignored` and `v2_bootstrap_fixed_point` **fail**: stage0 compile of v2 `.dag` sources reports 44 errors (`if` branches resolve to incompatible list element types across infer/parse/resolve/complexity). Workspace tests excluding `v2-compiler-tests` pass. Re-run the ignored gates after stage0 self-compile is green again.

**Root-cause audit (2026-03-23):** All ~66 live invariant violations
trace to three root causes. Fixing root causes eliminates downstream
symptoms structurally; fixing symptoms individually is whack-a-mole.

| Root Cause | Violations | Core issue |
|---|---:|---|
| **I: Type nodes structurally incomplete** | ~32 | Parameterized types sometimes lose children (bare `leaf_node(name: "Map")` instead of `map_node(key:, value:)`). Downstream: `normalize_type_name` heuristic, leaf-vs-structured comparison, emit `"_"` placeholders, `classify_type_structure` in emit, Go `interface{}` holes. |
| **II: Error/Dynamic are names, not structure** | ~18 | Inference failures are smuggled through the type namespace as nodes named `"Error"` or `"Dynamic"`. Downstream: `node_type_equals` treats Error==anything, emit string-checks for error types, cascade suppression by name, permissive type compatibility. |
| **III: Divergent inference paths** | ~16 | `ExprCall` and `ExprMethodCall` compute the same operations through independent code paths with different logic. Downstream: duplicated map/flat_map/fold typing, asymmetric `map_insert`/`map_merge` handling, 4x bridge method name maps, inline string method checks bypassing classifiers. |

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

Scripted audit via `scripts/l1-ratchet.sh`. Current breakdown by
category (2026-03-23):

| Category | Count | What the compiler still "knows" |
|----------|-------|----------------------------------|
| `.connective` direct access | 74 | Product vs coproduct read from Node field |
| `Conj` / `Disj` references | 68 | Connective shape matching (includes parse, which must produce them) |
| Type constructors | 129 | `leaf_node`, `optional_node`, `container_node`, `tuple_node`, etc. |
| `node_is_*` predicate calls | 82 | Centralized type-specific dispatch helpers |
| Type-name comparisons | 17 | `.name == "Optional"`, `"Map"`, `"Dynamic"`, etc. |
| `classify_type_structure` calls | 22 | Structural classification (replaces raw `.connective` reads in emit) |
| `builtin_type_kind()` calls | 0 | **Deleted** |
| `BuiltinTypeKind` references | 0 | **Deleted** |

Progress since last audit: `BuiltinTypeKind` enum and `builtin_type_kind()`
are fully deleted. `classify_type_structure()` replaces direct `.connective`
reads in emit. `node_is_optional`, `node_is_map`, `node_is_container` are
centralized in `04_types.dag` (`infer_types`, imported by infer and emit). Type-name comparisons
dropped from 62 to 17 through centralization.

The `node_is_*` count rose from 43 to 82 because scattered inline checks
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
| 1 | Phase 1 | Fix regressions, fix root causes (InferredNode wrapper, normalization stage, path deduplication), continue L1 dissolution toward name opacity | Regressions fixed; R.C. II (`InferredNode`) landed; R.C. III (normalization + dedup) landed; P1.10, P1.12 done; scrambled-name tests exist |
| 2 | Phase 2 | `gist` end-to-end through emitted Rust | `gist` builds and runs correctly |
| 3 | Phase 3 | Compile bundle, ownership/artifact wiring, and v1 retirement | v2 can compile everything v1 still matters for |
| 4 | Phase 4 | Shared emit spine, generated tests as projections, DAG backend boundary | New backend = language facts + compiler-owned adapter, with no shared-core changes |
| 5 | Phase 5 | Remaining convergence work after bootstrap shape is stable | One `Node`-centric internal model across compiler structure |

Important clarifications:

- Phase 1 is the only intentionally overlapping phase. **Diagnostics are
  at 0.** Regressions (R1-R4), root cause fixes (P1.9, P1.14, P1.15),
  and invariant items (P1.10, P1.12) block Phase 2. L1 dissolution
  continues in parallel after the blockers are closed.
- The three root causes (I: incomplete types, II: error-as-name, III:
  divergent paths) are the organizing principle for Phase 1. Each root
  cause fix eliminates its downstream violation cluster.
- `M*`, `R*`, and `S*` are support structures for this phase order, not
  competing schedules.

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
| R2 | Anonymous record tuple index: hardcoded 0-3, falls back to `"0"` for index >= 4 (`emit_rust` 1574-1578) | I | Emit `compile_error!()` for index >= 4 |
| R3 | Duplicate map/flat_map/fold type refinement: ~40 lines in both `ExprCall` (1765-1803) and `ExprMethodCall` (1919-1935) paths | III | Extract shared helper; both paths call it |
| R4 | `map_insert` key type hardcoded `"String"` (`infer` 1781) | III | Read key type from receiver's map children |

#### Tier 1: Immediate invariant fixes (< 1 day each)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.12 | Emit catch-all fail-closed | Quick | `emit_typed_item` catch-all emits `compile_error!()` instead of `// unhandled` |
| P1.10 | Collapse parallel bridge name maps | **Blocking** | Delete `runtime_bridge_method_name` (dead in core); collapse 4 parallel enum→string tables to one per-target rendering without shared string intermediary |
| P1.13 | Dead code cleanup | **Done** | Dead functions removed from `00_core.dag` and `complexity.dag` |
| P1.11 | Delete speculative artifact types | **Done** | Only consumed plan types remain in `artifact.dag` |

#### Tier 2: Root cause fixes (structural, each eliminates a violation cluster)

| ID | Item | Root Cause | Status | Notes |
|----|------|-----------|--------|-------|
| P1.9 | `InferredNode` wrapper | II | **Blocking** | Introduce `InferredNode = Resolved { node } \| CompilerError { message, span }`. Unify `Error` and `Dynamic` into `CompilerError`. Inference returns `InferredNode`, not `Node`. Eliminates ~18 downstream name-checking violations. Delete `node_is_error_type`, `node_is_dynamic`. |
| P1.14 | Normalization stage | III | Planned | New pass between resolve and infer. Unifies `Call`→`MethodCall` bridging, populates structural properties from `.dag` declarations, enforces arity completeness. Eliminates the duplicated inference path cluster. |
| P1.15 | Deduplicate inference paths | III | Planned | After P1.14, a single code path handles each semantic operation. Shared `refine_collection_result_type` helper. `map_insert`/`map_merge` handled uniformly. |

#### Tier 3: L1 dissolution (toward name opacity)

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
| P1.2 | Infer cleanup via data tables | **Partial** | Emit metadata extracted. Method handling deferred to P4. |
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

### P1.14 Design: Normalization Stage

New pass: `parse → resolve → **normalize** → infer → emit`

Job:
- Unify `ExprCall`→`ExprMethodCall` for known method patterns (the
  bridge rewrite that currently lives inside `infer_expr`)
- Populate structural properties from `.dag` type declarations
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
- Parallel bridge name maps collapsed (P1.10)
- Emit catch-all fail-closed (P1.12)
- Scrambled-name tests exist and pass for at least the inference layer
- `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` passes
- Fixed point still holds after every structural change
- Phase 2 may start once regressions, P1.9, P1.10, P1.12, and P1.14 are done,
  even if the L1 ratchet is not yet at 0

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
```

That bundle comes out of `compile.dag` plus the Rust emitter.

### Phase 2 Exit Criteria

- `cargo test -p v2-compiler-tests v2_gist_full_pipeline -- --ignored` passes
- The emitted gist crate builds and runs in dry-run mode
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
| P3.1 | Verify parity with remaining v1 paths | **Done** | Two root causes identified: tuple field naming (fixed), if-branch type unification (fixed). Remaining 197 diagnostics are field access resolution issues. |
| P3.2 | Ownership wiring + authoritative compile bundle | In progress | `compile_sources` now returns `complexity`, `ownership`, and `artifact_plan`, and emit dispatch follows the planned artifact target; unsupported obligations/reporting still need consolidation |
| P3.3 | Artifact planning above emit | In progress | Default single-artifact planning now runs between infer and emit through the real artifact contract. Speculative boundary types (`BoundaryContract`, `verify_boundaries`, `ArtifactReport`) deleted in P1.11; re-add only when a real consumer lands end-to-end. Real partitioning and per-artifact orchestration remain. |
| P3.4 | Runtime shim dissolution | Planned | Move the remaining v1 runtime shim pieces into `.dag` runtime templates |
| P3.5 | Feature-gate v1 | **Done** | v1 crates gated behind `v1-bootstrap` feature; `cargo test -p v2-compiler-tests` runs 0 tests without feature |

### Key Decisions for Phase 3

- The compile result stops being just `files + diagnostics`
- Ownership becomes a first-class pipeline output, not a side analysis
- Artifact planning becomes part of the real compile flow, not a side
  module with stringly targets
- Unsupported proof or validation obligations must surface explicitly

### Phase 3 Exit Criteria

- The compile bundle has one authoritative typed shape
- ownership is included alongside complexity in the pipeline output
- artifact planning runs between infer and emit in the primary compile path
- v1 is no longer required for normal compilation

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
| P4.1 | `LanguageSpec` becomes the single authority | Planned | Shared emit already imports extdep language tables; remaining duplication must collapse into one contract |
| P4.2 | Shared emit fold + target adapters | Planned | Highest-risk refactor; Rust/Python/Go still own full tree dispatch today |
| P4.3 | Generated tests as first-class projection | Planned | Preserve the current Rust path while generalizing the contract |
| P4.4 | DAG backend/runtime boundary | Planned | Add canonical DAG artifact and keep execution downstream |
| P4.5 | Typed backend plumbing and CLI surface | Planned | Backend selection should stop being stringly |
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
- Generated tests are still mostly a Rust-specific path.

### Phase 4 Exit Criteria

- No backend owns a whole-tree `ExprData` dispatcher
- No backend owns a separate whole-tree TCO walker
- `LanguageSpec` is the single authority for language facts
- Generated tests are first-class artifact outputs
- The DAG backend emits a canonical artifact without embedding an
  interpreter in the compiler stages

---

## Phase 5: Convergence (L2 Preparation)

**Gate:** one `Node`-centric internal model flows through the compiler,
with the naming cleanup already landed and the bootstrap architecture
stable enough to make the deeper dissolutions worth doing.

This phase is intentionally later. It should happen after the naming,
pipeline, and shared emit boundaries stop moving.

### Phase 5 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P5.0 | `kind_tag` string dispatch elimination | Planned | `02_parse.dag` dispatches on `TokenKind` through ~200 string comparisons via `kind_tag()`. Replace with structural match on `TokenKind` enum. Prerequisite for P5.1. |
| P5.1 | Token dissolution | Planned | Replace `Token` / `TokenKind` structures with `Node` compositions |
| P5.2 | Module/import dissolution | Planned | Dissolve `Module`, `Import`, and `ImportNames` into `Node` compositions |
| P5.3 | Diagnostic / compile-output dissolution | Planned | Dissolve `Diagnostic`, `Severity`, `CompileResult`, and `TextFile` where it is still valuable |
| P5.4 | Service/support type dissolution | Planned | Verify which service-layer types still need to move |
| P5.5 | Residual semantic enum cleanup | Planned | Move remaining compiler-only semantic types toward `.dag` or `Node`-based representation where appropriate |

### Phase 5 Exit Criteria

- Target filenames from M1 are fully normalized
- Compiler-internal structure is consistently `Node`-centric
- Each convergence step survives re-bootstrap and fixed-point verification
- The compiler is in a clean place to start real L2 work

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
| General generic syntax | Special-cased `Result` / `Option` is enough for bootstrap scope |
| Full linear type checking | Ownership proof work has started, but full proof remains beyond the current migration |
| Widen V5 | The conservative version covers current hot paths |

### Compiler Improvements

| Item | Why deferred |
|------|--------------|
| Anonymous record target resolution | Must fail closed, but is not blocking active phases |
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

#### Root Cause III: Divergent inference paths (~16 sites)

Dissolved by: P1.14 (normalization) and P1.15 (deduplication).

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
| V2 non-bootstrap | `cargo test -p v2-compiler-tests --features v1-bootstrap` | After every change |
| Diagnostics ratchet | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_strict_compile_diagnostic_count -- --ignored` | End of Phase 1 |
| Fixed point | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_bootstrap_fixed_point -- --ignored` | After any `.dag` change that affects bootstrap output |
| Gist pipeline | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_gist_full_pipeline -- --ignored` | End of Phase 2 |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | After any `.dag` change (goal: 0) |
| Scrambled-name tests | `cargo test -p v2-compiler-tests v2_scrambled_name_inference` | After P1.16; verifies name opacity |

**Scrambled-name test design:** Rename all type names to arbitrary strings
(consistently across declarations and references), run through inference,
verify identical structural decisions. If inference depends on `"Map"`
being called `"Map"`, this test breaks. Scoped from infer onward — parse
and resolve legitimately work with real names. Inference receives nodes
with opaque names and no name registry.

Manual Phase 2 smoke still exists in addition to the automated test:
build the emitted gist crate and run it in dry-run mode. There is not yet
a dedicated `v2_gist_end_to_end` test in the tree, so the roadmap should
not pretend that one exists.

**Review-queue discipline:** Prefer **scoped commits** (one invariant or
theme per commit) on automation branches, per `CLAUDE.md`, so CI and
blame stay attributable when diffs are large.
