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

Six mutually exclusive lanes. Each lane owns distinct files — two
lanes never modify the same file. All run in parallel from bootstrap.

```
            ┌─ Lane A: Inference (M2, M4-L1) ────────────────────────┐
            │                                                         │
M1 COMPLETE─┼─ Lane B: Emission (CG, LS, RE-1, KF-6) ───────────────┤
Bootstrap D │                                                         ├→ M4-L2
  COMPLETE  ├─ Lane C: Complexity (CX, KF-1, KF-2) ──────────────────┤  (Node.name
            │                                                         │  deletion —
            ├─ Lane D: DSL Modeling (RE workflow, BC-1..4, services) ──┤  cross-cutting
            │                                                         │  final phase)
            ├─ Lane E: Testing (KF-3, KF-4, M3) ─────────────────────┘
            │
            └─ PERF (continuous — parallel to all lanes)
```

### Lane file ownership (mutually exclusive)

| Lane | Owns | Never touches |
|------|------|--------------|
| **A: Inference** | 00_core, 02_parse, 04_resolve, 04_infer, 04_types, 04_patterns, 04_lookup, 04_items, 04_access, 04_service | 05_emit*, complexity, dsl/, tests/ |
| **B: Emission** | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python, 04_emit_info, dsl/extdeps/languages/*, dsl/extdeps/transports/* | 04_infer, 04_types, complexity, dsl/std/, tests/ |
| **C: Complexity** | complexity.dag, docs/cx-*, docs/cost-* | 04_*, 05_*, dsl/, tests/ |
| **D: DSL Modeling** | dsl/std/, dsl/extdeps/{llm,github,shell,cron,cloud,git}/, dsl/gunbc/, dsl/tools/, dsl/config/ | src/v2/*.dag (compiler sources) |
| **E: Testing** | src/v2/tests/, scripts/, docs/testing-*, std/verification.dag, compiler_tests_rust.dag, coercion.dag (test extraction) | 04_*, 05_emit*, complexity |
| **PERF** | performance ratchets only — reads all, writes none | (monitoring, not modification) |

### What each lane delivers

| Lane | Tracks | KF | Release gates |
|------|--------|-----|--------------|
| A | M2 (BND-1..4), M4-L1 (declaration algebra, Tier 2.5/2.6/3) | — | Gate 1 (M2, M4) |
| B | CG (TLC-4, P1-B), LS (spec data), RE-1 (transport fidelity), KF-6 (Verilog) | KF-6 | Gates 1 (CG,LS,RE), 3 (parity), 4 (hardware) |
| C | CX-NEXT (526→0), KF-1 (complexity proof), KF-2 (reject suboptimal) | KF-1, KF-2 | Gates 1 (CX), 4 (complexity) |
| D | RE-2..5 (review.dag, gist.dag), BC-1..4, service extdep models | — | Gates 1 (RE), 5 (business cases) |
| E | M3 (test generation), KF-3 (witnesses), KF-4 (cross-language) | KF-3, KF-4 | Gates 3 (parity), 6 (test gen) |

### Cross-lane dependencies (minimal)

Lane D reads from Lane A (resolved types) and Lane B (emitted code)
but never writes to their files. Lane E reads from all lanes (to
test them) but never writes to their files. The only true cross-lane
gate is M4-L2 (Node.name deletion) which touches all files — it runs
as a final coordinated phase after Lanes A+B reach their acceptance
criteria.

### Dependency-gated phases

```
Phase 1 (now):  Lanes A, B, C, D, E all run in parallel
Phase 2:        M4-L2 (Node.name deletion) — after Lanes A+B done
Phase 3:        Gate 2 (structural debt) — after M4-L2
Phase 4:        Gate 7 (demo polish) — after all other gates
```

---

# Layer 1: Active Gates

## Bootstrap Status

| Stage | Status | Gate | Notes |
|-------|--------|------|-------|
| **A** | GREEN | 0 diagnostics | PR #264 |
| **B** | GREEN | 0 emitted-Rust errors | PR #307. `bootstrap_fixed_point` CI gate (PR #346) |
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
| L1 gate | `scripts/l1-ratchet.sh --check` | GREEN (0, hard gate — PR #352) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

## Ratchet Counts

| Metric | Current | Target | Notes |
|--------|---------|--------|-------|
| Self-compile diagnostics | 528 | 526 | Honest count — +2 from transport property inference complexity paths |
| L1 type knowledge | 0 | 0 | GREEN — hard gate (PR #352). Constructor functions dissolved, ListOf/ReceiverCollectionOf merged into ContainerOf. |
| Complexity violations | 325 | 526 | Honest: 526 functions with unrecognized descent (SameArgumentCall → Forever). Higher than 325 because analysis now covers std/ + lambda recursion visible. Ratchets down as analyzer improves. |
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
4. Run `./scripts/regenerate-stage0.sh` (includes two-pass fixed-point check)
5. Require `git diff --exit-code src/v2/stage0`

Stabilization rules:
- No manual `src/v2/stage0/` edits once regeneration is green.
- CI gate: `check-stage0-freshness.sh` (regenerate → diff → empty).
- Stage0 generated `.rs` files use `-merge` in `.gitattributes` (no line-level merge).
- CX gate disabled in both stage0 and `compile.dag` — emission not blocked by complexity violations. Re-enable when CX violations reach 0.
- CLI exit code filters complexity diagnostics: `main.rs` exits non-zero only for hard errors (non-`ComplexityUnknown`). Without this filter, the 522 pre-existing complexity violations make the freshness gate fatal. Remove the filter when CX reaches 0 (same gate as CX-E).

### Self-Hosting Gap (discovered 2026-04-08, fixed PR #346)

`bootstrap_fixed_point` is now a CI gate (~90s). Complexity early-return
removed from `bootstrap_stage0_to_stage1`. Two-pass regen in
`regenerate-stage0.sh`. `bootstrap_stage0_to_stage1` subsumed by
fixed-point in CI.

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

## M2: Boundary Sufficiency (Lane A)

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

### Unified container child encoding (PR #339, merged)

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
- [x] Consumer updates: major readers updated to `child_type_node` — lookup,
  items, patterns, emit_info, emit, emit_rust, access, infer, resolve.
- [x] All 314 tests pass, 0 self-compile diagnostics, L1 ratchet unchanged

**Completed (PR #340) — data-table reads + boundary-contract dissolution:**
- [x] Hardcoded `"T"`/`"K"`/`"V"` replaced by `container_param_name()` data
  table reads in `map_node`, `bare_map_node`, `list_of_type_variable`,
  `list_of_element`, `apply_type_substitution`, `resolve_node_bounded`
- [x] `rt_type` dissolved — 105 call sites migrated to two boundary-contract
  accessors: `decl_resolved_type` (54 Strict sites) and `emit_guarded_type`
  (48 Guarded sites). `rt_type` deleted. Non-Resolved branches return
  `error_type` (CompilerError propagates), not `unit_type` (no fabrication).
- [x] `container_param_name` `None` branches return `"__MISSING_PARAM__"`
  error marker (not fabricated `"T"`/`"K"`/`"V"` — visibly wrong if reached)
- [x] `child_type_node` updated to use `decl_resolved_type`
- [x] `is_fully_resolved`, `algebra_child_or_placeholder`, `for_each_element_type_node`,
  `apply_type_substitution` migrated from `rt_type` to `child_type_node`/`decl_resolved_type`
- [x] 316 tests pass, L1 ratchet 27 (unchanged), stage0 fresh

**Completed (bootstrap convergence):**
- [x] `substitute_type_slots` recurses into `inferred` on field-style
  children — generic type instantiation works through wrappers
- [x] `has_nested_records_node` extracts via `child_type_node` — data
  declarations with `List<SomeStruct>` correctly use JSON emission
- [x] Bootstrap convergence: two-pass fixed point verified
- [x] `child_inferred_or_name` dissolved — 7 call sites → `rt_type(n: ch)`,
  function deleted
- [x] `__EMIT_BUG_ANONYMOUS_FIELD__` removed — dead code

**Cross-talk analysis (PR #340 investigation):**

Eight Node fields serve multiple semantic roles, forcing consumers to
disambiguate by checking other fields. This is the same shape as the
`inferred` problem — two producers writing to the same channel with
different guarantees — but `inferred` is the only one causing active
information loss.

| Field | Roles | Cross-talk? | Status |
|-------|-------|-------------|--------|
| `inferred` | declaration type vs expression type | YES (different error contracts) | Bridge accessors (PR #340). Structural fix: next |
| `name` | identifier, type name, field name, method name (~7 roles) | YES (same guarantee, different semantics) | M4 dissolution target (PR #341) |
| `children` | struct fields, container params, if/match/call layouts (~12) | YES (layout-dependent) | ChildRole metadata table exists |
| `params` | function parameters vs type parameters | YES | Connective disambiguates |
| `return_cardinality` | field cardinality vs function return cardinality | YES | accessor naming exists |
| `properties` | transport config, module markers, operation modifiers | YES | transport-specific accessors |
| `body` | function body vs none-for-external | No (same contract) | — |
| `match_pattern` | match arm pattern vs field binding pattern | YES | — |

### Structural boundary types (next — dissolves bridge accessors)

**Root cause:** `decl_resolved_type`/`emit_guarded_type` are bridge
accessors that read from the same `inferred: InferredNode?` field and
handle error branches that shouldn't exist for their respective roles.
The accessors document the contract but don't enforce it structurally.
The fix is coproduct boundary types consumed directly by downstream code.

**Design:** Two producers write to `inferred` with different guarantees:
- **Resolve** (declarations): always `Resolved` — error is a compiler bug
- **Infer** (expressions): any variant — error is a user code problem

These are static, compile-time producers. The fix is not "track who set
it" but "make the producers produce structurally distinct things so the
wrong question is unaskable."

**Approach — coproduct boundary carriers:**

Model the resolve→infer→emit boundary as real coproduct types that
consumers pattern-match directly. No bridge accessors, no error branches
on impossible states.

- [~] **BND-1: Declaration-child boundary type.** Bridge accessors
  dissolved: `resolved_type_or_error` (101 sites) deleted, `rt_node`
  and `NodeType` deleted. Consumers use fail-closed `resolved_type`
  (returns `error_type`, not `unit_type`) or direct `InferredNode?`
  matches (PR #347). Remaining: `resolved_type` is still a bridge
  accessor — endstate is direct consumer pattern-matching on a
  declaration-only carrier that structurally can't represent errors.

- [~] **BND-2: Emit-ready boundary type.** Same infrastructure as
  BND-1 — emit sites use `resolved_type` (fail-closed). Remaining:
  emit should receive a carrier that structurally can't be error-typed,
  enforced by a gate at the infer→emit boundary.

- [x] **BND-3: Boundary vocabulary in `00_core.dag`.** `InferredNode`
  is the boundary type. No standalone `std/boundary.dag` needed — the
  compiler is the only consumer. Can extract to `std/` if a second
  consumer appears.

- [~] **BND-4: `container_param_name` derives from algebra.** T/K/V
  parameter names now derive from `algebra_type_param_names` declared
  per-profile in `std/algebra.dag` (PR #347). Hardcoded
  `container_type_param_names` table deleted. `__MISSING_PARAM__`
  sentinels dissolved — `container_param_name_required` (non-optional)
  centralizes the fallback in `std/types.dag` (PR #352). Remaining:
  `container_param_name_required` falls back to `kind_name` when no
  profile exists. Dissolves with Tier 3 (FF-9): when the compiler
  reads type declarations at resolve time, param names come from the
  declaration itself — the string→profile→names lookup chain and its
  fallback become unnecessary.

**Dependency:** BND-1 and BND-2 can land independently. BND-3 lands
with whichever is first. BND-4 requires Tier 2.5 (algebra-derived
type parameter names) — independent of BND-1/BND-2.

**Progress (PR #347):** `resolved_type_or_error`, `rt_node`, `NodeType`
deleted. Fabrication eliminated (`unit_type` → `error_type`). Hardcoded
`container_type_param_names` table deleted (derives from algebra).
Type parameter roles declared directly per-profile (no template scanning).
Net -201 lines.

**Current state:** `resolved_type` is the explicit boundary contract —
a 4-line fail-closed accessor that returns `error_type` for non-Resolved.
The `_ => error_type` branches are compiler-bug paths (resolve guarantees
Resolved for declarations, infer guarantees Resolved for successful
expressions). This is not fabrication — `error_type` carries
`CompilerError` in its `inferred` field and propagates visibly.

**Design direction — `Node<I>` parameterization (deferred):**
Recursive generics work in the language (tested: `MyList<T> = Nil | Cons { head: T, tail: MyList<T> }`).
`Node<I>` could parameterize the `inferred` field type. Challenge:
a single Node tree is heterogeneous (declaration children have
`Resolved`, expression children have `InferredNode`), so uniform `I`
doesn't capture the per-node invariant. Would require either per-field
parameterization or separate declaration/expression subtrees. Worth
exploring when Node generics or per-stage IR becomes practical.

**Acceptance:** No fabrication (done). `resolved_type` is the boundary
API. Further structural enforcement deferred to `Node<I>` exploration.

### Acceptance

No fabricated type args, no generic/wrong fallback return types, no
error-typed children reaching emit. BRIDGE fabrication count: 0 real.
Ownership and clone correctness tracked under CG lane.

---

## CG: Codegen Correctness + Optimality (Lane B)

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
authority. Deletion requires declaration-driven identity and structural
algebra.

**Status: L1 = 0 (hard gate, PR #352).** All 10 name comparisons
dissolved (Arrow connective for Callable, structural `is_pair` for
Tuple, `is_compiler_error` for Dynamic/Error, `ordered_element_collections`
for List). `ListOf` and `ReceiverCollectionOf` merged into
`ContainerOf { source: ContainerSource, element }` in algebra.dag.
L1-tracked constructor functions deleted from production code:
`container_node`, `tuple_node`, `callable_node`, `map_node`, `leaf_node`.
`bare_map_node` remains (not L1-tracked, inlined its `map_node` call).
`unify_template` enforces `ContainerSource` (carrier name must match).
`container_param_name_required` replaces `__MISSING_PARAM__` sentinels
(fail-closed: visible error marker if profile missing).

### M4 Lane 1: Declaration-driven algebra (Lane A)

Goal: compiler reads type/algebra facts from `.dag` declarations
instead of hardcoding them.

- Tier 1 (data tables → .dag): DONE
- Tier 2 (factor enrich_kernel_type): DONE
- Tier 2.5 (algebra bridge fidelity): DONE
  - [x] `CallableOf` variant for higher-order callback shapes
  - [x] T/K/V parameter names derive from algebra via `container_param_name_required`
  - [x] `ListOf`/`ReceiverCollectionOf` merged into `ContainerOf { source, element }`
  - [x] `unify_template` enforces `ContainerSource` (carrier name match)
- Tier 2.6 (functional system modeling):
  - [ ] Model function application as a concept (apply/call vs function-value-ref)
  - [ ] Inference encodes "this is a call" in the IR node, not as a type-arity heuristic
  - L1 Callable name comparisons already dissolved (Arrow connective).
    Tier 2.6 is about deeper concept modeling, not L1 violations.
- Tier 3 (full structural algebra, requires FF-9):
  - [ ] Compiler reads type declarations + algebra edges at resolve time
  - [ ] Derive kernel/container identity from type declarations
  - [ ] CollectionKind bridge dissolves when method algebras land
  - [x] 27 type constructor sites → 0 (PR #352)

### M4 Lane 2: Node.name deletion (D6) — Phase 2, cross-cutting

Goal: delete `Node.name` field. Rendering uses `source_text_at`,
resolve uses structural identity.

- B3 (emit rendering): DONE
- B4 (resolve structural identity): accessor layer done, `node.name`
  still semantic authority underneath
- D6 progress (PR #356):
  - [x] Thread `si: NewlineIndex?` through complexity + ownership
  - [x] Migrate ~90 accessor calls to `_at` variants
  - [x] Centralize type builders in `04_types.dag`
  - [x] Fix `ident_span` on 4 constructors (`name_span` parameter)
  - [x] Fix ident_span through infer/resolve node reconstruction (18 sites)
  - [x] Fix 6 name_span widening bugs (binding.span → node_name_span)
  - [x] Wire real `source_index` through emitter (46 calls), infer (13),
    resolve (12), access (9), ownership (2 entry points), types (18)
  - [x] Thread `source_index` through type utilities: `is_fully_resolved`,
    `node_type_compatible`, `node_type_equals`, `node_type_shape`,
    `node_type_deps`, `check_index_access_node`, etc.
  - [x] Remove dual-si from `build_complexity_report` (FuncEntry.si only)
  - [x] Migrate ~30 direct `n.name` reads to `authored_name_at` in emit/types
  - [x] Revert `@synthetic:` ident_span — structural identity is correct path
  - **Status:** 52 `source_index: none` remain in scope-free functions.
    ~20 direct `n.name` reads remain. 115 Node construction sites need
    `name:` removed for field deletion.
- D6 open (structural work, not mechanical wiring):
  - [ ] Add `NewlineIndex` to `ParserState` (14 parser calls)
  - [ ] Thread `source_indices` through compile.dag serialization (11 calls)
  - [ ] Thread `source_index` through mock/service/transport utilities (19 calls)
  - [ ] Migrate remaining `n.name` reads (resolve slot_bindings, service
    names, normalize, access `is_ordered_element_collection`)
  - [ ] `named_collection_type` fabrication — `container_param_name` gap
  - [ ] Kernel type identity: structural checks, not name recovery
  - [ ] Update ~115 Node constructions to drop `name:`
  - [ ] Delete `Node.name` field + non-`_at` accessors + scrambled-name tests

Lanes share only `00_core.dag` (different functions, no conflict).

### Acceptance

`scripts/l1-ratchet.sh --check` reports 0. Scrambled-name tests pass
then deleted. `Node.name` field deleted.

---

## CX: Complexity Analyzer (Lane C)

**Status:** Down from 315 → 164 (main) → 76 after PR #318 → 526 honest (PR #336).
Phase 1-2 complete (RecursionPattern deleted, all classifiers return LoweringTarget).
CX-N: var threading, type-directed dimension selection, algebra-to-dimension bridge.
526 honest violations — CostUnknown restored for unresolved descent patterns.
Complexity analysis re-enabled in compile pipeline (non-blocking gate).
PR #336: soundness fixes, graph extraction, is_valid_proof, honest CostUnknown.
See `docs/cx-violation-triage.md` for the 3-fix reduction path.

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
- [x] `proof_safe_for_branching` requires ALL dimensions structural (not just first)
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
  **Current: 526 violations (non-blocking gate). See CX-NEXT.**

### Acceptance (endgame — Gate 4)

0 violations without suppression. CX gate blocking. Node is the only
recursive type consumed by complexity analysis. All descent evidence
reads structural facts — no heuristic name-matching in the analyzer.

### CX Launch Gate (public release)

**Goal:** User-written .dag functions get proven complexity bounds.
Built on type-derived strict descent (not heuristic pattern-matching).
Fail-closed: unknown complexity is a hard error (INVARIANTS.md).

**Hard gate:**
- Strict descent on standard patterns: tree walks (recursive type
  fields), list consumption (`List |> fold/map`), arithmetic decrease
- Descent derived from type declarations (`recursive_type_set`,
  `RecursiveVariantFieldWitness`), not hand-maintained tables
- Fail-closed: if the analyzer cannot prove a bound, compilation
  fails with a hard error telling the user to restructure
- Bounds reported as user-facing diagnostics (`info: f is O(n) in tree_size`)
- Architecture consistent with `docs/cx-design.md` (P1-P6)

**Aspirational (not blocking launch):**
- Suboptimality rejection: small hand-curated equivalence catalog
  (e.g., `filter |> count > 0` → error, suggest `any()`)
- Compiler self-analysis: internal violations → 0
- CostUnknown variant deleted

**What this does NOT require:**
- Proving the compiler's own 1600 functions (internal debt, not user-facing)
- Exotic patterns: worklists, condition-dependent, parser SCCs
- Global descent (strict descent is sufficient for launch)
- The full Gate 4 architecture

**Acceptance test:**
```
cx_launch_user_code_bounds
  Input: .dag program with tree-recursive, list-fold, and arithmetic functions
  Assert: each function gets a proven bound in diagnostics
  Assert: no heuristic pattern-matching in the analysis path
  Assert: descent facts derived from type declarations
  Assert: function with unresolvable recursion produces hard error
```

---

## PERF: Compiler & Test Performance (parallel track)

**Goal:** Continuous visibility into compiler and test performance so
regressions are caught before they compound. Runs in parallel with all
lanes — any lane can introduce a regression.

### Existing infrastructure

| What | Where | Status |
|------|-------|--------|
| Self-compile time ratchet | `bootstrap::performance_ratchet` | `#[ignore]`, CI gate, 30s budget (~4.8s actual) |
| Bootstrap stage0→stage1 | `bootstrap::bootstrap_stage0_to_stage1` | `#[ignore]`, not in CI — subsumed by fixed-point |
| Bootstrap fixed-point | `bootstrap::bootstrap_fixed_point` | `#[ignore]`, CI gate (~60s) — subsumes stage0→stage1 |
| Full DSL compile | `pipeline::full_dsl_compiles` | `#[ignore]`, GREEN |
| Stage0 freshness gate | `scripts/check-stage0-freshness.sh` | CI blocking |
| Diagnostic ratchet | `strict_compile_diagnostic_count` | `#[ignore]`, ratchet 325 |

### Work items

- **PERF-1**: ~~Un-ignore `performance_ratchet` in CI.~~ DONE (PR #326).
  30s budget, ~4.8s actual. CI gate catches O(n²) regressions.
- **PERF-2**: `bootstrap_fixed_point` enabled in CI (PR #346).
  Two-pass self-hosting gate: builds stage1, uses it to produce stage2,
  diffs for idempotence. Subsumes `bootstrap_stage0_to_stage1`.
  Complexity early-return removed — test is unconditional.
- **PERF-3**: Self-compile complexity analysis. 526 honest violations
  (CostUnknown restored). OOM resolved by lambda recursion detection fix
  (PR #336). Complexity analysis re-enabled in compile.dag (non-blocking).
  Original OOM root causes (PR #336):
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

`performance_ratchet` and `bootstrap_fixed_point` running in CI.
No test >2s without justification. Self-compile time tracked per-PR.
Self-compile complexity analysis runs without OOM (PERF-3 + PERF-6).

### CX-NEXT: 526 → 0 violations (3 structural fixes)

**Status:** 526 honest violations. Full triage in
[`docs/cx-violation-triage.md`](docs/cx-violation-triage.md).

All 526 trace to 3 root causes. 280 are direct (recursive functions
where the analyzer can't see descent). 227 are composed (callers of
direct unknowns — resolve automatically). 3 structural fixes cover all:

| Fix | Direct | Composed | Total |
|-----|--------|----------|-------|
| Node tree descent recognition | ~230 | ~200 | ~430 |
| Parser SCC TokenPosition threading | ~73 | ~53 | ~126 |
| Graph DFS worklist (I1/I2) | 2 | ~10 | ~12 |

When all three are done, `CostUnknown` can be deleted from `CostExpr`
— because no code path can produce it.

**Cost algebra design direction:** SizeExpr is a parallel algebra that
should derive from std/ concepts. Sizes are structural facts, not
symbolic expressions. CostLog should emerge from iteration structure.
See triage doc for details.

**Deferred CX design improvements:**
- ComplexityReport stores rendered class strings; should keep typed
  CostExpr as authority and derive display at reporting boundary (M5/M9).
- ExplicitCount/Forever should be first-class iteration witnesses, not
  ad-hoc helpers (is_constant_bound, constant_bound_value).
- SCC edge collector (CX-P) checks all arg values — sound for
  single-Node-param functions but theoretically unsound for multi-Node-param
  functions. Proper fix: thread callee measure params through edge collection.
- Optional unwrap tracing: serialize SCC includes json_optional_node(Node?)
  — analyzer can't trace descent through Optional unwrap patterns.
- Condition-dependent termination: emit_cli_param_type_node recurses with
  with_required_cardinality (preserving, bounded by 1 via condition change).
  Analyzer can't express "condition becomes false after transformation."

**Open review feedback (PR #336):**
#8-9 (DFS worklist), #10 (CostExtern contracts), #11 (element name
heuristic). All documented at code sites and in the triage doc.
#13-14 (proof validation) and #20-21 (violation reason/span) resolved.

---

## RE: Real-Program Rust Emission (Lane B + D)

**Goal:** Compile real .dag programs to fully executable Rust. First
target: `gunbc/tools/review.dag` — a PR review agent using GitHub API,
LLM CLI backends, shell commands, and cron scheduling. This exercises
the full service/transport/workflow stack and serves as a stress test
for agent/LLM workflow patterns.

**Status:** More infrastructure exists than this section previously
documented. The pipeline already handles:
- `func` → `async fn` with `.await?` on effectful calls
- Service struct generation with config fields
- Service instance threading as extra function args
- String interpolation → `format!()`
- CLI entrypoint with clap (subcommand per `func`)
- `Cargo.toml` with reqwest/tokio/serde when `has_services`
- Dry-run mode with mock_response data
- REST, shell, and file transport call scaffolding

The gap: transport call bodies are scaffold-level — they don't consume
the detailed config (path templates, argv, HTTP method, query params)
already flowing to the emitter. The modeling is done; emission needs
to read what it already receives.

**First target: `review.dag`** — chosen because it's a stateless CLI
tool calling REST APIs and shell commands, which maps cleanly to .dag's
service model. The extdeps are already modeled:
- `extdeps/github/pulls.dag` (List, Get, Diff, CreateReview, ListReviews)
- `extdeps/llm/cli.dag` (Codex, Claude, Gemini via shell transport)
- `extdeps/shell.dag` (Exec.Run — `sh -lc {script}`)
- `extdeps/cron.dag` (Tab.Upsert — idempotent cron schedule)

### Dependency chain

```
RE-1: Transport emission fidelity (emit reads existing config)
  ├→ RE-2: review.dag compiles (dry-run)
  │    └→ RE-3: review.dag passes live integration
  └→ RE-4: Anthropic REST API end-to-end
       └→ RE-5: Multi-backend agent (CLI + REST switchable)
```

### RE-1: Transport emission fidelity

Make `emit_rest_call` and `emit_shell_call` consume the transport
config they already receive. PR #353.

**REST:**
- [ ] RE-1a: HTTP method from `transport.method` → `.get()`/`.post()`/etc.
  Inferred type enables structural dispatch; lowercases variant name for reqwest.
  Gap: emit reads variant name text, not the resolved HttpMethod alternative.
- [ ] RE-1b: Path template from `transport.path` with param substitution
  → `format!("/repos/{}/{}/pulls", owner, repo)`. Uses ExprStringInterp structure.
  Gap: interpolation segments still use expr_var_name_at, not full emit_typed_expr.
- [x] RE-1c: Query parameters from `transport.query`
  → `.query(&[("state", &state)])`. Uses emit_simple_expr for structural emission.
- [ ] RE-1d: Auth scheme from `config.auth` (Bearer vs Header("x-api-key"))
  Dispatches on ExprData shape (ExprVar=unit, ExprCall=payload).
  Gaps: collapses all unit variants (Bearer, ApiKey, Basic) to Bearer wire
  format; non-literal Header { name: expr } emits expr text as header name;
  ApiKey lacks structural carrier for key location. Needs resolved variant
  identity at the infer/emit boundary.
- [ ] RE-1e: Response code mapping from `response { 200 => ..., 401 => ... }`
  Status code match works but response/exit are encoded as synthetic property
  names — emitter re-parses strings. Needs first-class ResponseCase/ExitCase nodes.

**Shell:**
- [ ] RE-1f: argv from `transport.argv` with param substitution
  → `Command::new("sh").arg("-lc").arg(&script)`. ExprStringInterp structure.
  Gap: interpolation uses raw format!() with no shell escaping. Values with
  spaces, quotes, $(), backticks can break/inject commands. bash/emit.dag
  quoting facts modeled but not yet consumed by the emitter.
- [x] RE-1g: stdin from `transport.stdin`
  → `.stdin(Stdio::piped())` + stdout/stderr piped + spawn + write + wait.
- [ ] RE-1h: Exit code handling from `exit { 0 => ..., nonzero => ... }`
  Same structural gap as RE-1e: encoded as property names, not case nodes.

**Response:**
- [ ] RE-1i: `from "content/0/text"` JSON path extraction on response
  Not implemented. Nested path extraction deferred to RE-4.
- [ ] RE-1j: Nested output struct field mapping via serde rename
  `field_node_from_key` mechanism exists but no test covers it in this PR.

**Infrastructure (PR #353):**
- [x] Parser extended: `parse_rest_fields` captures method/path/query,
  `parse_shell_fields` captures stdin, `parse_config_fields` captures auth_input
- [x] Shell transport body marker preserved through resolve phase
  (`make_transport_node` body parameter fix)
- [x] `transport_env` excludes reserved keys (stdin separation)
- [x] `int_to_string_acc` digit ordering fix
- [x] HttpMethod moved from extdeps/transports/rest.dag to std/types.dag
- [ ] CloudAuthScheme dissolved: std/types.dag has protocol-level schemes
  (Bearer, Header, Basic, ApiKey). Cloud-specific SigV4/OidcToken need
  a Layer-2 cloud auth coproduct in extdeps/cloud that composes std/ schemes.
- [x] 7 RE-1 acceptance tests (5 REST + 2 shell)

**Blocked by:** Nothing — all data already flows to the emitter.

**Acceptance tests** (add to `v2-compiler-tests`):

```
# RE-1a: HTTP method
rest_emit_uses_transport_method
  Input:  service with `transport rest { method: GET, path: "/items" }`
  Assert: emitted Rust contains `client.get(` not `client.post(`

# RE-1b: Path template
rest_emit_substitutes_path_template
  Input:  service with `path: "/repos/{owner}/{repo}/pulls"`
  Assert: emitted Rust contains `format!("/repos/{}/{}/pulls", owner, repo)`

# RE-1c: Query params
rest_emit_includes_query_params
  Input:  service with `query: { state: state, per_page: per_page }`
  Assert: emitted Rust contains `.query(` with both params

# RE-1d: Auth scheme
rest_emit_uses_auth_scheme
  Input:  service with `auth: Header("x-api-key"), auth_input: api_key`
  Assert: emitted Rust contains `.header("x-api-key", ...)` not `Authorization`

# RE-1e: Response code mapping
rest_emit_maps_response_codes
  Input:  service with `response { 200 => Data, 401 => ErrorShape }`
  Assert: emitted Rust checks status code and deserializes accordingly

# RE-1f: Shell argv
shell_emit_uses_transport_argv
  Input:  service with `transport shell { argv: ["sh", "-lc", "{script}"] }`
  Assert: emitted Rust contains `Command::new("sh")` and `.arg("-lc")`

# RE-1g: Shell stdin
shell_emit_pipes_stdin
  Input:  service with `transport shell { argv: [...], stdin: prompt }`
  Assert: emitted Rust contains `Stdio::piped` and write to stdin

# RE-1h: Exit code
shell_emit_checks_exit_code
  Input:  service with `exit { 0 => Unit, nonzero => String }`
  Assert: emitted Rust checks `output.status.success()`

# RE-1i: JSON path extraction (general mechanism)
rest_output_from_clause_extracts_path
  Input:  operation with `output { value: String from "data/items/0/name" }`
  Assert: emitted Rust uses serde_json pointer or index chain to extract

# RE-1j: Nested output field mapping
rest_output_struct_uses_serde_rename
  Input:  operation with output fields whose JSON names differ from Rust names
  Assert: emitted struct has `#[serde(rename = "...")]` attributes
```

### RE-2: review.dag dry-run compilation

Compile `review.dag` + its imports to a binary that runs with
`--dry-run`, returning mock responses.

- [ ] RE-2a: Async for-each — detect FuncItem body, emit `.await?`
  in collection loop body
- [ ] RE-2b: Cross-module service resolution — review.dag imports
  from 4 modules (FF-9 handles this; verify it works for dsl/ tree)
- [ ] RE-2c: Conditional guard — review.dag's `already_done` check
  needs to short-circuit (`if`/`return` or restructured nesting)
- [ ] RE-2d: End-to-end compilation gate

**Blocked by:** RE-1

**Acceptance tests:**

```
# RE-2a: Async for-each
async_for_each_awaits_func_body
  Input:  `func f() -> Int { ... }` called inside `for x in items { f() }`
  Assert: emitted Rust contains `.await?` inside the for loop body

# RE-2d: End-to-end (ignored, CI gate)
review_dag_compiles_to_rust
  Run: compile dsl/gunbc/tools/review.dag + imports to Rust
  Assert: 0 hard diagnostics
  Assert: emitted files include main.rs with `review-pr` subcommand
  Assert: emitted Cargo.toml includes reqwest + tokio

# RE-2d: Emitted Rust compiles (ignored, CI gate)
review_dag_emitted_rust_builds
  Run: compile review.dag → write to /tmp → cargo check
  Assert: cargo check exits 0
  Note: same pattern as bootstrap_stage0_to_stage1
```

### RE follow-up: structural gaps from PR #353 review

Three design-level issues surfaced during review that need follow-up work:

**1. First-class response/exit case nodes (M4/M8/M9)**
The parser encodes `response { 200 => List<PullRequest> }` as synthetic property
names (`response_200`). The emitter re-parses these strings to rediscover patterns.
Fix: model as structural case arms — `ResponseCase { pattern: StatusPattern, body: Node }`
and `ExitCase { pattern: ExitPattern, body: Node }` — thread through parse/resolve/infer,
let emit consume directly. This removes the entire name-slicing class.

**2. Generic diagnostics/result carrier (M6/M9)**
Transport property inference introduced ad-hoc `InferPropertiesResult` and
`InferTransportResult` types. These should collapse into a generic result carrier
from `std/` that all inference helpers parametrize on, eliminating per-pass
boilerplate. Depends on std pattern work.

**3. Typed transport/auth accessors at infer/emit boundary (M4/M8)**
The emitter dispatches on expression shape (ExprVar vs ExprCall) and lowercased
variant names rather than resolved variant identity. The upstream fix: add typed
accessors or transport case nodes at the infer/emit boundary that preserve the
exact `HttpMethod` and `AuthScheme` alternative, so emit translates those directly
without reclassifying expressions.

### RE-3: review.dag live integration

Make the compiled binary work against real GitHub + LLM CLI.

- [ ] RE-3a: Auth token injection — `auth_token` from env var
  (`GITHUB_TOKEN`) or secret manager
- [ ] RE-3b: REST response deserialization — emitted struct fields
  match GitHub API JSON shape (serde attrs: `#[serde(rename)]`)
- [ ] RE-3c: Shell stdout capture for multi-line LLM output
- [ ] RE-3d: Cron upsert produces valid crontab entry

**Blocked by:** RE-2

**Acceptance tests:**

```
# RE-3a: Auth from env
service_auth_reads_env_var
  Input:  service with `auth_input: api_key` + func param `api_key: Secret`
  Assert: emitted CLI reads --api-key or GITHUB_TOKEN env var

# RE-3d: Cron emission
cron_upsert_emits_valid_crontab
  Input:  service call `cron.Tab.Upsert(tag: "x", schedule: "*/10 * * * *", ...)`
  Assert: emitted Rust writes crontab entry with tag-based dedup

# Integration (manual gate, not CI):
review_agent_dry_run_prints_mock_json
  Run: `./review-agent review-pr --owner gunb-ai --repo gunbc --pr-number 342 --dry-run`
  Assert: prints JSON with `{ "reviewed": true, "comment_url": "..." }`

review_agent_live_posts_review
  Run: `./review-agent review-pr --owner gunb-ai --repo gunbc --pr-number <N>`
  Assert: GitHub PR has new review comment
  Gate: manual — requires GITHUB_TOKEN + LLM CLI installed
```

### RE-4: Anthropic REST API end-to-end

Compile a .dag program calling `llm.Anthropic.Messages(...)` directly
via REST (not CLI wrapper). Exercises JSON body construction, response
path extraction, API versioning headers.

- [ ] RE-4a: JSON request body from input fields
  → `serde_json::json!({ "model": model, "messages": messages, ... })`
- [ ] RE-4b: `from "content/0/text"` path extraction on response JSON
- [ ] RE-4c: API version header from `current_api_version` data
- [ ] RE-4d: Secret handling — `api_key: Secret` → `x-api-key` header

**Blocked by:** RE-1

**Acceptance tests:**

```
# RE-4b: Anthropic response shape (uses RE-1i general mechanism)
anthropic_response_extracts_content_text
  Input:  `llm.Anthropic.Messages(...)` operation with `from "content/0/text"`
  Assert: emitted Rust extracts nested `content[0].text` from API response

# RE-4c: Custom headers
rest_emit_includes_api_version_header
  Input:  service with custom header config
  Assert: emitted Rust includes `.header("anthropic-version", "2023-06-01")`

# Integration (manual):
anthropic_messages_returns_response
  Run: .dag program sends prompt to Claude API, prints response text
  Assert: non-empty response string, valid token counts
  Gate: requires ANTHROPIC_API_KEY
```

### RE-5: Multi-backend agent (future)

A single .dag workflow that switches between CLI and REST backends
via config. Exercises backend-agnostic service dispatch.

**Blocked by:** RE-3, RE-4

**Acceptance test:**

```
multi_backend_review_agent
  Run: same workflow with --backend codex (CLI) and --backend anthropic (REST)
  Assert: both produce review output (different shape, same semantic result)
  Gate: requires both API keys + CLI tools
```

### RE ratchet

Track progress via a single ratchet: how many of the acceptance tests
above are green. Tests are to be added to `v2-compiler-tests` as each
RE item is implemented — the ratchet counts tests that exist AND pass.

| Metric | Current | Target |
|--------|---------|--------|
| RE-1 transport tests | 7/10 | 10 |
| RE-2 compilation tests | 0/3 (3 cargo check errors, ratchet at 3) | 3 |
| RE-3 integration tests | 0/4 | 4 |
| RE-4 API tests | 0/3 | 3 |
| RE-5 multi-backend test | 0/1 | 1 |
| Total | 0/21 | 21 |

**Depends on:** CG (codegen correctness) for reliable emission.
Parallel to CX (complexity doesn't block emission).

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
| M2 (Lane A) | `ExprLet` erases expected type → bare_map_node chain → 5+ downstream fallbacks | Propagate expected through ExprLet; normalize ExprVar→ExprCall for nullary |
| CG (Lane B) | 39 heuristic sites, all "existing authority not surfaced" | Surface connective through TypeRendering; surface AlgebraFieldTemplate to emit |
| CX (Lane C) | 4 analyzer heuristics = 4 missing std/ facts | Model operation size contracts in std/computation.dag |
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

# Structural Debt Scoreboard

**Goal: 0 structural debt.** The codebase is <20 compiler .dag files —
elimination is tractable.

**Current (2026-04-07): ~1,115 markers across 54 files.**

| Category | Count | Upstream cause | Dissolves when |
|----------|-------|---------------|----------------|
| Map<String,...> semantic keys | 537 | M4: no structural identity | M4 lands (structural refs replace string keys) |
| Positional projections (\|> first/last) | 298 | No list constructors | Cons/Nil or named projections in std/ |
| Fail-open fallbacks (_ => "") | 239 | No fail-closed strategy | M4 + exhaustive match enforcement |
| Heuristic name matching (.name == "X") | 37 | Dispatch on string literals | M4 + variant-based dispatch |
| Manual recursion (no bounded witness) | ~27 | No fold/repeat primitive | I1/I2 (repeat primitive) |
| String-prefix diagnostic dispatch | 4 | Fixed for CX (variant), emitted code remains | Full diagnostic variant coverage |

**Concentration:** emit_rust (262), complexity (202), infer (80), parse (68).
The top 5 files account for ~60% of all debt.

**Root cause correlation:** M4 (structural identity) is the upstream cause
of ~70% of debt. Map<String,...> keys (537) force fail-open lookups (239)
which force heuristic name matching (37). Fix M4 → ~800 markers dissolve.

**Emit dissolution path:** emit_rust's 262 markers are symptoms of
encoding language-specific decisions as imperative code. The correct
architecture: language specs in `dsl/extdeps/languages/` declare type
mappings, ownership rules, formatting conventions as structural facts.
A generic emitter reads those facts. Per-language emit files dissolve
into the spec declarations (P1-B: 3 backends → 1 parameterized
homomorphism, ~2,500 lines eliminated).

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
  **Stopgap: automate two-pass detection in `regenerate-stage0.sh`.**
  When the emitter changes its own output (e.g., main.rs template),
  pass 1 produces a binary that differs from the committed one. The
  script should detect this (pass-1 output != committed stage0), rebuild
  from pass-1 output, and run pass 2 automatically. Currently this
  requires manual hand-patching to break the cycle — fragile and
  error-prone (PR #341 CI failure was caused by this).
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

---

# Killer Features (KF)

These are capabilities that do not exist in any production system
today. They are the reason to use gunbc over writing Rust/Go/Python
directly. Each is grounded in the same structural property: .dag
programs are decidable, Node-bounded, and finite — so the compiler
can prove things that are undecidable in general-purpose languages.

## KF-1: Complexity Proof on Every Compile

**What:** Every function gets a proven asymptotic bound at compile
time. Not a lint. Not an optional analysis. A structural proof that
`find_duplicates` is O(n²) and `merge_sorted` is O(n).

**Why it doesn't exist:** General-purpose languages have undecidable
control flow (unbounded while, general recursion). .dag's three
iteration primitives (fold/descend/repeat) make the bound derivable
by construction.

**Example:**
```dag
fn find_duplicates(items: List<String>) -> List<String> {
  items |> filter(item =>
    items |> any(other => other == item)
  )
}
// Compiler: complexity: find_duplicates is O(n²)
//   filter(O(n)) × any(O(n)) = O(n × n)
```

**Status:** Partially working. The analyzer computes CostExpr for
all 1600 compiler functions. 526 get CostUnknown because descent
evidence doesn't propagate through Node trees and parser SCCs.
When the 3 structural fixes land (CX-NEXT), every function has a
proven bound and CostUnknown is deleted from the type system.

**Remaining work:**
- [ ] KF-1a: Node tree descent recognition (~230 direct violations)
- [ ] KF-1b: Parser SCC TokenPosition threading (~73 direct)
- [ ] KF-1c: Graph DFS worklist (2 direct, ~10 composed)
- [ ] KF-1d: Delete `CostUnknown` variant from CostExpr
- [ ] KF-1e: Re-enable CX gate as blocking (CX-E)

**Maps to:** CX lane (CX-NEXT). Same work, different framing — CX
frames it as "analyzer improvement," KF-1 frames it as "every program
gets a complexity certificate."

**Acceptance:**
```
every_function_has_complexity_bound
  Run: compile any .dag program
  Assert: ComplexityReport has 0 CostUnknown entries
  Assert: every function_class is one of O(1), O(n), O(n²), O(n log n), etc.
  Assert: CostUnknown variant does not exist in CostExpr enum
```

## KF-2: Reject Suboptimal Algorithms

**What:** The compiler refuses to compile code when a provably
cheaper equivalent exists. Not a suggestion — a compile error.

**Why it doesn't exist:** Requires (a) a decidable cost algebra with
ordering, (b) a catalog of equivalent operations with different costs,
and (c) the ability to compare implementations structurally. General-
purpose languages can't do (a) because cost is undecidable. .dag can
because all operations have declared CostShape.

**Example:**
```dag
fn has_match(items: List<String>, target: String) -> Bool {
  items |> filter(item => item == target) |> count > 0
}
// Compiler ERROR: suboptimal: filter(p).count() > 0 is O(n) allocation
//   + O(n) count. Use any(p) for O(n) with O(1) allocation.
//   Suggested: items |> any(item => item == target)
```

More examples:
```dag
// ERROR: sort then take first → O(n log n). Use min() → O(n).
items |> sort_by(x => x.score) |> first

// ERROR: map then filter → intermediate allocation.
// Use filter_map (single pass, no intermediate list).
items |> map(transform) |> filter(predicate)

// ERROR: nested any() is O(n²). Use Set lookup for O(n).
items |> filter(item => others |> any(o => o.id == item.id))
```

**Status:** NOT BUILT. The infrastructure is close:
- CostShape per method: declared in `std/algebra.dag` (done)
- CostExpr composition: `cost_seq`/`cost_mul` (done)
- Missing: cost ordering, call-site pattern detection, violation type

**Remaining work:**
- [ ] KF-2a: Cost ordering — `cost_dominates(a, b) -> Bool` on CostExpr
- [ ] KF-2b: Equivalence catalog — declare `(pattern, replacement, proof)`
  triples in `std/optimization.dag`. Pattern: structural method chain.
  Replacement: cheaper equivalent. Proof: `cost_dominates(replacement, pattern)`.
- [ ] KF-2c: Call-site analysis — detect method chain patterns in inferred
  AST, look up equivalence catalog, compare costs
- [ ] KF-2d: `SuboptimalCost` violation type in ComplexityViolation
- [ ] KF-2e: Diagnostic with suggestion — show original cost, replacement
  cost, and the equivalent code

**Maps to:** Exploratory direction "cost comparator." KF-2 makes it
concrete with an equivalence catalog and call-site analysis.

**Acceptance:**
```
reject_filter_count_when_any_exists
  Input: fn f(xs: List<Int>) -> Bool { xs |> filter(x => x > 0) |> count > 0 }
  Assert: compile ERROR with suggestion to use any()

reject_sort_first_when_min_exists
  Input: fn f(xs: List<Int>) -> Int { xs |> sort_by(x => x) |> first }
  Assert: compile ERROR with suggestion to use min()

accept_optimal_code
  Input: fn f(xs: List<Int>) -> Bool { xs |> any(x => x > 0) }
  Assert: compiles with no suboptimality warning
```

## KF-3: Automated Test Generation from Types

**What:** The compiler generates tests from type definitions. Add a
type → tests appear. No hand-written test code for structural
properties.

**Why it doesn't exist:** Requires (a) a finite, enumerable type
algebra, (b) canonical witness generation per type form, and (c) the
compiler itself as the test oracle. General-purpose languages have
open type systems (user-defined classes, traits, generics) that make
exhaustive enumeration impossible. .dag's types are compositional
products/coproducts over a finite kernel.

**Example:** Defining a type automatically generates:
```dag
type Shape
  = Circle { radius: Float }
  | Rect { width: Float, height: Float }
  | Triangle { a: Float, b: Float, c: Float }
```
The compiler generates:
1. **Witness tests:** One canonical value per variant (Circle{0.0},
   Rect{0.0, 0.0}, Triangle{0.0, 0.0, 0.0})
2. **Round-trip tests:** serialize → deserialize = identity, per target
3. **Emission tests:** each variant compiles to valid Rust/Go/Python
4. **Pattern exhaustiveness:** match on Shape covers all 3 variants
5. **Algebra law tests:** if Shape has algebra methods, test monoid/
   lattice laws with generated witnesses

**Status:** Level 0 done (coercion data → test assertions, ~48 tests
auto-generated). Levels 4-6 designed in `docs/testing-strategy.md` but
not implemented.

**Remaining work:**
- [ ] KF-3a: Witness generator — one canonical value per type form
  (primitive→zero, product→all fields, coproduct→each variant,
  optional→Some+None, collection→[]+[witness])
- [ ] KF-3b: Emission algebra enumerator — enumerate all
  `(NodeKind × TypeForm × Cardinality)` triples from .dag type defs
- [ ] KF-3c: Program synthesizer — one minimal .dag program per
  emission algebra element
- [ ] KF-3d: Cross-target compilation — synthesized programs compile
  to all 3 targets with 0 errors
- [ ] KF-3e: Algebra law tests — for types with algebraic structure,
  generate identity/associativity/commutativity checks from
  `std/algebra.dag` declarations
- [ ] KF-3f: Test receipt — `TestReceipt` from `std/verification.dag`
  as CI authority: what was proven, tested, generated, unknown

**Maps to:** Gate 6 (test generation), M3 (deferred milestone).

**Acceptance:**
```
new_type_generates_witness_tests
  Input: add `type Color = Red | Green | Blue` to a .dag file
  Assert: test suite grows by 3 witness tests (one per variant)
  Assert: no hand-written test code for Color

emission_algebra_coverage
  Run: enumerate all (NodeKind × TypeForm) pairs
  Assert: synthesized program exists for each
  Assert: all compile to Rust, Go, Python with 0 errors

algebra_law_tests_generated
  Input: type with Monoid algebra (e.g., List with concat)
  Assert: identity law test generated: concat([], x) == x
  Assert: associativity test: concat(concat(a,b), c) == concat(a, concat(b,c))
```

## KF-4: Cross-Language Equivalence Proof

**What:** Compile one .dag source to Rust, Go, and Python. The
compiler proves all three compute the same result for all inputs
by structural induction on Node depth.

**Why it doesn't exist:** No system compiles to 3+ targets with a
structural proof that outputs are equivalent. Transpilers (e.g.,
Haxe, Kotlin Multiplatform) compile to multiple targets but provide
no equivalence guarantee — bugs are found by running tests, not by
construction.

**Example:**
```dag
fn fibonacci(n: Int) -> Int {
  match n {
    0 => 0
    1 => 1
    _ => fibonacci(n: n - 1) + fibonacci(n: n - 2)
  }
}
// Compiler proves: fibonacci(10) in Rust == Go == Python
// by structural induction: base cases are literals (identical),
// inductive step preserves + semantics across all targets
// (IntegerArithmetic in LanguageSpec has identical semantics).
```

**Status:** Not started. The foundation exists:
- Rust backend compiles the full DSL tree (`full_dsl_compiles`).
  Go and Python backends exist and emit code, but are not yet
  gated by `full_dsl_compiles` (Rust-only currently)
- Coercion data maps .dag types to target types per language
- LanguageSpec declares per-target semantics

Missing: 3-target `full_dsl_compiles`, differential testing
infrastructure, semantic equivalence assertions, shared test oracle.

**Remaining work:**
- [ ] KF-4a: Shared test oracle — given a .dag function + inputs,
  compile to all 3 targets, run each, compare outputs
- [ ] KF-4b: Semantic equivalence assertions — declare which
  LanguageSpec properties preserve equivalence (integer overflow,
  float precision, string encoding)
- [ ] KF-4c: Divergence catalog — document where targets intentionally
  differ (e.g., Rust panics on overflow, Python has arbitrary-precision
  ints) and how .dag handles it (coercion or explicit constraint)
- [ ] KF-4d: CI gate — differential test suite runs on every PR

**Maps to:** Gate 3 (language parity), P1-B (3→1 homomorphism).

**Acceptance:**
```
cross_language_equivalence_fibonacci
  Run: compile fibonacci.dag to Rust, Go, Python
  Run: execute each with input n=20
  Assert: all three return 6765

cross_language_equivalence_full_suite
  Run: compile all .dag example programs to 3 targets
  Run: execute each with shared inputs
  Assert: outputs match (modulo documented divergences)
```

## KF-5: Decidable High-Level Language

**What:** .dag is one of the only decidable languages suitable for
real backend and frontend development. Every program provably
terminates. Not "probably" — provably, by construction.

**Why this matters:** Most decidable languages are academic
(Agda, Coq) or domain-specific (SQL, regex). They prove properties
but you wouldn't write a web service in them. Most practical
languages (Rust, Go, Python, TypeScript) are Turing-complete — the
halting problem is undecidable, so the compiler fundamentally cannot
prove termination, bound complexity, or guarantee resource usage.

.dag sits in the gap: decidable AND practical. You write API
services, data pipelines, agent workflows, CLI tools — the same
things you'd write in Python or Go — but the compiler can prove
things about your code that are literally impossible in those
languages. KF-1 (complexity), KF-2 (optimality), KF-3 (test
generation), and KF-6 (hardware) all follow from this property.

**How:** Three bounded iteration primitives replace unbounded loops:
- `fold` — structural descent over a collection (size decreases)
- `descend` — recursive with proof witness (tree shrinks)
- `repeat` — bounded by explicit fuel (counter decreases)

No `while(true)`. No general recursion. No `goto`. The compiler
can prove termination because the language makes non-termination
unrepresentable. This is not a restriction that limits what you can
write — it's a restriction that eliminates an entire class of bugs.

**Status:** DONE — this is a language property, not a feature to
build. The work is communicating it clearly and ensuring KF-1
(complexity proof) lands so the decidability is visible in every
compile.

**Acceptance:** The landing page explains: ".dag is a decidable
language. Every program provably terminates. The compiler proves
O(n) vs O(n²) for every function — something that is mathematically
impossible in Python, Go, or Rust."

## KF-6: Hardware Compilation Target (Verilog/SPICE)

**What:** Compile .dag programs to Verilog (digital circuits) and
SPICE (analog circuit netlists). The same source that compiles to a
Rust web service also compiles to an FPGA bitstream.

**Why it's possible:** .dag's decidability is exactly the property
that makes hardware synthesis viable. Turing-complete languages
can't compile to circuits because you can't synthesize unbounded
loops into fixed hardware. .dag's programs have:
- Known iteration bounds → circuit pipeline depth
- No dynamic allocation → fixed-size hardware
- Proven complexity → timing budget derivable from cost bound
- Products → parallel wires, coproducts → muxes, functions →
  combinational logic blocks

**Why it matters for the demo:** This is the most visceral proof
that decidability enables capabilities impossible in other languages.
"Here's a data pipeline. It compiles to a Rust binary. The same
source also compiles to a circuit you can run on an FPGA." Even if
the hardware target is basic at release, supporting it demonstrates
the architectural thesis.

**LLM leverage angle:** LLMs can write .dag code, which compiles to
Rust/Go/Python/Verilog — verified multi-target output from one
source. An LLM writing Verilog directly produces unverified HDL.
An LLM writing .dag produces code the compiler can prove correct,
bounded, and optimal before synthesis. This is a concrete use case
for agent workflows generating hardware descriptions.

**Example:**
```dag
// A simple FIR filter — compiles to Rust AND Verilog
fn fir_filter(
  samples: List<Float>,
  coefficients: List<Float>
) -> List<Float> {
  samples |> map(i =>
    coefficients |> fold(acc: 0.0, coeff =>
      acc + coeff * samples[i]
    )
  )
}
// Rust target: fn fir_filter(...) -> Vec<f64> { ... }
// Verilog target: module fir_filter(clk, samples, coefficients, out);
//   Compiler knows: fold is 4 cycles (|coefficients| = 4),
//   map is |samples| cycles, total pipeline depth = 4 × N
```

**Status:** NOT STARTED. Requires new RenderTarget + LanguageSpec.

**Remaining work:**
- [ ] KF-6a: `RenderTarget::Verilog` variant in `artifact.dag`
- [ ] KF-6b: Verilog `LanguageSpec` — type mappings (Int→reg[63:0],
  Bool→wire, List<T>→memory array), operator semantics (+ → add
  module), block syntax (always @, module/endmodule)
- [ ] KF-6c: Verilog coercion data in
  `dsl/extdeps/languages/verilog/types.dag` — TypeCheckpoint +
  InhabitantDecl for hardware primitives
- [ ] KF-6d: Clock/pipeline inference — derive clock cycles from
  complexity bound (O(n) → n-cycle pipeline, O(1) → combinational)
- [ ] KF-6e: SPICE netlist target — analog circuit description from
  arithmetic operations (adders, multipliers, filters)
- [ ] KF-6f: Basic emitter — `05_emit_verilog.dag` (or folded into
  parameterized homomorphism if P1-B lands first)
- [ ] KF-6g: End-to-end test: compile a .dag program to Verilog,
  run through Icarus Verilog simulation, verify output matches
  Rust execution

**Maps to:** LS lane (LanguageSpec), CG-3 (parameterized emission),
P1-B (3→1 homomorphism → 4→1 with Verilog).

**Acceptance:**
```
verilog_basic_arithmetic
  Input: fn add(a: Int, b: Int) -> Int { a + b }
  Assert: emitted Verilog contains `module add(` and `assign out = a + b`

verilog_pipeline_from_fold
  Input: fn sum(xs: List<Int>) -> Int { xs |> fold(acc: 0, x => acc + x) }
  Assert: emitted Verilog has N-stage pipeline (N = list bound)
  Assert: pipeline depth matches complexity bound

verilog_matches_rust_output
  Run: compile fir_filter.dag to Rust and Verilog
  Run: execute Rust version, simulate Verilog version (Icarus/Verilator)
  Assert: outputs match for same input data
```

## KF summary and dependency chain

```
KF-5 (decidability) ─── language property, DONE
  ├→ KF-1 (complexity proof) ←── CX lane
  │    └→ KF-2 (reject suboptimal) ←── equivalence catalog
  ├→ KF-6 (hardware target) ←── LanguageSpec + coercion data
  └→ KF-3 (test generation) ←── M3 + verification.dag
       └→ KF-4 (cross-language proof) ←── differential testing
```

Decidability (KF-5) is the root property that enables everything
else. KF-1, KF-3, and KF-6 are independent entry points. KF-2
requires KF-1. KF-4 requires KF-3.

| Feature | Effort | Impact | Priority |
|---------|--------|--------|----------|
| KF-5: Decidable language | DONE (communication only) | Foundational — the thesis | P0 |
| KF-1: Complexity proof | Medium (3 fixes, CX lane) | High — table stakes | P0 |
| KF-2: Reject suboptimal | Medium (catalog + analysis) | Very high — the "wow" demo | P1 |
| KF-3: Test generation | Large (witness gen + synthesis) | High — eliminates manual tests | P1 |
| KF-6: Hardware target | Large (new LanguageSpec + emitter) | Very high — visceral demo | P1 |
| KF-4: Cross-language proof | Large (oracle + divergence catalog) | Medium — impressive but niche | P2 |

---

# Public Release Gate

**Goal:** Ship gunbc as a public, demonstrable system. Every item
below is a non-negotiable gate — the release is the conjunction of
all of them. No partial credit.

## Gate 1: All Active Lanes Complete

Every Layer 2 root-cause track reaches its acceptance criteria.

| Lane | Gate condition | Current | Ratchet |
|------|---------------|---------|---------|
| M2 | No fabricated types, no BRIDGE, BND-1..4 landed | 0 real BRIDGEs (4 emitter template strings remain), BND open | `full_dsl_compiles` 0 diagnostics |
| CG | Every codegen decision from one structural authority | TLC-1/2/3 done, TLC-4 partial | `bootstrap_fixed_point` passes |
| M4 | `l1-ratchet.sh` = 0, `Node.name` deleted | L1=37 | `l1-ratchet.sh --check` |
| CX | User code gets proven bounds via type-derived strict descent (see CX launch gate below) | 524 violations in compiler code (non-blocking, internal debt) | User-facing: bounds on standard patterns. Internal: `strict_compile_diagnostic_count` tracked but not blocking |
| LS | All emitter decisions from spec-referenced data | Not started | No inline target-language knowledge in emitter |
| RE | review.dag compiles and runs (RE ratchet 21/21) | 0/21 | RE ratchet table |
| PERF | No test >2s, self-compile <30s, no OOM | ~6.5s, OOM fixed | `performance_ratchet` |

**Acceptance test:**
```
release_gate_all_lanes
  Run: all CI gates green simultaneously
  Assert: l1-ratchet=0, CX launch gate green, RE=21/21,
          bootstrap fresh, full_dsl 0 diags, perf <30s
```

## Gate 2: Structural Debt = 0

The Structural Debt Scoreboard reaches 0 across all categories.

| Category | Current | Gate |
|----------|---------|------|
| Map<String,...> semantic keys | 537 | 0 |
| Positional projections | 298 | 0 |
| Fail-open fallbacks | 239 | 0 |
| Heuristic name matching | 37 | 0 |
| Manual recursion | ~27 | 0 |
| String-prefix dispatch | 4 | 0 |
| Total | ~1,115 | 0 |

Dissolves via M4 (~800), CX (~27), LS (~262). Not independent
work — completing the lanes IS the debt elimination.

**Acceptance test:**
```
structural_debt_zero
  Run: scripts/structural-debt-count.sh (to be created — grep-based marker count)
  Assert: total = 0
```

## Gate 3: Language Parity and Correctness (KF-4)

All three target languages (Rust, Go, Python) produce correct,
equivalent output for the full .dag surface area. See KF-4 for
the cross-language equivalence proof design.

| Criterion | Test |
|-----------|------|
| All 3 backends compile the full DSL tree | `full_dsl_compiles` with each `RenderTarget` |
| Structural form coverage | Test generator emits one program per `(NodeKind × TypeForm × Cardinality)` triple; all 3 backends compile each |
| Expression semantics equivalence | Shared test inputs produce semantically identical output across backends |
| Coercion data coverage | Every `TypeCheckpoint` + `InhabitantDecl` in each language's `types.dag` has a corresponding test |
| No backend-specific emission logic | P1-B complete: 3 backends → 1 parameterized homomorphism |

**Acceptance tests:**
```
full_dsl_compiles_all_targets (to be added — currently Rust-only)
  Run: compile dsl/ tree to Rust, Go, Python
  Assert: 0 hard diagnostics for each target

structural_form_coverage_all_targets
  Run: test generator produces programs for each emission algebra element
  Assert: all programs compile for all 3 targets

language_parity_expression_equivalence
  Run: shared expression test suite compiled to all 3 targets
  Assert: outputs match (modulo language-specific formatting)
```

## Gate 4: Complexity + Suboptimality + Hardware (KF-1, KF-2, KF-6)

KF-1 (complexity proof), KF-2 (reject suboptimal), and KF-6
(hardware target) must all ship.

| Criterion | Test |
|-----------|------|
| 0 CostUnknown | `CostUnknown` variant deleted from `CostExpr` |
| 0 heuristic classifiers | No name-matching in complexity analysis |
| All descent from structural facts | `std/termination.dag`, `std/computation.dag`, `std/algebra.dag` are the only authorities |
| Gate blocking | CX gate re-enabled (CX-E), complexity failures = compile failures |
| Self-compile proves itself | The compiler's own 1600 functions all have proven bounds |
| Suboptimal rejection | Equivalence catalog in `std/optimization.dag` with at least 5 pattern→replacement rules |
| Verilog target | Basic arithmetic + fold pipeline compiles to valid Verilog |

**Acceptance tests:**
```
complexity_zero_violations
  Run: cargo test strict_compile_diagnostic_count -- --ignored
  Assert: 0 complexity diagnostics (not just 0 hard errors — 0 total CX)

cost_unknown_variant_deleted
  Assert: CostExpr enum has no CostUnknown variant (grep)

no_heuristic_classifiers
  Assert: 0 name-matching patterns in complexity.dag (grep for .name ==)

suboptimal_filter_count_rejected
  Input: fn f(xs: List<Int>) -> Bool { xs |> filter(x => x > 0) |> count > 0 }
  Assert: compile error with suggestion to use any()

verilog_basic_emission
  Input: fn add(a: Int, b: Int) -> Int { a + b }
  Assert: emitted Verilog compiles with Icarus Verilog (iverilog)
```

## Gate 5: Business Cases

Real programs compiled from .dag to working binaries. Each
demonstrates a distinct capability.

### BC-1: PR Review Agent (review.dag)

Covered by RE lane. A CLI tool that:
- Lists open PRs via GitHub API
- Fetches diffs and context files via shell
- Sends to LLM (Codex/Claude/Gemini) for review
- Posts review comments to GitHub
- Self-schedules via cron

**Acceptance:** `./review-agent review-cycle` reviews open PRs
end-to-end. RE ratchet 21/21.

### BC-2: Code Snapshot Tool (gist.dag)

Already modeled in `dsl/gunbc/tools/gist.dag`. A CLI tool that:
- Captures git diff or recent changes
- Creates GitHub Gist with formatted markdown
- Returns shareable URL

**Acceptance tests:**
```
gist_dag_compiles_to_rust
  Run: compile gist.dag + imports to Rust
  Assert: 0 diagnostics, emitted binary has gist/gist-diff/gist-recent subcommands

gist_agent_dry_run
  Run: ./gist-agent gist --dry-run
  Assert: prints mock gist URL
```

### BC-3: LLM API Client (anthropic.dag / openai.dag)

A .dag program that calls an LLM API directly (REST, not CLI
wrapper). Demonstrates service modeling, auth, JSON handling.

**Acceptance tests:**
```
llm_client_compiles
  Run: compile a .dag program using llm.Anthropic.Messages(...)
  Assert: 0 diagnostics, emitted Rust includes reqwest + auth header

llm_client_live
  Run: send a prompt, get response
  Assert: non-empty response text
  Gate: manual, requires ANTHROPIC_API_KEY
```

### BC-4: Self-Hosted CI Pipeline (stretch)

The compiler's own build/test/regen pipeline modeled in .dag and
compiled to a binary that replaces the current shell scripts
(`regenerate-stage0.sh`, `check-stage0-freshness.sh`, `l1-ratchet.sh`).

**Acceptance:** Shell scripts deleted, replaced by .dag-compiled
binary. BP-1 acceptance criteria met.

## Gate 6: Automated Test Generation (M3 + KF-3)

Tests are structural claims derived from .dag type definitions.
The compiler generates tests, not humans. See KF-3 for full design.

| Criterion | Test |
|-----------|------|
| Structural form coverage | One test per emission algebra element, auto-generated |
| Coercion round-trip tests | Generated from `TypeCheckpoint` × `InhabitantDecl` data |
| Algebra law tests | Generated from `std/algebra.dag` declarations |
| Cross-language equivalence | Same .dag input → all backends → compare outputs |
| Receipt schema | `TestReceipt` type (to be added to `std/verification.dag`) as CI authority |

**Acceptance tests:**
```
test_generator_produces_structural_coverage
  Run: test generator from emission algebra
  Assert: at least one test per (NodeKind × TypeForm) pair

generated_tests_all_pass
  Run: generated test suite
  Assert: 0 failures across all targets

test_receipt_validates
  Run: CI produces TestReceipt, validates against ratchets
  Assert: receipt covers all structural forms
```

## Gate 7: Demo Polish

The system should be demonstrable to a public audience. This means
clean documentation, compelling examples, and one "wow" moment.

| Item | What |
|------|------|
| Landing page / README | What gunbc is, why it exists, 30-second getting started |
| Example gallery | 3-5 .dag programs showing types, services, compilation |
| English-language error messages | Diagnostics readable by non-compiler-engineers |
| "Spice" demo | One impressive demonstration — e.g., compile a .dag agent workflow, show the generated Rust, run it live against a real API, show the complexity proof |

**Acceptance:** A new user can clone the repo, compile an example
.dag program to Rust/Go/Python, and run the result — in under
5 minutes, with no prior knowledge of the system.

## Release dependency chain

```
Gate 1 (lanes) ──┬──→ Gate 2 (debt=0) ──→ Gate 7 (polish)
                 ├──→ Gate 3 (parity)
                 ├──→ Gate 4 (complexity)
                 ├──→ Gate 5 (business cases)
                 └──→ Gate 6 (test generation)

Gates 2-6 can proceed in parallel once Gate 1 lanes reach
their respective acceptance criteria. Gate 7 is last because
polish depends on stable features.
```

## Release ratchet (master scoreboard)

| Gate | Status | Acceptance |
|------|--------|-----------|
| 1. Active lanes | IN PROGRESS | All lane ratchets green |
| 2. Structural debt | 1,115 → 0 | `structural-debt-count.sh` = 0 |
| 3. Language parity (KF-4) | PARTIAL | 3-target `full_dsl_compiles` + cross-language equivalence |
| 4. Complexity + suboptimality + hardware (KF-1, KF-2, KF-6) | 526 → 0 | `CostUnknown` deleted + catalog ships + Verilog emits |
| 5. Business cases | 0/4 | BC-1..4 acceptance met |
| 6. Test generation (KF-3) | NOT STARTED | Generated tests all pass |
| 7. Demo polish (KF-5) | NOT STARTED | 5-minute onboarding, decidability front-and-center |

## Killer feature ratchet

| Feature | Status | Gate |
|---------|--------|------|
| KF-5: Decidable high-level language | DONE (language property) | Landing page explains it |
| KF-1: Complexity proof on every compile | 526 unknown → 0 | CostUnknown deleted |
| KF-2: Reject suboptimal algorithms | NOT STARTED | 5+ rules in equivalence catalog |
| KF-3: Test generation from types | Level 0 done | Levels 4-6 implemented |
| KF-4: Cross-language equivalence proof | NOT STARTED | Differential test suite green |
| KF-6: Hardware target (Verilog/SPICE) | NOT STARTED | Basic arithmetic emits valid Verilog |
