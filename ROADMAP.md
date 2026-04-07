# gunbc Roadmap

## Architecture

Two substrate primitives: **Node** and **Edge**. Everything else —
types, truth values, cardinality, product/coproduct — is compositional
modeling in `.dag`. Languages are coercion targets. Testing is
compilation.

**Bounded kernel invariant:** Node is the only recursive semantic
authority in the compiler IR. All durable recursive structures are
Node trees — recursion lives in the data (children list), not in
type definitions. Non-Node types are flat discriminants and data
tables. This makes descent provable by construction: any function
that walks Node.children is structurally bounded.

Full thesis: [docs/architecture.md](docs/architecture.md)
Compiler laws and coercion model: [docs/compiler-laws.md](docs/compiler-laws.md)
Coercion design (algebra-keyed inhabitants): [docs/coercion-design.md](docs/coercion-design.md)
Testing strategy: [docs/testing-strategy.md](docs/testing-strategy.md)
Invariant enforcement: [INVARIANTS.md](INVARIANTS.md)
Modeling guidelines: [MODELING.md](MODELING.md)

## Critical Path

```
            ┌─ Lane 1: M2 (boundary sufficiency) ──┐
M1 COMPLETE─┤                                       ├→ M4 (structural identity)
Bootstrap D ┼─ Lane 2: CG (codegen correctness) ───┘       └→ M5 → M6 → M7
  COMPLETE  ├─ Lane 3: CX (164 → 0)
            ├─ Lane 4: LS (language spec modeling) ─→ CG-3 parameterized emission
            └─ PERF (continuous — parallel to all lanes)
            CM: cross-cutting design lens (informs all lanes) → src/v2/CM.md

Lane 1 owns: 04_infer, 04_types, 02_parse, 04_method
Lane 2 owns: 05_emit, 05_emit_rust, 04_emit_info, ownership, languages
Lane 3 owns: complexity, dsl/std/
Lane 4 owns: dsl/extdeps/languages/{rust,go,python}/ — spec-sourced data
CM informs how Lanes 1-3 approach work; does not own files separately
PERF owns: performance ratchets, bootstrap convergence tests, timing budgets
M4 follows Lanes 1+2 (needs structural facts + clean render path)
LS follows CG-3 (needs parameterized emission to consume spec data)
```

---

# Layer 1: Active Gates

## Bootstrap Status

| Stage | Status | Gate | Notes |
|-------|--------|------|-------|
| **A** | GREEN | 0 diagnostics | PR #264 |
| **B** | GREEN | 0 emitted-Rust errors | PR #307. `bootstrap_stage0_to_stage1` still `#[ignore]` |
| **C** | GREEN | regen binary self-compiles | PR #308. 1 perf-only bootstrap patch: `dag_syntax_spec` cache |
| **D** | GREEN | `regenerate-stage0.sh && git diff --exit-code` | PR #308. Freshness gate blocking in CI. `render_node_type` fuses Node → String directly (TypeRendering dissolved PR #331) |

**Bootstrap D** = regenerated code replaces committed stage0 with zero
manual patches, AND the regenerated binary produces identical output
when it self-compiles (fixed point convergence). **Blocks all other
lanes from editing stage0 Rust directly.**

Note: "zero manual patches" means zero patches to *generated* files.
All stage0 content is 100% generated — zero hand-maintained files.

Previously eliminated:
- PR #316: v2_compiler_infer_method.rs, std_types.rs,
  extdeps_languages_rust_emit.rs, v2_compiler_tokenize.rs,
  extdeps_languages_dag_syntax.rs — via thread_local! caching in emitter
  and `chars_to_string` runtime function for O(1) tokenizer substring.
- v2_coercion.rs — replaced by `src/v2/coercion.dag`, coercion registries
  now generated from .dag type checkpoint/inhabitant data declarations.
- v2_rt.rs — runtime templates already modeled in `runtime_rust.dag`;
  `rust_runtime_source()` produces the complete file. Emitter includes it
  via `emit_v2_rt_module()`.
- main.rs — CLI entrypoint with FF-9 import resolution and diagnostic
  rendering, modeled as emitter string templates in `05_emit_rust.dag`.
  `emit_compile_match_arm()` emits the full Compile subcommand with
  `--source-root` transitive import resolution.
- compiler_tests.rs — test harness modeled in `compiler_tests_rust.dag`;
  `compiler_tests_source()` produces the complete file. Emitter includes
  it via `emit_compiler_tests_module()` when self-compiling.

## CI Gates

| Gate | Command | Status |
|------|---------|--------|
| Lint | `cargo clippy --workspace -- -D warnings` | GREEN |
| Tests | `cargo test -p v2-compiler-tests` | GREEN (316 pass, 0 fail, 44 ignored) |
| Full DSL | `full_dsl_compiles -- --ignored` | GREEN (93 dsl + 29 v2) |
| Diagnostic ratchet | `strict_compile_diagnostic_count -- --ignored` | 314 (0 self-compile diagnostics) |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | 37 (target: 0) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

## Ratchet Counts

| Metric | Current | Target | Notes |
|--------|---------|--------|-------|
| Self-compile diagnostics | 314 | 0 | All indirect-recursion complexity violations |
| L1 type knowledge | 37 | 0 | Down from 70; name-based workarounds tracked for M4 |
| Complexity violations | 0 | 0 | GREEN — CostUnknown deleted, all costs concrete (PR #336) |
| Emitted Rust errors | 0 | 0 | GREEN |
| DSL complexity ratchet | 2 | 0 | stack_size + fold_stack (deferred to CX lane) |

## Bootstrap Dependency Chain

```
.dag source ──(v2-compiler)──▶ stage0 .rs ──(cargo/rustc)──▶ v2-compiler binary
     ▲                                                              │
     └──────────────────────────────────────────────────────────────┘
```

- **Source of truth:** `.dag` files. Stage0 `.rs` is a derived artifact.
- **Cycle-breaker:** Committed stage0 allows fresh clones to bootstrap.
- **Fixed-point:** `regen pass N == regen pass N+1`. Two-pass bootstrap
  required when the emitter changes its own output.
- **External dependencies:** cargo, rustc (opaque transforms, not modeled).
- **Hand-maintained files (0):** All stage0 content is generated.
  See Bootstrap Status for elimination history.

**Merge workflow problem:** Stage0 `.rs` files are derived, but git
treats them as text and produces line-level merge conflicts. These
conflicts carry zero information — the only correct resolution is
regen. Fix: `.gitattributes` marks generated stage0 files as `-merge`
(no line-level merge), CI freshness gate ensures committed stage0
matches `.dag` source. Merge workflow becomes: resolve `.dag` conflicts
→ accept either side of stage0 → regen → commit.

This dependency chain is currently implicit (shell scripts + convention).
Modeling it as `.dag` types is tracked under BP-1 (Layer 3).

## Bootstrap Health Rules

Clean-repo workflow:
1. `cargo check -p v2-compiler`
2. `cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored --nocapture`
3. `cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored --nocapture`
4. Run `./scripts/regenerate-stage0.sh`
5. Require `git diff --exit-code src/v2/stage0`

Stabilization rules:
- No manual `src/v2/stage0/` edits once regeneration is green.
- CI gate: `check-stage0-freshness.sh` (regenerate → diff → empty).
- Stage0 generated `.rs` files use `-merge` in `.gitattributes` (no line-level merge).
- CX gate disabled in both stage0 and `compile.dag` — emission not blocked by complexity violations. Re-enable when CX violations reach 0.

## Reviewer Root Cause Analysis (2026-04-03)

Two stacked failures, not one. First, some target-language facts are
missing (TLC-1 through TLC-4). Second, facts that exist upstream are
lost before emit, so emission compensates with heuristics. The deepest
bug is upstream information loss, not the leaf workaround. Every
remaining emit workaround is either an inference-boundary bug (M2) or
a missing codegen authority (CG) — never a standalone emitter patch.

---

# Layer 2: Root-Cause Tracks

Four named architectural problems. Each has one root cause and one
definition of done. Lanes 1–3 run in parallel; M4 follows.

## M2: Boundary Sufficiency (Lane 1)

**Root cause:** The resolution→emit boundary does not carry enough
structure. Emit compensates with heuristics. Every remaining workaround
is an inference-boundary bug or a missing upstream fact.

### Structural error propagation

`child_inferred_or_empty` fabricates Unit on inference failure instead
of propagating error state. `node_inferred_to_outputs` builds outputs
from fabricated types. Highest-confidence correctness bug (reviewer
2026-04-02).

- [~] `child_inferred_or_empty` no longer fabricates Unit — `Untyped` returns the child's own structure (partial: reuses raw structure as type authority, not explicit error propagation)
- [x] `node_inferred_to_outputs` refuses error-typed children (fail-closed) — all-or-nothing gate via `rt_node` check; returns `[]` if any child is not `Typed`

### Incomplete parameterization and bidirectional inference

One root cause behind multiple symptoms: type information does not flow
bidirectionally through inference boundaries, so downstream stages see
incomplete types and fabricate.

Symptoms:
- `bare_map_node()`/`bare_list_node()` fabricate partial structure
- Fold accumulators under-resolved (magic names Unit/Dynamic/Error)
- Higher-order method templates collapse callable structure into ReceiverSelf
- `expected` parameter threaded to `infer_expr` but incomplete for formal params
- Callback shapes (`fn(Acc, T) -> Acc`) synthesized at inference time, not declared

Open items:
- [x] Incomplete parameterized types rejected at normalization — `container_expected_arity` returns `Int?` (fail-closed: unknown names → None, no false positives on operations sharing container names)
- [~] `bare_map_node`/`bare_list_node` eliminated or gated before emit — normalization catches authored bare containers; `empty_map()` with non-keyed expected now diagnosed. Expected threading covers: let body, list elements, lambda body, return, tail-return, if branches, match arms, record literal fields (struct field types), function arguments (parameter types). Fold-init path now receives outer expected when available. Remaining gap: fold-init in non-tail let value position has `None` expected — circular dependency (need value type to give expected, but value inference produces the type). Requires bidirectional inference with unification. Self-compile diagnostic count: 0
- [x] Thread `expected` to formal params at matching positions — over-arity args no longer receive synthetic expected types; non-callable expected boundary overload remains open
- [x] Refine fold accumulators structurally via `is_fully_resolved` — recursive: checks TypeVariable on self, collection arity, and recurses into all children
- [x] `CallableOf` in `AlgebraTypeTemplate` for higher-order signatures

### BRIDGE fabrication progress (83 → 0)

BRIDGE count reduced from 83 to 0 real (4 emitter template strings remain).
Phases: contradictory predicate fix (83→29), Phase C structural resolution
(29→28), typed seed helpers (28→21), match/if arm expected propagation
(21→16), receiver patching (16→6), final typed helpers + if/else
bidirectional (6→0). Post-inference unification replaced all heuristic
patches (PR #334 review fixes).

### Compositional type parameter resolution (major cleanup landed)

Phases A-D landed (PRs #325, #332, #334). Compensation infrastructure
deleted, method-name dispatch eliminated. Remaining work: import scoping
(FF-9 / declaration-driven loading), ExpectedContext modeling for edge
cases, Tier 2.5/3 algebra fidelity.

- [x] Phase A: TypeVariable preservation (`algebra_child_or_placeholder`)
- [x] Phase B: TypeVariable rendering in emit (fail-closed: `compile_error!`)
- [x] Phase C: Structural return type resolution via template unification
  (`resolve_type_variables_from_template` replaces `refine_collection_result_type`)
- [x] Phase D: Deleted compensation infrastructure
  (`refine_collection_result_type`, `substitute_algebra_result`,
  `bridge_placeholder_type_names` — ~280 lines removed)
- [x] `AlgebraTypeVariable` variant replaces `is_algebra_placeholder_name`
- [x] Templates enriched with `CallableOf` contracts (self-describing)
- [x] Bare-container ReceiverSelf enrichment from arg-derived bindings
- [x] Post-inference unification for match/if arms (order-independent)
- [x] ReceiverSelf structural witness for receiver type patching
- [x] Narrow T/K/V placeholder filtering restored for std.types imports

**Result:** 0 method-name dispatches for type resolution (was 6).
Emit is fail-closed: TypeVariable → `compile_error!`.
Seed helpers and post-inference unification are tactical cleanup —
the final compositional model requires ExpectedContext + FF-9.

### Remaining BRIDGE fabrications (0 real)

All real BRIDGEs eliminated. The 4 remaining occurrences in stage0 are
emitter template strings (BRIDGE message formats for code the compiler
itself compiles — not type resolution failures in the compiler).

Approach: typed seed helpers provide `Map<K,V>` context at call sites,
post-inference unification resolves match/if arms order-independently,
ReceiverSelf witness gates receiver patching structurally.

**Design direction — ExpectedContext sum type (deferred):**

Replace `expected: Node?` with `expected: ExpectedContext?` to
distinguish different expectation sources. Not needed for current
BRIDGE count (already 0), but would improve inference for edge cases
where expected type is available but not threaded (e.g., nested
`empty_map()` in complex expressions).

### Unified container child encoding (PR #339, in progress)

**Root cause:** Container type nodes (List, Map, Set) encode their type
parameters as bare children (child IS the type, `inferred: none`), while
struct fields use field-style encoding (child has a name and `inferred:
Resolved(type)`). This dual encoding forces every consumer to handle both
cases, producing `child_inferred_or_name` and scattered `inferred == none`
guards.

**Authority:** The resolve phase is the normalization point. Parser creates
type expressions with bare children (pre-resolve). Resolve converts to
field-style children with named type parameters (T, K, V) from
`container_type_param_names` in `std/types.dag`. Post-resolve consumers
extract via `child_type_node` (bridge handles both encodings during
transition).

**Completed (PR #339) — bridge work, not endpoint:**
- [x] `container_type_param_names` data table in `std/types.dag`
- [x] `container_node`, `map_node`, `bare_map_node` produce field-style children
- [x] `child_type_node` bridge helper (handles both bare and field-style)
- [x] `container_param_name` returns `String?` (fail-closed, no fabricated default)
- [~] Consumer updates: major readers updated to `child_type_node` — lookup,
  items, patterns, emit_info, emit, emit_rust, access, infer, resolve.
  Remaining readers still use raw `rt_type` in emit/emit_go/emit_python
  sites and algebra template paths. These work post-resolve (Resolved
  wrappers guarantee correct extraction) but bypass the bridge.
- [x] All 314 tests pass, 0 self-compile diagnostics, L1 ratchet unchanged
- [ ] Endgame: all post-resolve readers use `child_type_node`, hardcoded
  "T"/"K"/"V" replaced by data table reads, wrapper construction via
  single helper (not inline Node literals)

**Completed (bootstrap convergence):**
- [x] `substitute_type_slots` recurses into `inferred` on field-style
  children — generic type instantiation works through wrappers
- [x] `has_nested_records_node` extracts via `child_type_node` — data
  declarations with `List<SomeStruct>` correctly use JSON emission
- [x] Bootstrap convergence: two-pass fixed point verified
- [x] `child_inferred_or_name` dissolved — 7 call sites → `rt_type(n: ch)`,
  function deleted
- [x] `__EMIT_BUG_ANONYMOUS_FIELD__` removed — dead code

**Modeling gaps exposed by edge-case analysis (next PR):**

Three reviewer-flagged edge cases share a common root: the `inferred`
field carries implicit invariants that should be structural facts.

1. **`inferred` has multiple semantic roles with different error contracts.**
   On expressions: can be CompilerError/TypeVariable/Untyped (all meaningful).
   On struct field children: should always be Resolved.
   On container wrapper children: should always be Resolved.
   These roles use the same `InferredNode?` type, so `rt_type` treats them
   all with the same Unit fallback — correct for emission, wrong for type
   reasoning where errors should propagate or be impossible by construction.

2. **`rt_type` is one function for multiple operations.**
   "Extract for emission" (Unit fallback OK) vs "extract for type reasoning"
   (error = bug, not fallback) vs "extract parameter binding" (always
   Resolved by construction). A typed model would have separate accessors
   with different error semantics.

3. **Pre-resolve vs post-resolve is a convention, not a structural fact.**
   "Post-resolve children use field-style encoding" is enforced by
   producer convention. `child_type_node` exists as a bridge because
   the structure doesn't distinguish resolved from unresolved nodes.
   If resolve's output were structurally marked, consumers wouldn't need
   the bridge — the wrong question would be unaskable.

**Direction:** These are instances of the same CM pattern — implicit
pipeline-stage invariants that should be structural facts on the Node.
Candidate modeling: `inferred` role (expression/field/parameter)
distinguishable from Node structure, or resolve-boundary marking that
makes pre/post-resolve structurally distinct.

### Acceptance

No fabricated type args, no generic/wrong fallback return types, no
error-typed children reaching emit. BRIDGE fabrication count: 0 real.
Ownership and clone correctness tracked under CG lane.

---

## CG: Codegen Correctness + Optimality (Lane 2)

**Root cause:** Codegen decisions (type rendering, sharing, ownership,
clone, expression semantics) are scattered across emitter heuristics
instead of derived from structural authorities. This produces
correct-but-suboptimal code and blocks new backends.

**Status:** TypeRendering type dissolved (PR #331). `render_node_type`
fuses Node → String in one pass, consuming coercion data directly.
Clone semantics unified in SharingStrategy (CloneTemplates removed).
TLC-1 complete: ExprVar → ExprCall normalization at inference time
(PR #329). TLC-2 complete: `SimpleMethodSpec` co-locates template +
wrapping flag, both maps derive from single authority. TLC-3 done: all
three backends dispatch per collection type via `IndexingSemantics`
templates. TLC-4 partial: `emit_let_binding_annotated` infrastructure
exists but disabled for bootstrap convergence (.dag type inference
produces `()` for empty collection element types). Higher-order method
templates (filter/any/all/flat_map) data-driven via
`HigherOrderMethodSpec`.

One root cause, two symptoms:
1. Facts that exist upstream are lost before emit — emission compensates
   with heuristics (411 `shared_types` threading sites, 28 hardcoded
   `.clone()` decisions in the Rust emitter alone)
2. Target-language facts are missing entirely — expression-level gaps
   (TLC-1..4) force per-backend special cases

### Completed (FF-1: ownership analysis)

- [x] Ownership analysis wired into Rust emitter: fan-out=1 function params move instead of clone
- [x] VarBindingKind flows through ownership analysis to emission
  - Let-binding moves: BindingUsage carries binding_kind, is_owned_local filter replaces params set
  - Match-bound variable handling: MatchBoundBinding variant excludes &T refs from movable set

### CG-1: Authority consolidation

Single source of truth for each codegen decision. Kill dual authorities
and heuristic fallbacks.

**Type rendering**

`render_node_type` fuses Node → String in one pass (PR #331).
`TypeRendering` type dissolved from `04_emit_info.dag`. `emit_node_type`
delegates to `render_node_type`. Coercion data consumed directly:
`coerce_primitive_type`, `coerce_container_template`, `target_callable`.

- [x] Delete `emit_node_type_rc` / old type rendering path
- [x] `emit_primitive_type` deleted (subsumed by `coerce_primitive_type`)
- [x] `TypeRendering` type + `build_type_rendering` + `render_type` dissolved
- [x] `render_node_type` fused function (Node → String, one pass)

**Sharing and ownership**

- [x] Sharing derived from `is_type_constant` + `TypeSummary` (no Rc wrap for fixed-width carriers)
- [x] `build_rc_types` renamed and reorganized as `build_shared_types` in Rust emitter
- [x] `ValueContext` deleted — `has_fn_fields` moved to `TypeSummary`, sharing
  computation lives in `EmitGraphInfo.shared_types` field (still Rust-emitter-local;
  authority has not moved upstream — deferred to coercion engine)
- [x] `is_type_constant` in 05_emit_rust.dag consulted by `build_shared_types`
- [x] Clone semantics in LanguageSpec (28 hardcoded `.clone()` → data-driven)
- [x] Explicit parent-enum ownership facts through resolve/infer/emit
- [x] Phase B cleanup: rename `rc_types` parameter → `shared_types` across ~423
  occurrences in emit pipeline (mechanical, no semantic change)

### CG-2: Expression-level gap closure (TLC-1..4)

Each gap is a missing LanguageSpec fact. Closing them unblocks new
backends.

- [x] **TLC-1: Call syntax / reference distinction.** Inference normalizes
  ExprVar → ExprCall for zero-arg function references (PR #329). All three
  backends emit `name()` via existing ExprCall handling. `is_zero_arg_callable_ref`
  dissolved (L1 33→32). Imported zero-arg function refs untested.
- [x] **TLC-2: Runtime bridge signature derivation.** Runtime helper
  return types and wrapping conventions must derive from the same
  type/coercion authority as emission. `wraps_result` field added to
  `RuntimeFunction` registry (PR #331); `rt_wraps_result()` derives
  from registry. `SimpleMethodSpec` type co-locates template + wrapping
  flag for method templates (same pattern as `HigherOrderMethodSpec`).
  Both `rust_method_templates()` and `rust_method_wraps_result()` now
  derive from `rust_simple_method_specs` — single authority, no parallel maps.
- [x] **TLC-3: Indexing / character access semantics.** `IndexingSemantics`
  in LanguageSpec — per-collection-type templates (list/map/string index/slice).
  All three backends dispatch per collection type via shared
  `IndexingSemantics` data + `is_string_like`/`node_is_keyed_collection`
  (pre-existing; prior description was stale).
- [~] **TLC-4: Explicit annotation requirements.** `AnnotationRequirements`
  in LanguageSpec — let binding templates (inferred/annotated), lambda param
  templates (typed/untyped). Infrastructure exists (`emit_let_binding_annotated`)
  but disabled at all three Rust let-binding sites for bootstrap convergence:
  .dag type inference produces `()` for empty collection element types.
  Requires M2 inference improvement before re-enabling.

### CG-3: Parameterization

Make emission fully data-driven. Adding a backend = adding data.

- [x] Method templates: simple methods use `apply_named_template` with per-language
  `Map<String, String>` data. Templates are pure method syntax; Rc wrapping
  composed separately from sharing authority. Covers count/join/split/first/last/
  enumerate/chars/skip/take + higher-order (map/filter/fold/sort_by/any/all/flat_map).
- [x] Higher-order method templates: `HigherOrderMethodSpec` type in Rust extdeps
  with `method_name`, `inline_template`, `fn_ref_template`, `wraps_in_sharing`.
  Shared `emit_rust_higher_order_method` replaces 4 hardcoded emitters
  (filter/any/all/flat_map). Data-driven dispatch via method name lookup.
- [x] Transport/config: `TransportKind` enum + `classify_transport()` centralize dispatch; `ServiceFieldSet` + `compute_service_fields()` centralize field queries. Remaining per-backend rendering is inherent language differences (HTTP clients, shell runners, file I/O) — addressed by the 3→1 homomorphism.
- [ ] LanguageSpec completion — all target-language facts data-driven (see LS lane below)
- [x] TypeRendering dissolved — `render_node_type` consumes coercion data directly (PR #331)
- [~] 3 backends → 1 parameterized homomorphism (PR #338).
  **Phase 1-3 complete** (-206 .dag lines, -481 stage0 lines):
  shared expr wrappers (`emit_expr_var_shared`, `emit_expr_field_access_shared`,
  `extract_string_interp_parts`), shared typed handlers (`emit_typed_cast_shared`,
  `emit_typed_index_shared`, `emit_typed_slice_shared`), ExprData accessors
  (`expr_field_access_summary`, `expr_method_call_semantics`), dead code
  deletion (`emit_*_typed_bin_op/cast/index/slice`).
  **Phases 4-6 deferred** — blocked on depth/indent asymmetry (see below).
  **Review feedback** (recorded, not yet addressed):
  (a) Unify `SimpleMethodSpec` + `HigherOrderMethodSpec` into shared
  `MethodTemplateSpec` to reduce schema drift across method families.
  (b) Method names could be a structural enum (M4 direction) rather than
  strings for clearer algebraic dispatch across language extdeps.

### Depth/indent asymmetry (cross-cutting blocker)

Go renders sub-expressions at `depth: 0` and manually prepends
`make_indent(level: depth)` at the wrapper level. Python threads
`depth` through all recursive calls. This fundamental difference
pervades every handler and prevents clean parameterization of:
- TCO handlers (~10 functions per backend, ~130 lines potential)
- Method call dispatch (~4 functions per backend, ~80 lines potential)
- Block statement emitters (scope-threading + depth)
- `emit_typed_let`, `emit_typed_for_each`

**Resolution options:**
1. **Align Go to Python's strategy.** Go's `wrap_result` callback in
   `emit_shared_expr` already handles indentation for simple expressions.
   If Go switched to `depth: depth` recursion and removed per-wrapper
   `prefix` wrapping, the two backends would become structurally
   identical. Risk: changes Go's emitted output formatting (whitespace
   only, not semantics).
2. **Add `DepthMode` to shared layer.** Shared handlers take a depth
   resolver: `sub_depth: fn(Int) -> Int` where Go passes `d => 0` and
   Python passes `d => d`. More complex but preserves current behavior.
3. **Accept current state.** -206 lines is the practical limit without
   resolving this asymmetry. The remaining ~210 lines of savings may
   not justify the design cost.

**LanguageSpec extensions needed** (once depth is resolved):
- `async_call_prefix: String` — Python: `"await "`, Go/Rust: `""`
- `TcoSyntax` — loop_open, loop_close, break_keyword, continue_str,
  stmt_terminator (data-driven TCO formatting)

### Coercion infrastructure (reference)

`render_node_type` reads coercion data from `.dag` declarations:
- [x] `TypeCheckpoint` (primitives) from language `types.dag` — live via `coerce_primitive_type`
- [x] `InhabitantDecl` (algebra containers) from language `types.dag` — live via `coerce_container_template`
- [x] `CallableRepr` (callable syntax) — live via `target_callable`

Shared schema in `std/coercion.dag`; per-language instances in
`extdeps/languages/{rust,python,go}/types.dag`. Design doc:
[docs/coercion-design.md](docs/coercion-design.md).

Remaining parallel authorities:
- [x] Copy/value semantics: `is_rust_value_type` reads `TypeCheckpoint.is_copy`
  (dissolved `rust_value_types` list). `is_simple_type_node` wraps `is_rust_value_type`
  (Copy check for data-def caching). `is_primitive_numeric_node` resolves to Rust
  target type and checks against `integer_types`/`float_types`/`"bool"` — narrower
  than Copy (excludes Unit; PR #333)
- [x] Dead code: deleted `try_target_primitive_type`, `target_primitive_type`,
  `target_container_template`, `is_value_type`, `emit_primitive_type` + unused imports
- [x] Runtime bridge wrapping: `wraps_result` field on `RuntimeFunction` registry
  (map_keys, map_values, append) and `SimpleMethodSpec.wraps_result` for method
  templates (split/enumerate/chars/skip/take). Both derived maps come from their
  respective single authorities — no parallel maps.

### Acceptance

For each target language, the compiler must prove:

1. **Correctness**: emitted code compiles and passes the full test suite
   with zero manual patches or escape hatches.
2. **Optimality**: no unnecessary clone, copy, or allocation — every
   sharing decision traces to a structural fact (TypeRendering,
   ValueContext, or LanguageSpec) and can be audited. Emitted code
   should be what a competent human would write by hand.
3. **Completeness**: every codegen decision derives from exactly one
   structural authority — no heuristic fallbacks, no dual authorities,
   no hardcoded target-language knowledge in the emitter.
4. **Backend independence**: adding a new target language requires only
   LanguageSpec + coercion data files, zero emission logic changes.

### Proof mechanism: structural coverage by construction

The .dag input language is decidable and Node-bounded, so the space
of structural forms reaching the emitter is finite. Correctness is
proved by construction over this finite algebra, not by post-hoc
validation of specific programs.

**Emission algebra.** The emitter's input space is the product of
`(NodeKind × TypeForm × Cardinality)` triples that survive type
checking. This set is enumerable from the .dag type definitions
themselves — it is the compiler's own structural vocabulary.

**Structural induction on Node depth.** Node is the only recursive
type, so the proof has two parts:
- *Base case*: every leaf form (literal, variable ref, constant) emits
  valid target code for all type forms.
- *Inductive step*: if children emit valid code, the parent node's
  assembly emits valid code. This holds when every assembly decision
  (clone, wrap, annotate) reads from a structural authority rather
  than a heuristic — which is exactly what CG-1 through CG-3 enforce.

**Exhaustive form coverage.** A test generator synthesizes one minimal
`.dag` program per element of the emission algebra and compiles it to
every target. This extends `full_dsl_compiles` from "these programs
compile" to "every structural form compiles." The generator is
derivable from the .dag type definitions, so new forms added to the
language automatically produce new test cases.

**LanguageSpec modularity.** Once every emitter decision reads from
LanguageSpec (CG-3), the proof becomes modular: prove LanguageSpec
covers every structural form → proves any backend using it is
complete. Adding a language = adding a LanguageSpec instance + proving
coverage of the same finite algebra.

No escape hatches in type rendering. No fabricated names. No post-hoc
passes to fix what construction should have prevented.

---

## LS: Language Spec Modeling (follows CG-3)

**Thesis:** Every target language has a spec. The spec constrains what
the emitter can produce. Model the spec as .dag data declarations in
`dsl/extdeps/languages/{rust,go,python}/`. The emitter reads the spec —
it never decides. The spec IS the invariant.

This is what .dag is made for: decidable, structural, authoritative
facts. A language spec is exactly that — finite rules about what
syntax is valid and what semantics it carries. Modeling it in .dag
means the compiler can prove it follows the spec by construction.

**Pattern:** For each language decision the emitter currently makes
via inline logic, find the relevant section of the language spec,
model it as data, reference the spec section in a comment, and have
the emitter consume the data.

### LS-1: Type cast rules

Emitter currently uses `is_primitive_numeric_node` (target-only check).
The Rust spec defines `as` validity as a relation `(source, target)`.

Ref: Rust Reference §8.2.4 "Type cast expressions"
https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions

Spec table (value-level casts only):
- Integer/Float → Integer/Float (numeric cast)
- bool/char → Integer (primitive to integer cast)
- u8 → char (u8 to char cast)
- Enumeration → Integer (enum cast)
- NOT valid: bool→float, integer→bool, integer→char (except u8)

Model: `CastCategory` enum + `CastRule` relation + `can_as_cast(from, to)`
lookup in `extdeps/languages/rust/types.dag`. Emitter calls lookup
instead of classifying inline.

Go equivalent: type conversion rules (all numeric conversions valid,
`int64(x)` syntax). Python: constructor calls (`int(x)`, `float(x)`).

### LS-2: Operator semantics

Which operators are valid for which types, and what syntax they produce.
Currently scattered across emitter logic.

### LS-3: Expression syntax

Statement vs expression distinction, block syntax, match exhaustiveness
requirements, semicolon rules. Each language has different rules.

### LS-4: Ownership and borrowing (Rust)

Move/copy/borrow rules. Currently heuristic in the Rust emitter.
The Rust Reference defines these structurally.

### LS-5: Visibility and module system

`pub`, `pub(crate)`, Go capitalization, Python `__all__`. Currently
hardcoded per-backend.

### Acceptance

Every emitter decision traces to a spec-referenced data declaration
in `dsl/extdeps/languages/`. Adding a new target language = modeling
its spec as .dag data. Regression tests auto-generated from the spec
data (each rule → one test).

---

## M4: Structural Identity (L1 = 0) — follows Lanes 1+2

**Root cause:** The compiler uses `Node.name` (a string) as semantic
authority. 27 type constructors + 10 name comparisons = L1 37.
Deletion requires declaration-driven identity and structural algebra.
CG render path cleaned (TypeRendering dissolved, PR #331). M2
expected threading in good shape (remaining gap: bidirectional inference).

**L1 name comparisons breakdown (10):**
- 4× Callable: `has_fn_fields` (2), resolve `n_is_special` (1), `render_node_type` (1)
- 3× Tuple: `render_node_type` (3)
- 2× Dynamic/Error: `pattern_subject` (2), counted as 1 line in resolve
- 1× List: `index_access` (positional indexing check)

**Structural properties needed to dissolve:**
- Callable: needs structural discriminator (only type using `params` for type parameters + `inferred` for return type). Tier 2.6 design: model callable as concept
- Tuple: needs either a dedicated connective or a flag. Currently indistinguishable from 2-field struct Conj without name
- Dynamic/Error: need structural markers for "unresolved/error type"
- List positional indexing: needs "supports positional access" algebra fact

### Lane 1: Declaration-driven algebra

Goal: compiler reads type/algebra facts from `.dag` declarations
instead of hardcoding them.

- Tier 1 (data tables → .dag): DONE
- Tier 2 (factor enrich_kernel_type): DONE
- Tier 2.5 (algebra bridge fidelity):
  - [x] `CallableOf` variant for higher-order callback shapes
  - [ ] Derive T/K/V type parameter names from algebra declarations
- Tier 2.6 (functional system modeling):
  - [ ] Model function application as a concept (apply/call vs function-value-ref)
  - [ ] Inference encodes "this is a call" in the IR node, not as a type-arity heuristic
  - [ ] Dissolves `rt.name == "Callable"` L1 violations (4 sites: has_fn_fields ×2, resolve, render_node_type)
  - Same pattern as iteration modeling (fold/descend/repeat): ad-hoc emit
    decisions are symptoms of a missing concept layer. Once the functional
    system is modeled, arity-based rendering questions disappear.
- Tier 3 (full structural algebra, requires FF-9):
  - [ ] Compiler reads type declarations + algebra edges at resolve time
  - [ ] Derive kernel/container identity from type declarations
  - [ ] CollectionKind bridge dissolves when method algebras land
  - [ ] 27 type constructor sites → 0

### Lane 2: Node.name deletion (D6)

Goal: delete `Node.name` field. Rendering uses `source_text_at`,
resolve uses structural identity.

- B3 (emit rendering): DONE
- B4 (resolve structural identity): accessor layer done, `node.name`
  still semantic authority underneath
- D6 open:
  - [ ] Migrate remaining emit sites (Python ~5, Go ~5, shared ~5)
  - [ ] Update ~256 Node constructions to drop `name:`
  - [ ] Migrate synthetic node identity to structural
  - [ ] Delete `Node.name` field + scrambled-name tests

Lanes share only `00_core.dag` (different functions, no conflict).

### Acceptance

`scripts/l1-ratchet.sh --check` reports 0. Scrambled-name tests pass
then deleted. `Node.name` field deleted.

---

## CX: Complexity Analyzer (Lane 3)

**Status:** Down from 315 → 164 (main) → 76 after PR #318 → 313 after PR #336.
Phase 1-2 complete (RecursionPattern deleted, all classifiers return LoweringTarget).
CX-N: var threading, type-directed dimension selection, algebra-to-dimension bridge.
0 violations — CostUnknown deleted, all costs concrete (PR #336).
Complexity analysis re-enabled in compile pipeline.
PR #336: soundness fixes, graph extraction, is_valid_proof, CostUnknown deletion.

**Root cause:** The analyzer maintains parallel heuristic classifiers
instead of consuming the structural facts already modeled in std/.
Violations are not analyzer bugs — they are missing facts about data
and operations. The path to 0 is modeling those facts, not extending
the analyzer with name-matching heuristics.

### Design principles

**Recursion is emergent.** Recursion is not a first-class concept to
model or analyze. It falls out of functions calling each other — just
like real programs don't "know" they're recursing. Only iteration
primitives (fold/descend/repeat) are modeled explicitly because they
are intentional developer constructs. The analyzer never tries to
"prove recursion terminates" — it computes the tightest bound it can
from structural facts about data and operations.

**All programs are bounded.** All data is ultimately quantifiable
(Bit/Word64). The analyzer reports HOW bounded, not WHETHER bounded.
If it can't derive a tight bound, the answer is Forever (2^63-1), not
a violation. Forever < infinity — it is "heat death of the universe"
scale, a concrete finite quantity. The system never deals with actual
infinity.

**No rejected patterns.** Every call pattern has a finite bound.
`self(same_arg)` → `repeat(Forever)`. The analyzer computes bounds;
it never rejects. CX violations mean "the analyzer couldn't derive a
bound from available facts" — the fix is providing the missing fact,
not adding a heuristic recognizer.

**Complexity classes are emergent.** Linear, quadratic, Forever are
themselves emergent properties of numbers and arithmetic. The
underlying number modeling is anemic for now but should improve. As
arithmetic facts become richer, complexity classes will emerge with
greater precision without analyzer changes.

**CX heuristics are CM gaps.** Every name-matching classifier in the
analyzer (Theme A items below) is a symptom of a missing concept in
std/. The fix path is the same as CM: model the fact, let the
property emerge, and the heuristic dissolves.

Computation model and migration plan:
[docs/cx-computation-model.md](docs/cx-computation-model.md)

DFA triage maps violations to four algebraic root causes:

| Root cause | Count | Fix |
|-----------|-------|-----|
| Parser SCCs | ~80 | DescentEvidence lattice (std/termination.dag) |
| Fold/catamorphism | ~40 | Descend primitive (std/iteration.dag) |
| CostExpr/SizeExpr | ~30 | Flat product-of-bounds (std/computation.dag) |
| Accessor-on-var | ~14 | Signature-driven fold (std/algebra.dag) |

### PR #318 work (CX-K through CX-R)

Reduced violation count via heuristic descent recognizers. Review
flagged 15 items as heuristics-over-structure — the count dropped but
the approach needs migration to structural authorities before the
remaining path is sound. Work accomplished:

- SCC proof constructor with TokenPosition progress dimension
- Parser edge classification (`collect_parser_edges_for_scc`)
- `is_algebra_iteration_method` reads `AlgebraMethodSemantics`
- Children iteration produces descent evidence
- Match-based descent detection (is_match_option_descent)
- Field projection descent recognition
- SCC parameter name unification in emit_pattern (CX-Q/CX-R)
- Soundness fixes: branching_only, all_safe, any-argument fabrication

### Review feedback (15 items from PR #318 — 10 resolved, 5 remaining)

**Theme A: Heuristic descent recognizers** (6 items)
- [x] #1-4: Already data-driven via `function_size_effects`, `node_field_roles`,
  `AlgebraMethodSemantics` tables (fixed in PR #328, stale line numbers)
- [~] #5: `is_match_option_descent` — documented assumption (PR #336).
  Full fix deferred to CX-D (coproduct projection metadata in std/)
- [~] #6: `lambda_param_names |> last` — replaced by `iteration_element_name`
  (PR #336). Direct template lookup deferred: emitter inlines cross-module
  AlgebraTypeTemplate variants into caller, breaking compilation.
  Structural fact confirmed in std/algebra.dag CallableOf declarations.

**Theme B: Producer patches** (2 items)
- [ ] `02_parse.dag:2759` — `node_to_name_str` split. Deferred until P4.1
  (wrapper transparency in SCC edge classification)
- [x] `04_types.dag:418` — not a patch; proper cardinality design

**Theme C: Fabrication / scope issues** (5 items — all fixed in PR #336)
- [x] `ParserResultDirectState` field renamed `progress` → `input` (consistency)
- [x] `same_progress_subgraph_has_cycle` now detects 1-node ProgressSame self-loops
- [x] `val_inner_vars` scope leak fixed — arm-local bindings no longer escape
- [x] `branching_proof_safe` accepts lexicographic proofs (checks first dimension)
- [x] Self-loop detection matches `proof_has_non_descending_cycle` pattern

**Theme D: Boundary / testing** (2 items)
- [ ] `tests/pipeline.rs:2505` — diagnostic tests non-hermetic (larger scope, deferred)
- [x] `05_emit_rust.dag:1334` — `emit_variant_pattern` checks `fielded_variants`
  when all bindings are wildcards (PR #336)

### Dependency chain

```
CX-D (model facts in std/)
 ├→ CX-B (wire LoweringTarget into analyzer, delete RecursionPattern)
 ├→ CX-A (consume DescentEvidence from std/termination.dag)
 ├→ CX-C (consume operation size contracts for fold evidence)
 └→ CX-E (re-enable gate once violations = 0)
      └→ PERF-3 (memory ratchet — can't validate until CX re-enabled)
```

**Dead code:** Three std/ declarations exist with no downstream consumer:
- `computation.dag: LoweringTarget` — defined, lowering table complete,
  but `complexity.dag` never reads it (CX-B not started)
- `termination.dag: is_valid_proof` — declared, returns `false`
  (CX-A not wired)
- `complexity.dag: RecursionPattern` (stage0 Rust) — should be replaced
  by `LoweringTarget` but both coexist with no bridge

### Work items

- **CX-D**: Model operation facts → heuristics dissolve. **Partial.**
  Three categories of structural facts:
  (1) **Operation size contracts** — `CollectionSizeEffect`
      (ShrinkEffect/ProjectionEffect/IdentityEffect) declared per-method
      on `AlgebraFieldTemplate`. `CostShape` (cost class) also on template.
      Complexity reads both from `AlgebraMethodSemantics`. Deleted:
      `ListMethodKind`, `classify_list_method`, `method_cost_shape_table`,
      `is_size_preserving_method`. `take` is explicitly `none`.
      Remaining modeling gap: `CollectionSizeEffect` mixes cardinality,
      structural-identity, and projection — reviewer wants orthogonal facts.
      `produces_collection` removed from `AlgebraMethodSemantics` — now
      derived at consumer from expression return type. `size_effect`/`cost_shape` on `MethodSemantics`
      is a bridge; endgame: facts on the resolved method Node. **(PR #328)**
  (2) **Type structure facts** — field sub-value relationships
      (child access produces a smaller tree). Not started.
  (3) **Coproduct projection facts** — match arms narrow the type,
      producing strictly smaller data. Not started.
  The lowering table in `std/computation.dag` (CallPattern → LoweringTarget)
  already models (1) but has no downstream consumer in complexity.dag.
  Remaining Theme A items: `is_tree_size_preserving_wrapper` hardcodes
  callee name. `is_children_list_field` reads from `node_field_roles`
  data table (already modeled).
- **CX-A**: DescentEvidence lattice unification — parser mutual recursion.
  **Partial.** Lattice, parser SCC proofs, lexicographic [TreeSize,
  TokenPosition] all implemented — including single-function lexicographic
  (delegates to SCC proof constructor). `proof_has_non_descending_cycle`
  builds graph directly from `ProofEdge` (ProgressSame bridge removed)
  and detects non-descending self-loops. Old classifiers deleted:
  `self_calls_have_descending_witness`, `self_calls_have_strict_parser_progress`.
  All recursion classification goes through unified proof constructor.
  Dead `skip()` ExprCall handlers removed (inference bridge normalizes).
  **(PR #328)**
  Deferred: `is_valid_proof` stub in termination.dag (blocked on
  graph utility extraction to `std/graph.dag` — import direction
  prevents termination.dag from calling SCC detection in complexity.dag).
  `ParserResultDirectState` field renamed (PR #336).
  `branching_proof_safe` now accepts lexicographic proofs (PR #336).
- **CX-B**: CostExpr/SizeExpr dissolution — cost expressions become flat
  products of SizeBounds from `std/computation.dag`'s lowering table.
  **Partial.** `CostShape` type moved to `dsl/std/algebra.dag`, declared
  per-method on `AlgebraFieldTemplate`. `method_cost_shape_table` (19-entry
  `Map<String, CostShape>`) deleted. Complexity reads `cost_shape` from
  `AlgebraMethodSemantics`. **(PR #328)**
  Remaining: flatten recursive CostExpr/SizeExpr into flat SizeBound
  products (Phase 4 — eliminates 18 cost algebra functions). `CostShape`
  is a bridge classifier — endgame: derive cost from `std/computation.dag`
  contracts. `produces_collection` now derived at consumer from return type.
  See [migration phases](docs/cx-computation-model.md#migration-phases).
- **CX-C**: Signature-driven fold evidence — self-calls inside
  `children |> fold` callbacks get structural descent proofs.
  **Blocked-by: CX-D** (needs operation size contracts for fold).
  Done: `is_algebra_iteration_method` reads `AlgebraMethodSemantics`;
  children iteration produces descent evidence.
  Deferred: lambda element uses `last` convention (Theme A);
  child-descent hardcodes `"children"` (Theme A).
- **CX-E**: Re-enable complexity gate — remove `complexity_diags = []`,
  un-ignore 14 complexity tests (10 `complexity_*`, 3 `soundness_*`,
  1 `structural_classify_*`; all `#[ignore]` with "CX track" comment).
  **Blocked-by: CX-A, CX-B, CX-C** (0 violations required).

### Acceptance

0 violations without suppression. CX gate re-enabled. Node is the only
recursive type consumed by complexity analysis. All descent evidence
reads structural facts — no heuristic name-matching in the analyzer.

---

## PERF: Compiler & Test Performance (parallel track)

**Goal:** Continuous visibility into compiler and test performance so
regressions are caught before they compound. Runs in parallel with all
lanes — any lane can introduce a regression.

### Existing infrastructure

| What | Where | Status |
|------|-------|--------|
| Self-compile time ratchet | `bootstrap::performance_ratchet` | `#[ignore]`, CI gate, 30s budget (~4.8s actual) |
| Bootstrap stage0→stage1 | `bootstrap::bootstrap_stage0_to_stage1` | `#[ignore]`, CI gate, ratchet 0 |
| Full DSL compile | `pipeline::full_dsl_compiles` | `#[ignore]`, GREEN |
| Stage0 freshness gate | `scripts/check-stage0-freshness.sh` | CI blocking |
| Diagnostic ratchet | `strict_compile_diagnostic_count` | `#[ignore]`, ratchet 325 |

### Work items

- **PERF-1**: ~~Un-ignore `performance_ratchet` in CI.~~ DONE (PR #326).
  30s budget, ~4.8s actual. CI gate catches O(n²) regressions.
- **PERF-2**: `bootstrap_stage0_to_stage1` enabled in CI (PR #326).
  When emission succeeds, gates emitted-Rust correctness (0 cargo check
  errors). Returns early without validation when complexity violations
  block emission — not yet an unconditional gate. Convergence proof
  (pass-1 = pass-2) remains in `bootstrap_fixed_point` (`#[ignore]`,
  not yet a CI gate — expensive: two full builds + two compiles).
- **PERF-3**: Self-compile complexity analysis. CX-E resolved (0 violations,
  CostUnknown deleted). OOM remains on full self-compile (~1600 functions):
  complexity analysis disabled in compile.dag for memory. Root causes (PR #336):
  (1) CostExpr tree blowup — mitigated by eager simplification in cost_seq/cost_par
  (2) Redundant body walks — mitigated by pre-computing all recursion patterns
      in build_complexity_report before the cost phase
  (3) Persistent map threading — each cache_summary creates a new Rc<CostInternTable>
      with cloned map (not yet addressed)
  (4) Rc-based CostExpr accumulation — ~1000 nodes per function × 1600 functions
      (not yet addressed — requires arena allocation or structural sharing)
  Next: profile peak RSS to identify which of (3)/(4) dominates.
- **PERF-4**: Test suite wall-clock ratchet. Current: ~270s for 271
  tests. Budget TBD. Individual tests >2s are suspect (per project
  convention). Add per-test timing visibility.
- **PERF-5**: Operation-count contracts. Test performance via structural
  operation counts (node visits, inference passes, emit calls) rather
  than wall-clock time. More deterministic, catches algorithmic
  regressions independent of machine load.

### PERF-6: Dependency-modeled computation (no redundancy, no retention)

**Thesis:** Re-derivation and over-retention are symptoms of the same
root cause: the pipeline doesn't model computation dependencies. When
it doesn't know what depends on what, it re-computes (redundancy) and
retains everything (over-retention) because it can't prove when a value
is safe to drop or when a walk is redundant.

**Two manifestations:**

1. **Redundant work** — same function body walked N times by different
   analyses that could share a single traversal.
2. **Over-retention** — intermediate values (CostExpr trees, descent var
   maps) kept in memory after all consumers have read them, because
   the pipeline has no concept of "transit vs result."

The general principle: if you can't express "this value is no longer
needed," you can't drop it. Every `Map<String, X>` that accumulates
full intermediate state and never prunes is an instance.

**Current instances (self-compile, ~1600 functions):**

Redundant work:
- `classify_recursion_pattern` walks body 2-3× (proof dimensions)
- `construct_scc_termination_proof` walks body 5-6× per SCC member
- `collect_descent_vars` walks body 1× per param per dimension
- `max_path_self_calls` walks body 1× (could be cached)

Over-retention:
- `CostInternTable.summaries` holds full CostExpr trees for ALL 1600
  functions simultaneously. Each tree is ~1000 Rc nodes. Consumers
  only need the classified string ("O(n)"), not the tree. But there's
  no transit/result distinction — everything is retained.
- Processing order is declaration order, not topological (callee-first).
  Topological order would allow dropping callee summaries after all
  callers are processed. The SCC computation already provides this order.

**Mitigation (PR #336):**
- Pre-compute all recursion patterns before cost phase
- Eager simplification in cost composition (prevent CostExpr blowup)
- Single-function patterns stored in scc_index (avoid re-classification)

**Endgame:** Two structural changes:
1. **Single-walk analysis:** Each function body walked ONCE → produces a
   `FunctionAnalysis` record with all facts (proof, descent vars, cost).
   Redundant walks become unrepresentable.
2. **Transit/result distinction:** Intermediate values (CostExpr trees)
   classified eagerly and dropped. The report stores classifications,
   not trees. `CostInternTable` becomes a transit cache with pruning,
   not a permanent accumulator. Functions processed in topological order
   so callees can be dropped when all callers are done.

### Acceptance

`performance_ratchet` and `bootstrap_stage0_to_stage1` running in CI.
No test >2s without justification. Self-compile time tracked per-PR.
Self-compile complexity analysis runs without OOM (PERF-3 + PERF-6).

### CX-NEXT: Cost algebra as compositional data modeling

**Status:** Design direction established (PR #336 discussion). Not yet implemented.

**Core problem:** `SizeExpr` and `CostExpr` are parallel expression algebras.
SizeExpr reinvents Add/Max/Log as size-specific variants instead of deriving
them from existing std/ concepts. This is the same anti-pattern as building
a mini-language inside the cost analyzer instead of composing facts from the
data model.

**Design direction:**
- **Sizes are facts, not expressions.** A collection's size is a structural
  fact (Layer 3 in MODELING.md), not a symbolic expression. The cost algebra
  should reference sizes by identity, not by re-encoding them in SizeExpr.
- **CostLog should emerge from iteration structure.** Divide-and-conquer
  (n → n/2) produces log₂(n) iterations — this should emerge from the
  recursion pattern, not from a hand-placed CostLog primitive. Remove CostLog
  from CostExpr; add SizeLog to SizeExpr (or better: derive it from the
  descent evidence in SizeBound).
- **SizeBound should carry descent type.** Currently `ArithmeticParam` drops
  whether descent is subtraction (linear: n iterations) or division
  (logarithmic: log n iterations). Preserving `ArithmeticDescent { op, by }`
  in SizeBound lets `bounded_recursive_cost` produce the correct iteration
  count without adding ad-hoc SizeExpr variants.
- **ExplicitCount should preserve the literal.** `ExplicitCount { n: 5 }`
  should produce `SizeConst { value: 5 }`, not `SizeConst { value: 1 }`.
  The constant IS the bound — collapsing it to 1 loses information. Only
  asymptotic normalization (formatting) should collapse constants.

**Open review feedback (PR #336, deferred to follow-up):**
- #8-9: `dfs_finish_order`/`dfs_collect_component` — visited-set-bounded
  recursion needs worklist primitive (ROADMAP I1/I2)
- #10: CostExtern — needs stdlib cost contract system
- #11: `iteration_element_name` — positional heuristic, needs cross-module
  template lookup (CG-2/CG-3)
- #13-14: `is_valid_proof` / `proof_has_non_descending_cycle` — proof
  validation gaps in std/graph.dag
- #15: SCC zero-placeholder — standard fixed-point technique, but should be
  documented as such (not silent fabrication)
- #16: `SizeConst(1)` for ExplicitCount — loses literal bound

---

# CM: Compiler Concept Modeling (cross-cutting retrospective)

**Analysis:** [`src/v2/CM.md`](src/v2/CM.md),
[`src/v2/CM-inventory.md`](src/v2/CM-inventory.md)
(will fold into MODELING.md when stable)

**Principle:** Surface existing structural authorities to every
consumer. Never add parallel classification types. Consume existing
authorities, never duplicate. After each feature, retro for new
modeling gaps.

**What CM analysis found for each lane:**

| Lane | CM diagnosis | Highest-leverage fix |
|------|-------------|---------------------|
| M2 (Lane 1) | `ExprLet` erases expected type → bare_map_node chain → 5+ downstream fallbacks | Propagate expected through ExprLet; normalize ExprVar→ExprCall for nullary |
| CG (Lane 2) | 39 heuristic sites, all "existing authority not surfaced" | Surface connective through TypeRendering; surface AlgebraFieldTemplate to emit |
| CX (Lane 3) | 4 analyzer heuristics = 4 missing std/ facts | Model operation size contracts in std/computation.dag |
| M4 | CM provides endgame rationale: structural fields ARE identity, names are rendering sugar | Surface structural fields; delete Node.name |

**Unowned files (~42 heuristic sites in 04_resolve, 04_access, coercion, 00_core):**
No lane addresses these. Highest-value: coercion.dag container-to-algebra
table (should derive from algebra declarations), 04_resolve.dag alias
resolution (4 fail-open sites).

**Arity boundaries (cross-cutting):** Container under/over-arity,
empty services/resources, Optional collapse, Callable/lambda mismatch.
Enforce arity from algebra profile at normalization. See CM.md §Arity
boundaries.

**Relationship to other lanes:** M2, CG, and CX all generate work
items that are actually CM problems. When a PR "moves a heuristic
upstream," that's CM work wearing an M2/CG hat.

## MM-1/2/3: Detailed analysis in CM.md

Full heuristic inventory, acceptance criteria, and design constraints
are in [`src/v2/CM.md`](src/v2/CM.md). Summary:

## MM-1: Item identity

**Problem:** 4 independent classification forests, ~27 branches total.
- Raw structural interrogation in emit (`.connective ==`, `.body !=`, `.transport ==`): 55 sites
- `TypedItemUnhandled` / `""` / `false` fail-open fallbacks: 8 sites

**Work items:**
- [x] Design: determine irreducible structural facts about items
- [~] Surface existing fields (`body`, `transport`, `connective`, `params`) to
  every consumption site — no new classification type.
  Shared predicates exist; ~55 raw structural interrogation sites not
  yet migrated to use them.
- [~] Implement: fail-closed boundaries (no TypedItemUnhandled, no `""` fallbacks)
  `TypedItemUnhandled` variant deleted; else branches still emit error
  markers (compile_error/panic/comment) — need upstream diagnostic instead.
- [x] Delete: all `classify_*` forests, all fail-open fallbacks, all name-keyed side-tables
  PR #324: `TypedItemKind` enum + `classify_typed_item` dissolved. Shared
  boolean predicates (`is_type_def_item`, `is_function_item`, etc.) in
  05_emit.dag replace the taxonomy. Backends compose predicates directly.

**Acceptance:** Emit dispatches on existing Node structural fields
directly. Classification forests dissolve because consumers pattern-match
on the facts they need (`body != none`, `connective`) rather than on a
derived taxonomy.

## MM-2: Type structure

**Problem:** `Conj/Disj/NoConnective` is a primitive, but its
interpretation (product → struct, sum → enum, leaf → primitive) is
re-derived at every consumption site.

**Current counts (2026-04-05):**
- `.connective ==` in emit + lookup + types: 57 sites
- `TypeSummary.repr` (StructRepr/EnumRepr) exists but only used by some consumers
- Type rendering dispatch in `05_emit.dag:1095-1191`: 97 lines, 7 branches

**Work items:**
- [ ] Surface connective (the existing authority) to all consumers that
  currently re-derive it — connective and `TypeSummary.repr` are the
  authorities; no new interpretation type
- [ ] Extend `TypeSummary.repr` to cover all type rendering, not just
  type definitions
- [ ] Delete: all inline `.connective == Conj` / `Disj` re-interpretation
  in emit

**Acceptance:** Emit reads connective or `TypeSummary.repr` (existing
authorities) directly. No multi-branch re-interpretation of connective
in emit — type rendering is an exhaustive match on the existing
authority.

## MM-3: Expression semantics

**Problem:** Method identity is a string. Call semantics (nullary
invocation, method kind, built-in refinement) re-derived from name
matching at every consumption site.

**Current counts (2026-04-06):**
- `method_def.name` / `method_name ==` dispatch: ~17 sites (emit_rust: 12, infer: 5, complexity: 0 — all complexity method dispatch reads from AlgebraMethodSemantics)
- Nullary function detection: 3 sites in emit_rust, 0 in Go/Python (bug)
- Built-in call type refinement by name: 4 sites in infer

**Work items:**
- [ ] Surface existing `method_def` Node and `AlgebraFieldTemplate`
  structural facts through the pipeline to emit — these ARE method
  identity; no new dispatch enum
- [ ] Normalize ExprVar → ExprCall during inference for nullary
  functions — the expression IR is the authority for invocation
  semantics
- [ ] Fix: Go/Python emit must handle nullary function references
  (ExprCall normalization fixes all backends at once)

**Acceptance:** Emit reads method structural facts from existing
`method_def` / `AlgebraFieldTemplate` authorities, not name strings.
Nullary invocation is ExprCall in the IR (normalized during inference),
not a type-name check at render time.

## CM acceptance (structural)

Each criterion is a claim: "this class of mistake is unrepresentable."
Consumers read existing structural authorities; the wrong question
can't be asked. See `src/v2/CM.md` for full design rationale.

**One clear end-state:** The existing structural data (Node fields,
connective, method_def, AlgebraFieldTemplate) flows through stage
boundaries intact to every consumer that needs it. No new parallel
classification types. No lossy boundaries that drop distinctions
consumers need. Reading a field IS reading the authority — that is
not re-derivation.

| Model | Claim | How to verify |
|-------|-------|---------------|
| MM-1 | Existing Node fields flow to emit intact; classification is unnecessary | No `classify_*` forests; emit reads `body`, `transport`, `connective` from the Node at the boundary |
| MM-2 | Connective / TypeSummary.repr is the authority; re-interpretation is unnecessary | Emit reads connective or repr — no inline `.connective == Conj` checks. TypeSummary.repr is only the single authority if proven non-lossy for ALL consumers, not just emit |
| MM-3 | method_def / AlgebraFieldTemplate is the authority; name dispatch is unnecessary | Emit reads structural method facts — no `method_name ==` string dispatch |

**Invariant guardrail:** Consume existing authorities, never duplicate.
Any proposed new fact layer or boundary type must demonstrate that
(a) no existing authority carries the needed fact, (b) the existing
authority cannot be surfaced to the consumer, and (c) the new layer
lands with a real downstream consumer in the same change. Speculative
metadata is rejected — the repo has already deleted prior boundary
layers that introduced unused or lossy fact tables.

---

# Layer 3: Deferred

## P1: Modeling Consolidation

Programs are applied mathematics with informal names. ~90 redundant
types are standard algebraic structures reimplemented ad-hoc.

| Item | What | Impact | Blocked on |
|------|------|--------|-----------|
| P1-A | Result monad unification (58 parser/resolve types → 2 generic) | ~58 types deleted, ~200 sites simplified | Generic type support |
| P1-B | Emit parameterization (3 backends → 1 parameterized homomorphism) | ~2,500 lines eliminated | CG-3 |
| P1-C | String→structural identity (~200 Map<String,X> → edges) | IS M4 Lane 2 | M4 |
| P1-D | Context/accumulator dedup (10 types → 4) | ~6 types eliminated | Independent |
| P1-E | Non-Node recursive type dissolution | ~45 variants, ~56 CX violations | CX lane |

Expected total: ~100 types eliminated, ~2,500 emit lines eliminated,
stage0 mirror ~17K → ~13K lines.

## Model Convergence

Post-bootstrap. Remaining recursive types dissolve into Node:

| Type | Dissolution | Milestone |
|------|-------------|-----------|
| CostExpr/SizeExpr | Flat product-of-bounds via std/computation.dag | CX-B |
| TypeRendering | Coercion engine | M5 |
| MatchPattern | Node discriminant metadata | M7 |
| InferredNode | Keep wrapper, non-recursive reference | M7 |

Non-recursive authority leaks (same class of unfinished migration):

| Leak | Fix |
|------|-----|
| `Node.name` as authority (~256 sites) | M4 Lane 2 |
| Transport/config per-backend rendering | Inherent per-language differences |
| Bare parameterized types | Reject at normalization |
| ~~Missing `CallableOf`~~ | ~~M4 Tier 2.5~~ (DONE) |
| Semantic strings (parent_enum, service_name) | Structural Node references |

## Pipeline Algebraic Grounding

Every compiler type is a mathematical concept in disguise. Grounding
in std/ eliminates ad-hoc implementations and makes new capabilities
fall out of existing infrastructure.

Key redundancies:

| What | Count | Actually is | Collapses to |
|------|-------|-------------|-------------|
| Parse result types | 68 | State monad `ParseM<T>` | 1 generic type |
| ProgressKind ≅ DescentEvidence | 2 | Same BoundedLattice | 1 from std/termination.dag |
| CostExpr + SizeExpr | 12 variants | Tropical semiring | Node trees |
| AlgebraTypeTemplate | 9 variants | Type constructor free algebra | Node trees |
| TypeRendering | 7 recursive fields | Coercion checkpoint | std/coercion.dag |

Missing std/ concepts: `std/discrimination.dag` (pattern matching),
`std/graph.dag` (directed graph algebra), Signature + Term in
`std/algebra.dag`, Cardinality lattice in `std/types.dag`.

## Deferred Milestones

- **M1**: COMPLETE (90 dsl + 29 v2 files, 0 diagnostics)
- **M3**: Test generation and guarantee receipt. Receipt schema as .dag
  type, CI validates receipt against ratchets, cross-language test
  taxonomy. Depends on M2.
- **M5-full**: Language plugin extraction. Delete 3 language-specific
  emit files (~6,857 lines). Adding a language = adding data.
  Depends on M4 for identity dissolution.
- **M6**: Parse-emit symmetry. Parse and emit are structural inverses.
  The parser reads SyntaxSpec; the emitter should read the same spec
  in reverse.
- **M7**: Dissolve structural bridges. `connective`, `return_cardinality`,
  `MatchPattern`, `InferredNode` wrapper — all become graph structure
  or metadata on Node.

## BP-1: Build Pipeline Model

The full compilation pipeline — `.dag` source → intermediate
representation (Rust/Python/Go) → executable artifact — is a
dependency DAG that should be modeled in `.dag`. Each stage has an
artifact type, a transform tool, and its own dependencies. This is
the same problem the compiler solves for user code: resolve a
dependency graph, detect cycles, determine evaluation order.

```
.dag source ──(v2-compiler)──▶ .rs/.py/.go ──(cargo/python/go)──▶ binary/runtime
     ▲                              │
     └──────── bootstrap cycle ─────┘  (Rust only: stage0 self-compile)
```

### Existing infrastructure

Pieces already modeled:

| Concept | Where | What it captures |
|---------|-------|-----------------|
| Intermediate targets | `src/v2/artifact.dag` `RenderTarget` | Rust, Python, Go, Dag |
| Artifact taxonomy | `src/v2/artifact.dag` `ArtifactKind` | ServiceBinary, Library, Frontend, GeneratedSupport |
| Per-target scaffold | `src/v2/languages.dag` `ProjectScaffold` | manifest file, source dir, extension |
| Cargo as tool | `dsl/extdeps/cargo.dag` | packages, targets, profiles, Build/Test/Clippy ops |
| Workspace graph | `dsl/extdeps/gunbc.dag` | `CrateRole`, `WorkspacePackage`, 13 packages |
| Build phases | `dsl/tools/build.dag` | multi-stage cargo pipeline with result aggregation |
| Codegen execution | `dsl/tools/codegen.dag` | stamp-file-gated conditional codegen |
| Output paths | `dsl/config/codegen_paths.dag` | output dirs, stamp files, path templates |
| Pipeline runs | `dsl/gunbc/workflow/types.dag` | `PipelineRun`, `PipelineArtifact`, stage outcomes |

What's missing: the **derivation chain** connecting these — which
artifact is produced from which source by which tool, the bootstrap
cycle and its fixed-point condition, and the generated-vs-source
distinction as structural data.

### Work items

- [ ] **BP-1a: Derivation model.** Types for pipeline stages, derivation
  edges (source artifact → tool → output artifact), and external tool
  dependencies. Compose existing `RenderTarget`, `ProjectScaffold`,
  cargo service, and `ArtifactKind` into an explicit derivation DAG.
  Location: `dsl/std/pipeline.dag` or extend `dsl/extdeps/gunbc.dag`.
- [ ] **BP-1b: Bootstrap cycle.** Model the self-compile cycle: stage0
  as bootstrap seed, the fixed-point condition (pass N == pass N+1),
  and the two-pass rule (required when emitter changes its own output).
  Hand-maintained files (2) enumerated as source artifacts distinct
  from generated artifacts.
- [ ] **BP-1c: `.gitattributes` generation.** Files the pipeline model
  marks as "generated" get `-merge` in `.gitattributes`. The
  `.gitattributes` file is a derived artifact from the pipeline model,
  not hand-maintained config. Analogous to how `dsl/tools/bootstrap.dag`
  generates `.gitignore` from `dsl/config/gitignore.dag`.
- [ ] **BP-1d: Merge workflow.** Document and enforce: resolve `.dag`
  conflicts → accept either side of generated stage0 → regen → commit.
  CI freshness gate (`check-stage0-freshness.sh`) is the safety net.

### Acceptance

1. The derivation chain from `.dag` source through intermediate
   representation to executable is explicit `.dag` data, not
   shell-script convention.
2. `.gitattributes` is generated from the pipeline model. Generated
   files have `-merge`; hand-maintained files do not.
3. Every file in `src/v2/stage0/src/` is classified as either
   generated or hand-maintained by the model.
4. The bootstrap cycle, fixed-point condition, and two-pass rule are
   structural facts in the model.
5. Stage0 merge conflicts no longer require manual resolution.

## Bootstrap Design Debt

Acknowledged design debts from the stage0 elimination work (PR #337).
These are pre-existing patterns that became more visible when main.rs
and compiler_tests.rs moved from hand-maintained to emitter-generated.

| Item | What | Root cause | Fix direction |
|------|------|-----------|---------------|
| **FF-9a** | `has_pipeline` uses `module.name == "v2.compiler.compile"` string check to decide crate naming, main.rs shape, and compiler_tests emission | No explicit artifact/entrypoint fact in the type system | Model `Artifact` / `Entrypoint` as structural facts; `has_pipeline` dissolves when artifact identity is structural |
| **FF-9b** | `extract_module_path` / `extract_import_paths` are line-based text scanners duplicating parser knowledge | Bootstrap bridge — the CLI needs to discover modules before the full parser runs | Replace with AST-backed import traversal; CLI loading becomes a thin model-driven pass |
| **FF-9c** | `--source-root` conflates entry roots and dependency pools; only first root scanned for entries | No explicit entry-root vs dependency-root distinction | Either split into `--entry-root` + `--source-root`, or add explicit entry module list |
| **FF-9d** | `ct_self_compile_sources` and `gist_sources` maintain separate curated file lists (`dsl_deps`) | Duplicate source-closure authority alongside FF-9 import resolution | These closures should use the same import resolution as the production binary |
| **VER-1** | `std/verification.dag` types exist but are not yet the live authority; `CoercionAssertion` / `CoercionTestEntry` in coercion.dag are the active producer/consumer | Coercion tests are domain-specific; shared vocabulary not yet needed by a second consumer | Wire `TestCase` from std/verification.dag as the shared abstraction when a second structural test domain (algebra laws, tokenizer contracts) is added |

FF-9a through FF-9d are all facets of the same root cause: the
bootstrap pipeline's source-discovery layer was hand-maintained and
is now emitter-generated, but it's still a text-scanning bridge rather
than a model-driven pass. The endgame is FF-9 complete: the compiler's
own import resolution is the single authority for source discovery
everywhere — CLI, tests, and CI.

## Exploratory Directions

- **Bounded iteration**: three primitives — fold (structural descent
  over children), descend (recursive with proof witness), repeat
  (bounded by fuel). All loops are sugar over these. `.dag` has no
  unbounded `while`.
- **Cost comparator**: refuse to compile suboptimal code when a
  cheaper equivalent exists. Requires cost algebra completion.
- **Cost algebra extensions**: Pow, Sqrt, Exp; amortized analysis via
  potential functions; space as peer dimension. See
  `docs/cost-algebra.md`.
- **Style emission**: Emission formatting (indentation, braces,
  statement terminators) should be data-driven from two authorities:
  (1) Language spec — significant whitespace (Python), block delimiters,
  statement terminators. These are correctness requirements.
  (2) Style spec — indent unit, brace placement, readability conventions.
  These are readability requirements (correctness during bootstrap,
  optional post-bootstrap via external formatters like gofmt/rustfmt).
  `BlockSyntax` on `LanguageSpec` is the first step. Future work:
  TCO syntax templates, async call prefix, identifier escaping
  conventions, post-emission formatting pass.
- **Post-Rust path**: .dag → native code (LLVM/Cranelift) directly,
  no Rust intermediate. Optional — Rust/Go as intermediates may
  suffice indefinitely.
