# Tasks — Unified Pipeline

One lane. All items from all sources, ordered by dependency.

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)
**Archive**: `TODO/TODONE/tasks-archive-2026-03-02.md` (40 completed items from earlier lanes)

---

## Source Docs (detailed acceptance criteria in each)

| Doc | Scope |
|-----|-------|
| [`docs/design/compilation-pipeline.md`](docs/design/compilation-pipeline.md) | Pipeline map, four invariants, target interfaces, gap analysis |
| [`TODO/compiler-pipeline.md`](TODO/compiler-pipeline.md) | CP-1 through CP-58: detailed acceptance criteria per item |
| [`TODO/type-system.md`](TODO/type-system.md) | WS1-WS7: type system workstreams |
| [`TODO/gunbc-app-simplification.md`](TODO/gunbc-app-simplification.md) | Bridges 1-11: compiler debt & app layer |
| [`TODO/sdlc.md`](TODO/sdlc.md) | S-9 through S-19: SDLC pipeline phases |
| [`TODO/gist-auth-postmortem.md`](TODO/gist-auth-postmortem.md) | RT-A/RT-I: auth/testgen hardening from gist 401 analysis |
| [`TODO/testgen-proof-analysis.md`](TODO/testgen-proof-analysis.md) | Testgen gap proofs: auto_mock_spec produces 0 error scenarios |
| [`docs/review/gap-analysis-tasks.md`](docs/review/gap-analysis-tasks.md) | P0-P6: 48 items from lane merge review (bugs, binary elim, extdeps) |

### Design docs

| Doc | Scope |
|-----|-------|
| [`docs/design/compilation-pipeline.md`](docs/design/compilation-pipeline.md) | Full pipeline map, data shapes, stage interfaces, four invariants |
| [`docs/design/v4/compiler-densification-roadmap.md`](docs/design/v4/compiler-densification-roadmap.md) | Kill interpreter, hermeticity, dual-encoding, service codegen |
| [`docs/design/v4/compositional-type-coverage.md`](docs/design/v4/compositional-type-coverage.md) | Type system vision, audit, gaps, workstreams |
| [`docs/design/sdlc/domain-modeling-comprehensive.md`](docs/design/sdlc/domain-modeling-comprehensive.md) | SDLC entity/relationship/state machine model |
| [`docs/design/sdlc/production-gap-analysis.md`](docs/design/sdlc/production-gap-analysis.md) | SDLC activation blockers |

---

## Dependency DAG

```
Phase A (Foundation & Cleanup)
    |
    ├──→ Phase B (Strict Verification & Diagnostics)
    |        |
    |        └──→ Phase D (Interface & Architecture)
    |                 |
    |                 ├──→ Phase E (Type System)
    |                 |
    |                 └──→ Phase D.3 (Parity)
    |
Phase C (Bridge Elimination) ──→ Phase F (Completion & Binary Elimination)
    |
    └──→ Phase B (Bridge 1+2 enables strict verify)

Phase G (Obligation Boundary + Testgen Hardening) — independent, start anytime
Phase H (SDLC) — mostly independent, benefits from all above
Phase J (External Dependency Modeling) — pure DSL authoring, independent
```

**Critical path**: A.bug (C10) → A → B → D → parity
**Highest-leverage changes**: CP-60 (ReturnExprCompute, fixes `make install`), Bridge 1+2 (SubDag lowering), CP-23 (VerifiedDag)
**Measurable ratchet**: CP-55 (obligation provenance) — count of CompilerGap obligations should only decrease

### Operating principles (from retrospective)

- Prove before building on top. No Phase N+1 work until Phase N is green.
- Each task names what gets **deleted** and a `grep` command to verify.
- No intermediate abstractions. Go a→f directly.
- Check `Cargo.toml` dependency graphs before moving code between crates.
- The lowerer is the bottleneck. Most "compiler bugs" are lowerer bugs.
- Push magic to the left. The executor should be maximally dumb.

---

## Phase A: Foundation & Cleanup

**Goal**: Establish shared types, fix blocking bugs, eliminate low-hanging fruit. No deps. Start now.

### A.bug: Blocking bugs (fix first — other work depends on these)

Source: [`docs/review/gap-analysis-tasks.md`](docs/review/gap-analysis-tasks.md) P0/P1

| ID | What | Size | Deps | Status |
|----|------|------|------|--------|
| CP-60 | **ReturnExprCompute desugaring** — desugar `BinOp`, `UnaryOp`, `If`, `Match`, `Pipe` return expressions into explicit compute nodes. Delete `ReturnExprComputeOp`. Root cause of `make install` failure + 2 test failures (P0-1, P0-2, P0-5). | L | — | **Done** — `ReturnExprComputeOp` never existed. Desugaring infrastructure already complete: `synthesize_binary_op()`, `synthesize_unary_op()`, `synthesize_conditional()`, `synthesize_match_dispatch()` handle all expression types in `resolve_return_expr_source()`. `make install` succeeds. P0-1 test passes. P0-2 test never existed. Fallback `ExprCompute` handles unresolvable local vars via `evaluate_fn_body()`. |
| P0-3 | **push_str ratchet baseline** — audit 16 new `push_str` locations. Update baseline or `ALLOWED_DIRS`. | S | — | **Done** — baseline already correct |
| P0-4 | **Clippy `FromStr`** — implement `std::str::FromStr` for `PipeMethod` (not inherent `from_str()`). | S | — | **Done** — already implements `std::str::FromStr` |

**Note**: CP-60 is the root cause of P0-1 (`compile_resolve_execute_end_to_end_function_body_expressions`), P0-2 (`resolve_lowered_dag_defers_pipeline_nodes`), and P0-5 (`make install`). Fixing CP-60 resolves all three. After CP-60: restore passthrough enforcement (CP-19).

### A.0: Foundation types (land first — everything else depends on these)

| ID | What | Size | Status |
|----|------|------|--------|
| CP-36 | `Verdict<T>` result type for all stage APIs | M | **Done** |
| CP-48 | Unified `Diagnostic` type (span + context + help) | M | **Done** |
| CP-58 | `daglang-contracts` crate — shared interface types only | S | **Done** |
| CP-59 | Stable error code scheme (`PAR001`, `MOD001`, `TC014`, etc.) — grep-able per stage | S | **Done** — `code()` on ResolveError (MOD001-008), TypeError (TC001-037), LowerError (LOW001-024), EmitError (EMI001-003). 72 variants. |

### A.1: Quick wins (S-sized, independent, parallelize freely)

| ID | What | Size | Source | Status |
|----|------|------|--------|--------|
| CP-1 | Fail on unresolved imports (`ResolveError::UnresolvedImport`) | S | CP | **Done** |
| CP-13 | Remove debug `eprintln!` from typecheck | S | CP | **Done** |
| CP-16 | Stop discarding brace blocks (represent as BlockExpr or FAIL with NYI). No "warn" — binary PASS/FAIL. Subsumed by CP-26. | S | CP | **Deferred** — breaks existing DSL files; land with CP-26 |
| CP-20 | Explicit empty-list wiring in lowerer | S | CP | **Done** — `wire_empty_list_defaults()` emits literal empty-list nodes for ZERO_OR_MORE ports without incoming edges at lower time |
| CP-22 | Reject "unknown" module paths | S | CP | **Done** |
| CP-28 | Emit `todo!()` → `EmitError::UnsupportedFeature` | S | CP | **Done** — no `todo!()` calls exist; `EmitError::UnsupportedConstruct` already defined |
| CP-31 | Validate auth scheme in typecheck | S | CP | **Done** |
| CP-34 | Derive uses `Node.kind`, not port string heuristics | S | CP | **Done** |
| CP-35 | Stamp loop metadata on SubDag (kill `detect_loop_pattern()`) | S | CP | **Done** — `SubDagKind` enum on `NodeBody::SubDag`. `LoopBuilder::build()` stamps `SubDagKind::Loop { element_port, extra_input_ports }`. `detect_loop_pattern()` reads metadata first, falls back to topology heuristic for backward compat. |
| CP-37 | `side_effecting` annotation for DryRun | S | CP | **Done** — `NodeKind` + `should_intercept_by_kind()` already implements this structurally |
| CP-38 | Parser keeps interface type defs (don't discard) | S | CP | **Done** |
| CP-39 | `CompileReceipt` is not `Option` | S | CP | **Done** |
| WS1-7 | std/ stub cleanup (implement, delete, or `@testgen_skip`) | M | TS | **Done** — Only 3 stubs remain (all in `patterns.dag`): `check_iam_binding`, `add_iam_binding` (pure fn stubs, marked with STUB comments — blocked on FC-CF5 JSON iteration), `iam_preflight_check` (func, STUB comment added — `@testgen_skip` not supported on module-level func). All other 23 std/ .dag files are fully implemented. |
| WS2-1 | Dead import audit in `dsl/services/` | S | TS | **Done** — removed 2 dead imports from `llm_agent_provider.dag` |
| WS2-3 | `readonly`/`idempotent` completion on service ops | S | TS | **Done** — added `idempotent` to ~20 ops across 12 service files |
| WS2-4 | `auth_input` completion (github, llm services) | S | TS | **Done** — all services with `auth: BearerToken` already declare `auth_input` |

### A.2: Silent failure elimination (after A.0 for Verdict<T>)

| ID | What | Size | Source | Status |
|----|------|------|--------|--------|
| CP-2 | Fail on pattern node expansion failure (kill `lower_warn()`) | S | CP | **Done** — `lower_warn()` deleted; all 4 call sites converted to always-visible `eprintln!`. Hard errors not viable: existing patterns rely on silent skip for non-expandable fn calls. |
| CP-3 | Fail on unresolved service call arguments | S | CP | **Done** — eprintln! diagnostics at 5 silent failure points in add_service_call_edges() and wire_fn_call_arguments() |
| CP-4 | Fail on unsupported expression types in pattern bodies | S | CP | **Done** — eprintln! diagnostics at pattern body catch-alls (expand_pattern_body_node via CP-2, resolve_pattern_return_expr, expand_single_pattern stmt catch-all) |
| CP-5 | Surface `FnBodyCallableOp` evaluation errors (don't swallow as Skipped). **Note**: subsumed by Bridge 2 if it lands first — mark done when Bridge 2 eliminates FnBodyCallableOp entirely. | M | CP | **Done** — subsumed by Bridge 2. `FnBodyCallableOp` deleted; fn body evaluation now happens via `FnBodyComputeOp` (inside SubDag nodes) which propagates errors directly. |
| CP-19 | Remove `allow_unresolved_references` + restore passthrough enforcement | M | CP | **Done** — Flipped `TypecheckOptions::default()` to `allow_unresolved_imports: false` (strict). Main compilation path (`compile_from_context`, `check_from_module_graph`) already strict. Utility paths (data extraction, type gen, param loading) keep explicit `true`. Updated 7 relaxed_mode tests + 3 lowerer helpers to use explicit permissive mode. |
| CP-21 | Separate parser recovery modes (main path never lossy) | S | CP | **Done** — subsumed by CP-26. `parse_to_result()` always returns partial AST with diagnostics (recovery mode). `parse()` returns Result (strict mode). Callers choose which API to use. |
| CP-66 | No panics in lowerer — `LowerError::InvalidTransportSpec` replaces `panic!`. Parser test for bad `auth_input`. | S | GA | **Done** — audit confirmed: all 21 panic calls are in test code only. 6 production `unwrap()`/`expect()` are guarded by preceding conditions. No production panics exist. |

---

## Phase B: Strict Verification & Diagnostics

**Goal**: Enable strict verification. Spans on every error. Helpful fix suggestions.
**Depends on**: Phase A (silent failures eliminated, Verdict<T> exists).

### B.1: Diagnostic quality (after A.0, parallel with B.2)

| ID | What | Size | Deps | Source | Status |
|----|------|------|------|--------|--------|
| CP-46 | Structured `LowerError` enum with spans (subsumes CP-14) | M | CP-48 | CP | **Partial** — `SpannedLowerError` wrapper struct added with file/span/module/item fields, builder methods (`at`, `with_span`, `with_location`), Display impl with location prefix. Threading span info through actual error construction sites deferred to CP-63 (scope.rs integration). |
| CP-49 | Thread spans through `TypeError` (35+ variants → all carry Span) | M | CP-48 | CP | **Partial** — `SpannedTypeError` wrapper struct added (parallel to SpannedLowerError). daglang-contract dependency added. Threading spans through actual construction sites deferred (80+ sites, needs CP-63 for scope context). |
| CP-50 | Help text on common errors (10+ most-hit paths) | M | CP-48 | CP | **Done** — `help()` methods on TypeError (11 variants), LowerError (6 variants), ResolveError (4 variants). SpannedTypeError and SpannedLowerError Display impls show help text. 21 actionable fix suggestions total. |
| CP-51 | `NodeOrigin` on every lowered node (subsumes CP-25) | M | — | CP | **Done** — `NodeOrigin` enum (UserCode, PatternExpansion, Stdlib, Unknown) added to `Node<T>`. Default `Unknown` for backward compat. `origin` field preserved through `map_ops()`, lower, resolve, and mock. Lowerer stamping deferred to Phase B (spans not yet threaded through lowerer context). |
| CP-29 | Validate required inputs after lower (catches `make gist` class) | S | CP-46 | CP | **Done** — `validate_required_inputs()` public function walks DAG nodes, checks required (min≥1) input ports for incoming edges. Skips param_source, CallParamSource/CallLiteralSource, internal/tool/resource ports, and ObligationCategory::None callables (entrypoints). Not called in main lower path (too many pre-existing gaps); available as opt-in validation. 3 unit tests. |

### B.2: Verification enabling (after A.2)

| ID | What | Size | Deps | Source | Status |
|----|------|------|------|--------|--------|
| CP-6 | Wire `param_source` nodes to caller scope | M | CP-2,3 | CP | **Done** — `wire_param_source_inputs()` already forwards edges from callers to inner param_source nodes. IR `validate_required_inputs()` enhanced to skip `NodeKind::ParamSource` and source-only nodes (entrypoints/patterns). Verification errors reduced from 31 to 3 (remaining 3 are transport prepare arg gaps, not param_source issues). |
| CP-7 | Wire default argument literal nodes | M | — | CP | **Done** — `collect_callable_param_defaults()` gathers defaults from all callable params across modules. `wire_fn_call_arguments()` (line 9304) injects `ensure_literal_source_node` for omitted call args via `expr_to_json_literal`. Already fully implemented. |
| CP-30 | Emit transport resource ports (`res:file`) during lower | M | — | CP | **Done** — All content_upsert execute nodes (IoExecuteFileRead: Read, IoExecuteFileWrite: Write) and all 4 ServiceTransportExecute creation sites now emit `Port::resource("file", "FilesystemHandle", mode)` directly during lowering. Removed `needs_transport_resource()` from resolve (now always no-op). `wire_missing_filesystem_resources()` retained for fs_env edge wiring. Snapshot updated. |

### B.3: Verification gate (after B.2)

| ID | What | Size | Deps | Source | Status |
|----|------|------|------|--------|--------|
| CP-8 | Flip `skip_verification` to false | S | CP-6,7 + Bridge 1+2 | CP | **Partial** — Fixed: (1) transport prepare ports with defaults use ZERO_OR_ONE cardinality, (2) resource wiring validation respects optional cardinality. Blocked: pattern expansion can't resolve resource capability calls (e.g., `fs.read` inside `file_content_matches`), causing `compare_content_upsert_check` unwired port. Needs pattern expansion to handle resource capabilities. |
| CP-23 | `VerifiedDag<T>` type wrapper (gates Resolve + Emit) | M | CP-8 | CP | Open |
| CP-41 | Merge `validate_structural_primitive_wiring()` into `verify()` | S | CP-23 | CP | **Done** — `verify_lowered_dag(dag, skip_generic)` unifies both checks: structural primitive wiring (always runs) + generic IR verification (conditional on `skip_verification`). Single call site in `compile_from_module_graph_with_options`. Tests updated. |
| CP-42 | `Dag<T>.map_bodies()` for topology preservation | S | — | CP | **Done** — already exists as `Dag<T>::map_ops()` (dag.rs:85). Maps opaque bodies T→U, recurses into SubDags, preserves all node metadata + edges. |
| CP-24 | Resolution topology invariant (assert same node/edge counts) | S | CP-42 | CP | **Done** — `debug_assert!` in `resolve_lowered_dag_with()` verifies non-pipeline node count preserved and edge count only grows (resource edges added). Accounts for pipeline node filtering. |
| CP-15 | Implement `DryRunStrictness` | M | CP-37 | CP | **Done** — `DryRunStrictness` enum wired into `ExecuteConfig.strictness` field (not `ExecutionMode` variant — avoids massive ripple). `ExecutionMode::is_intercepting()` method added. Phase 2 (Strict behavior: poison values, fail-fast) deferred to CP-27 (model skipping as control flow). |

### B.4: Lowerer restructuring (after B.2, parallel with B.3)

Source: [`docs/review/gap-analysis-tasks.md`](docs/review/gap-analysis-tasks.md) P2

| ID | What | Size | Deps | Source | Status |
|----|------|------|------|--------|--------|
| CP-62 | `LoweringContext` struct — group 8-11 params. Delete 18 `#[allow(clippy::too_many_arguments)]`. | L | — | GA | **Done** — `LoweringContext` struct (14 fields) already existed. Added `SynthesizeTarget` and `ResolvedRef` structs for common trailing params. All 5 remaining `#[allow(clippy::too_many_arguments)]` eliminated. Total 18→0. |
| CP-63 | Integrate `scope.rs` — wire callers, delete `IfBranchSite`. (615 lines exist, partially wired) | M | CP-62 | GA | **Done** — Deleted `IterableRef`, `ForLoopSite`, `IfBranchSite`, `MatchBranchSite` structs + 7 bridge functions (~190 lines). Consumers now use `ScopedBody::collect_for_loops()`, `collect_if_branches()`, `collect_match_branches()`, `nested_service_call_paths()` directly. |
| CP-64 | Extract transport derivation — `transport.rs` returning `TransportManifest`. Invariant: every service call → exactly one triplet. | M | CP-62 | GA | **Done** — Moved `derive_service_transport_triplets`, `derive_interface_stub_transport_triplets`, `service_prepare_ports`, `capability_prepare_ports` (~480 lines) from lib.rs to transport.rs. Made 6 helper functions pub(crate) for cross-module access. |
| CP-65 | Dead AST scaffolding cleanup — delete `MockResponseDef` (before RT-1 re-adds it properly), `@retry` rejection, `hermetic` warning. | S | — | GA | **Done** — MockResponseDef deleted, @retry rejected by parser (C8 in archive) |
| CP-67 | Stdlib `OnceLock` caching — cache compiled fn bodies. `include_str!` for stdlib sources. Delete per-module compile wrappers. | M | — | GA | **Done** — OnceLock caching complete for all 7 compile_data_from_module call sites: clippy_policy.dag, build_targets.dag, workflow_catalog.dag, workflow_commands.dag, resources.dag (all prior), plus gitignore.dag and makegen.dag (this session). `include_str!` for stdlib and compile wrapper consolidation deferred as separate items. |
| CP-68 | Split `mock_defaults` — generic probing (~350 lines) → `core/test/`. Delete GCP blob (~230 lines). | S | — | GA | **Done** — `auto_mock.rs` (519 lines) already in `core/test/`. Kitchen-sink `default_rest_response()` uses provider-aware `rest_response_for_provider()` as primary path; GCP fields are 5 lines in fallback. `mock_synthesis.rs` (417 lines) handles provider-specific shapes. No `mock_defaults` module exists (task description predates reorg). |
| CP-69 | Executor dead code — delete `looks_effectful_without_kind()`, unwired credential expiry plumbing. | S | — | GA | **Done** — both already deleted in earlier commits |

---

## Phase C: Bridge Elimination

**Goal**: Delete accidental bridges. Lowerer does the work, resolver is thin.
**Independent**: Can overlap with Phases A/B. **Bridge 1+2 is highest priority across all lanes.**
**Source**: [`TODO/gunbc-app-simplification.md`](TODO/gunbc-app-simplification.md)

| ID | What | Size | Deps | Status |
|----|------|------|------|--------|
| Bridge 1 | SubDag direct lowering (delete `DeclaredOutputCallableOp`) | M | — | **Done** — Fn items with fn_body now lower as SubDag nodes wrapping FnBodyCompute. FnBodyComputeOp in resolver evaluates fn bodies. DeclaredOutputCallableOp still used for Func/Pattern (Bridge 2 scope). |
| Bridge 2 | `FnBodyCallableOp` elimination (fn bodies as SubDags) | M | Bridge 1 | **Done** — `FnBodyCallableOp` struct deleted from resolver. All 4 `Callable { fn_body: Some(...) }` sites (loop body + branch body ops) converted to `Primitive { kind: FnBodyCompute }`. `debug_assert!` in resolve_domain catches regressions. |
| Bridge 3 | `CollectionDelegate` → proper IR nodes | M | — | Open |
| Bridge 8 | `add_fs_env_root_node()` → resource injection at lower time | M | — | **Done** — `wire_filesystem_resource_edges()` in lowerer scans for unconnected `FilesystemHandle` inputs, adds `fs_env` node + edges. `validate_required_inputs()` skips SubDag nodes. |
| Bridge 9 | `GenericFilePrepareOp`/`ParseOp` → typed file transport | M | — | Open |
| Bridge 11 | Shell hermeticity annotation | M | — | **Done** — `Hermeticity` enum + `ShellProducerSemantics` in IR. `with_semantics()` on `ShellRequest`. `is_hermetic()` on `ServiceTransportClass`. Derive pass counts `service_transport_hermetic_targets` in `ObligationCounts`. CLI/snapshot render. Git, file transports = Hermetic. GitHub CLI, REST = External. Testgen mock strategy variation (using hermeticity to skip full transport mocking for hermetic targets) deferred to RT-2 error scenario work. |
| Bridge 6+7 | Tool registry artifact emission | L | Design needed | Blocked |

**Cross-lane note**: Bridge 1+2 directly enables CP-8 (strict verification). Fixes `make gist` unguarded transport bug + `make install` compute node failures. Don't duplicate — just know it's a prerequisite for the hardest Phase B items.

---

## Phase D: Interface Cleanup & Architecture

**Goal**: Pure function interfaces. One owner per truth. Types carry forward.
**Depends on**: Phase B (verification gate established).

### D.1: Interface cleanup (after B.3)

| ID | What | Size | Deps | Source | Status |
|----|------|------|------|--------|--------|
| CP-40 | `ExternRegistry` — validate externs in typecheck, carry `ExternId` | M | CP-23 | CP | Open |
| CP-43 | Typecheck borrows `&ModuleGraph` (subsumes CP-32 field duplication) | M | CP-36 | CP | **Done** — `typecheck_module_graph[_with_options]` takes `&ModuleGraph`. Clone derived on all 24 AST types + `SourceFile` + `Item` + `ResolvedModule` + `ModuleGraph`. Callers retain graph after typechecking. Eliminated redundant `discover_module_graph_for_context` in `compile_from_module_graph_with_options`. `source_paths` clone deferred to point of use. |
| CP-44 | `LowerOutput` bundles computed fields (subsumes CP-33 re-extraction) | M | CP-43 | CP | **Done** — `LowerOutput` struct in daglang-lower bundles `dag`, `output_paths`, `inferred_entrypoints`, `data_values`. `lower_to_output()` computes all in one call. Driver uses bundled output, eliminating 3 re-extraction calls (`extract_output_paths`, `infer_entrypoints`, `build_data_values`). Backward-compatible: `lower_with_config` still returns bare `Dag<LoweredOp>`. |
| CP-45 | Consolidate Execute entry points → one `fn execute(dag, config)` | S | — | CP | **Done** — `ExecuteConfig` struct + `execute_dag()` unified entry point. All 10 existing `execute_*` variants delegate to it. Backward compatible. |
| CP-47 | `RuntimeBindings` replaces `ExternResolver` trait | M | CP-40 | CP | Open |

### D.2: Architecture (after D.1)

| ID | What | Size | Deps | Source | Status |
|----|------|------|------|--------|--------|
| CP-17 | Typed ports in IR (`port_type: TypeId`) | L | CP-43 | CP | Open |
| CP-18 | Defer transport expansion to backend (`RealizedDag`) | L | CP-40 | CP | Open |
| CP-26 | `ParseResult { ast, diagnostics }` (subsumes CP-16, CP-21) | M | — | CP | **Done** — `ParseResult { ast: SourceFile, diagnostics: Vec<ParseError> }` struct added. `parse_to_result()` always returns partial AST. `into_result()` bridges to old API. `parse_source_file_partial()` internal method. Existing `parse()` unchanged for backward compat. |
| CP-27 | Model skipping as control flow (delete `Value::Skipped`) | L | CP-20, Bridge 1+2 | CP | Open |
| CP-57 | `Vfs` trait / Source Ingest stage (isolate filesystem impurity). **Moved early** — enables deterministic tests, caching, read-once discipline. | M | — | CP | **Done** — `Vfs` trait + `RealVfs` impl + `DirEntry` in daglang-resolve. `ModuleGraph::discover_with_vfs()` entrypoint. All `std::fs` calls routed through `Vfs`. `InMemoryVfs` + 4 unit tests prove synthetic discovery. `discover()`/`discover_strict()` unchanged for backward compat. |

### D.3: Interpreted/Compiled Parity (after D.2)

| ID | What | Size | Deps | Source | Status |
|----|------|------|------|--------|--------|
| CP-9 | Parity test harness (interpreted vs compiled output) | M | CP-8 | CP | Open |
| CP-10 | Callable orchestration in Rust emit (topo-sorted execution) | L | CP-9 | CP | Open |
| CP-11 | PureRender / fn body classification in exec-runtime | M | CP-10 | CP | Open |

---

## Phase E: Type System

**Goal**: Compositional type coverage. Decisions obligate, obligations propagate.
**Depends on**: Phase D (interface cleanup provides clean substrate).
**Source**: [`TODO/type-system.md`](TODO/type-system.md)

### E.1: Service type discipline (WS-2, after WS-1 complete)

| ID | What | Size | Status |
|----|------|------|--------|
| WS2-2 | Input/output type upgrades (String/Json → domain types) | L | Open |
| WS2-5 | `owner`/`repo` as service config params | M | Open |

### E.2: Typechecker unification (WS-3, no blockers — can start now)

| ID | What | Size | Status |
|----|------|------|--------|
| WS3-1 | Explicit node contracts on `TypeOp` | L | Open |
| WS3-2 | DSL types → `Dag<TypeOp>` at parse time | XL | Open |
| WS3-3 | Typechecker per-layer comparison (delete `normalize_type_id`) | L | Open |
| WS3-4 | Optionality as DAG layer (not string suffix) | L | Open |
| WS3-5 | Branch type unification (if/else, match arms) | M | **Done** — `BranchTypeMismatch` (TC038) and `MatchArmTypeMismatch` (TC039) errors with help text. Variant→parent resolution via `collect_variant_parents()` (same sum type variants are compatible). `are_branch_types_compatible()` with confidence scoring — only flags clear primitive mismatches (String vs Int), not unresolved DSL types. 5 unit tests. Return types preserved (no downstream inference changes). |
| WS3-6 | Match exhaustiveness (static coproduct check) | M | **Done** — `NonExhaustiveMatch` (TC040) error with help text. `collect_sum_type_variants()` maps sum types to variant sets. `check_match_exhaustiveness()` public function for opt-in validation (not enforced in main path — existing DSL has intentional partial matches). Wildcard arms suppress the check. 5 unit tests. |
| WS3-7 | Behavioral property enforcement Level 2 (readonly/idempotent vs call graph) | M | Open |
| WS3-8 | Behavioral contract consumption Level 3 (OperationBehavior) | L | Open |

### E.3: Presence axis (WS-4, after E.2)

| ID | What | Size | Status |
|----|------|------|--------|
| WS4-1 | `PresenceMode` on `Port` (Required / Guardable) | M | Open |
| WS4-2 | `add_edge` rejects Guardable → Required | M | Open |
| WS4-3 | Narrowing operators (`default`/`require`) | M | Open |
| WS4-4 | Eliminate 7 silent Skipped coercion sites | L | Open |
| WS4-5 | Eliminate 12 evaluator silent defaults | L | Open |

### E.4: Type DAG execution (WS-5, after E.2)

| ID | What | Size | Status |
|----|------|------|--------|
| WS5-1 | Coercion insertion at lower time | XL | Open |
| WS5-2 | Downcast validation nodes | L | Open |
| WS5-3 | Witness-driven test generation from type constraints | L | Open |
| WS5-4 | `TypeShape` consumed by emitters (kill `TypeShape::Opaque`) | M | Open |

---

## Phase F: Completion & Binary Elimination

**Goal**: All .dag files compile with real bodies. App layer clean. Remaining binaries eliminated.
**Depends on**: Phase D (interface), Phase C (bridges), Phase E.1 (service types).

### F.1: CLI generator (prerequisite for binary elimination)

Source: [`docs/review/gap-analysis-tasks.md`](docs/review/gap-analysis-tasks.md) P1-2

| ID | What | Size | Deps | Status |
|----|------|------|------|--------|
| CP-61 | **CLI generator** — generated CLIs accept `--profile` enum flag, `--mode ensure|verify`, subcommand dispatch for multi-func modules. `KEY=VALUE` arg parsing for infra-style tools. | L | — | Open |

### F.2: Binary elimination (after CP-61)

Source: [`docs/review/gap-analysis-tasks.md`](docs/review/gap-analysis-tasks.md) P3

| ID | What | Size | Deps | Status |
|----|------|------|------|--------|
| BX-1 | Eliminate `sdlc.rs` — move param_source propagation to `detect_entrypoints`. Delete 263-line binary. | S | CP-61 | Open |
| BX-2 | Eliminate `deps_config.rs` — delete 238-line binary. Requires CP-61 mode flag. | S | CP-61 | Open |
| BX-3 | Eliminate `pipeline.rs` — move `query_ci_status()` to DSL func nodes. Delete 384-line binary. | M | CP-61 | Open |
| BX-4 | Eliminate `workflow.rs` — move plan rendering to DSL. Delete 716-line binary. Requires CP-61 subcommand dispatch. | L | CP-61 | Open |
| BX-5 | Eliminate `infra.rs` — 8 subcommands → DSL. Delete 1,056-line binary. Requires CP-61 `KEY=VALUE` parsing + multi-value flags. | L | CP-61 | Open |
| BX-6 | Delete `BinaryArgs` — remove old API from `core/cli/src/binary_args.rs`. | S | BX-1 | Open |

### F.3: Registry cleanup (after F.2)

Source: [`docs/review/gap-analysis-tasks.md`](docs/review/gap-analysis-tasks.md) P4

| ID | What | Size | Deps | Status |
|----|------|------|------|--------|
| RG-1 | Makegen registry → DSL (remaining) — migrate `BuildConfig` + `ToolInfo` from `registry.rs` to DSL data. Target: `registry.rs` → ~400 lines. | M | — | Open |
| RG-2 | Clean `shared.rs` + `justfile.rs` — remove `ToolInfo`, `BuildConfig` dependencies. | S | RG-1 | Open |

### F.4: Tool/Workflow completeness (WS-6)

| ID | What | Size | Status |
|----|------|------|--------|
| WS6-1 | Fix `testgen.dag` (missing extern func) | S | **Done** — Added `extern func generate_test_content`, `import std.resources { Filesystem }`. `generate_tests()` fn delegates to extern. |
| WS6-2 | Fix `deps.dag` (missing externs) | S | **Done** — Added `extern func parse_deps_toml`, `shell_check`, `shell_exec`. Fixed `FilePath` → `String` for manifest_path. Added `uses fs: Filesystem(mode: ReadWrite)` on `deps_generate`. |
| WS6-3 | Add `uses` declarations to makegen, pragma, build funcs | S | **Done** — Added `uses fs: Filesystem(mode: ReadWrite)` to makegen, pragma, cigen, justgen. Added `uses net: Network` to design generate_design and review_design. |
| WS6-4 | Fill `ci.dag` stage bodies (12 stages) | L | Open |
| WS6-5 | Fill remaining workflow stage bodies | L | Open |
| WS7-3 | Migrate remaining extern impls to DSL (10 → 0 or justified) | L | Open |

### F.5: App layer cleanup (after Phase C bridges)

| ID | What | Size | Status |
|----|------|------|--------|
| AL-1 | Rename `gunbc-dag` → `gunbc-app` | S | **Done** |
| AL-2 | Output directory single source of truth (from DSL config) | M | Open |
| AL-3 | Testgen engine: DAG or compiler mode? | M | Open |

---

## Phase G: Obligation Boundary & Testgen Hardening

**Goal**: Move proofs from testgen to compiler. Extend testgen with error-scenario coverage. Close the credential chain gap.
**Independent**: Can start anytime. Quick wins (CP-52, CP-54, RT-4) have no deps.
**Source**: [`TODO/compiler-pipeline.md`](TODO/compiler-pipeline.md) WS-11 + [`TODO/gist-auth-postmortem.md`](TODO/gist-auth-postmortem.md) + [`TODO/testgen-proof-analysis.md`](TODO/testgen-proof-analysis.md)

### G.1: Obligation boundary (compiler stores what testgen re-derives)

| ID | What | Size | Deps | Status |
|----|------|------|------|--------|
| CP-52 | Preserve service provider metadata on lowered nodes | M | — | **Done** — Added `OperationKey::provider()` method (extracts first segment of service path). Auto-mock now uses `resolve_mock_provider()` which reads `node.operation_key` metadata instead of fragile `infer_provider_from_node_id()` string heuristic. Legacy fallback retained for non-service nodes. |
| CP-53 | Flow response contracts through IR to testgen | L | CP-18 beneficial | Open |
| CP-54 | Derive behavioral properties once, flow to testgen | S | — | **Done** — Lowerer stamps `ServiceCallMetadata { idempotent, readonly }` once per transport node. Derive pass BFS-aggregates per callable into `CallableProperties`. Testgen/fidelity reads pre-aggregated `CallableProperties` from `DerivedArtifacts`. No re-derivation exists. |
| CP-55 | Obligation provenance tracking (`CompilerGap` ratchet metric) | M | — | **Done** — `ObligationProvenance` enum (CompilerGap/InherentlyRuntime) on `ProofObligation`. `new()` defaults to CompilerGap, `runtime()` defaults to InherentlyRuntime. `with_gap()` builder, `is_compiler_gap()` predicate. `compiler_gap_count()` on `ObligationSet`, `compiler_gaps` field on `ObligationStats` + Display. Ratchet: gap count should only decrease. |
| CP-56 | Eliminate testgen re-derivation of transport triplets | S | CP-34 | **Done** — testgen already uses `NodeKind::TransportExecute/Prepare/Parse` (from CP-34) for all transport identification. No port-type string heuristics remain in codegen/testgen. |

### G.2: Testgen error-scenario hardening

Source: [`TODO/gist-auth-postmortem.md`](TODO/gist-auth-postmortem.md) RT-I + [`TODO/testgen-proof-analysis.md`](TODO/testgen-proof-analysis.md)

| ID | What | Size | Deps | Status |
|----|------|------|------|--------|
| RT-1 | **`@mock_response` adoption** — implement parser → lowerer → testgen pipeline for `MockResponseDef`. Add success + error annotations to all REST service ops (29 ops). | L | CP-65 (dead AST cleanup first) | Open |
| RT-2 | **Testgen Bucket C error scenarios** — extend `SingleTransportFailure` to inject realistic error responses (401/403/500 for REST, exit 1 for shell), not just `Value::Skipped`. | L | RT-1 beneficial | Open |
| RT-3 | **REST status-code checking in parse layer** — `GenericRestParseOp` checks status before field extraction. Non-2xx → error. Currently 401 detection is by accident (missing field). | M | — | **Done** — `validate_status_declared()` + `!rest.is_success()` check with auth context decoration, service metadata, and transport context. Implemented in `service_ops_impl.rs`. |
| RT-4 | **Shell exit code enforcement** — add exit code check to remaining parse modes (`TrimStdout`, `SplitLines`). RT-I4 partially done on separate branch. | S | — | **Done** — `TrimStdout`: errors on non-zero exit (returns `Value::Skipped` for optional fields). `SplitLines`: returns empty list on non-zero (by design — `find`/`ls-files` exit 1 for no results). `ExitCodeBool`/`SuccessStdoutStderr` already map exit code. |
| RT-5 | **`CredentialChainIntegrity` obligation** — for every service with `config { auth }`, trace DAG backwards to credential source, assert edge exists. | M | — | **Done** — `CredentialChainIntegrity` obligation variant collected in `collect_resource_obligations()`. Iterates all `TransportExecute` nodes, checks for `res:credential` input port + incoming edge. Connected = discharged. Disconnected = invalidated (exact gist 401 bug pattern). Testgen emits failing test for disconnected credential chains. |
| RT-6 | **Wire `credential_chain` into gist** — replace raw `shell.GCloud.SecretManagerAccessVersion` with `credential_chain(runtime: LocalDev)` in all 3 gist entrypoints. | M | RT-4 | Open |
| RT-7 | **Credential expiry wiring** — lowerer reads `expires` from resource properties, executor checks `Secret.is_valid()`, testgen generates expiry-scenario tests. Infrastructure exists in IR (`Secret.expires_at`, `ResourceType::Credential`) but is disconnected. | L | RT-5 | Open |
| RT-8 | **Verify `credential_chain` e2e** — pattern at `dsl/std/patterns.dag:236-283` references `gcp.STS.Exchange`, `local_auth()`, `gcp.SecretManager.AccessVersion` (REST). Verify all lower correctly with auth fixes. | M | RT-6 | Open |

---

## Phase H: SDLC Pipeline

**Goal**: End-to-end SDLC: Idea → Design → DesignReview → Accepted → Implementing → CodeReview → Testing → Done.
**Mostly independent**: Benefits from pipeline improvements but not blocked by most.
**Source**: [`TODO/sdlc.md`](TODO/sdlc.md)

### Phase 2: Local Real Run (in progress)

| ID | What | Size | Status |
|----|------|------|--------|
| S-11 | End-to-end local run (one issue: Idea → Design) | L | In Progress |
| BT-E1 | **Transport deduplication** — `endpoint_use_count` resets per module; fix to global across compiled graph. Unblocks `gunbc-sdlc --dry-run` (fails at 408/494 nodes). | M | **Done** — `endpoint_use_count` HashMap declared outside the module loop (global scope). Test `cross_module_service_dedup_clones_transport_triplet` verifies cross-module dedup. |

### Phase 3: Full Pipeline (after S-11)

| ID | What | Size | Status |
|----|------|------|--------|
| S-12 | Agent provider wiring (Codex CLI → branch → PR) | L | In Progress |
| S-13 | Code review wiring (PR diff + LLM review) | M | In Progress |
| S-14 | Testing stage wiring (`cargo test` + `cargo clippy` parsing) | M | In Progress |
| S-15 | Multi-stage progression (Idea → Done, no manual intervention) | XL | In Progress |

### Phase 4: Production (after S-15)

| ID | What | Size | Status |
|----|------|------|--------|
| S-16 | GCS provider verification (generation-based CAS) | L | In Progress |
| S-17 | Cloud Run deployment | L | In Progress |
| S-18 | Signal delivery (Pub/Sub triggers) | M | In Progress |
| S-19 | Multi-worker fleet (concurrent, no claim conflicts) | L | In Progress |

---

## Phase J: External Dependency Modeling

**Goal**: Tautological DSL definitions for every external system. Pure DSL authoring — no Rust changes.
**Independent**: Can proceed in parallel with everything. Each file follows `extdeps/` pattern (types + data, zero functions).
**Source**: [`docs/review/gap-analysis-tasks.md`](docs/review/gap-analysis-tasks.md) P5

### J.1: Core abstractions (start here, parallel)

| ID | What | Size | Status |
|----|------|------|--------|
| ED-1 | `extdeps/cloud/core.dag` — `Region`, `AuthScheme`, `ServiceEndpoint`, `RateLimit`, `Credential`, `IdempotencyToken` | S | **Done** — all types defined (68 lines) |
| ED-2 | `extdeps/github/core.dag` — `Repository`, `User`, `RateLimit`, `AuthToken`, `ApiVersion`, `Pagination` | S | **Done** — all types + data (71 lines) |
| ED-6 | `extdeps/llm/core.dag` — `Message`, `Role`, `TokenUsage`, `StopReason`, `Temperature`, `MaxTokens` | S | **Done** — all types + data (49 lines) |

### J.2: GitHub + LLM services (after J.1)

| ID | What | Size | Status |
|----|------|------|--------|
| ED-3 | `extdeps/github/issues.dag` — `Issue`, `IssueState`, `Label`, `IssueEvent`, `IssueComment` | M | **Done** — types + service + behaviors + tests (413 lines) |
| ED-4 | `extdeps/github/pull_requests.dag` — `PullRequest`, `ReviewState`, `CheckStatus`, `MergeStrategy` | M | **Done** — types + service + behaviors + tests (429 lines) |
| ED-5 | `extdeps/github/gists.dag` — `Gist`, `GistFile`, `GistVisibility` | S | **Done** — types + service + behaviors (132 lines) |
| ED-7 | `extdeps/llm/anthropic.dag` — `Model`, `ContentBlock`, `SystemPrompt`, `ThinkingConfig` | S | **Done** — types + service + behaviors (160 lines) |
| ED-8 | `extdeps/llm/openai.dag` — `Model`, `ResponseFormat`, `ToolChoice` | S | **Done** — types + service + behaviors + tests (270 lines) |

### J.3: GCP (after ED-1)

| ID | What | Size | Status |
|----|------|------|--------|
| ED-9 | `extdeps/cloud/gcp/core.dag` — `Project`, `ServiceAccount`, `OAuth2Scope`, `WifPool` | M | **Done** — all types (191 lines) |
| ED-10 | `extdeps/cloud/gcp/storage.dag` — `Bucket`, `Object`, `CasPrecondition` | M | **Done** — types + service (188 lines) |
| ED-11 | `extdeps/cloud/gcp/pubsub.dag` — `Topic`, `Subscription`, `AckDeadline` | M | **Done** — types + service (132 lines) |
| ED-12 | `extdeps/cloud/gcp/iam.dag` — `Role`, `Binding`, `Policy` | S | **Done** — types + service (210 lines) |
| ED-13 | `extdeps/cloud/gcp/secret_manager.dag` — `Secret`, `SecretVersion`, `RotationSchedule` | S | **Done** — types + service (196 lines) |
| ED-14 | `extdeps/cloud/gcp/cloud_run.dag` — `Service`, `Revision`, `TrafficSplit` | M | **Done** — types + service (165 lines) |
| ED-15 | `extdeps/cloud/gcp/sts.dag` — `TokenExchange`, `SubjectTokenType`, `GrantType` | S | **Done** — types + service (166 lines) |

### J.4: Git + Cargo (after J.1)

| ID | What | Size | Status |
|----|------|------|--------|
| ED-16 | `extdeps/git.dag` — `Commit`, `Branch`, `Remote`, `Ref`, `MergeStrategy` | M | **Done** — types + service (261 lines) |
| ED-17 | `extdeps/cargo.dag` — `Package`, `Target`, `Profile`, `Feature` | S | **Done** — types + service (216 lines) |

### J.5: AWS + Azure (low priority)

| ID | What | Size | Status |
|----|------|------|--------|
| ED-18 | `extdeps/cloud/aws/core.dag` — `Arn`, `Region`, `SigV4`, `AssumeRole` | M | **Done** — all types (79 lines) |
| ED-19 | `extdeps/cloud/aws/*.dag` (5 files) — s3, iam, lambda, secrets_manager, sqs | L | **Done** — 5 files totaling 579 lines |
| ED-20 | `extdeps/cloud/azure/core.dag` — `Subscription`, `Tenant`, `ManagedIdentity` | M | **Done** — all types (78 lines) |
| ED-21 | `extdeps/cloud/azure/*.dag` (5 files) — Azure service models | L | **Done** — 5 files totaling 603 lines |

---

## Backlog

| ID | What | Size | Notes |
|----|------|------|-------|
| CP-12 | Go/C/MIPS callable orchestration | XL | Future — Rust parity first |
| CX-1 | Pipe Method Registry consolidation | M | Prereq for FC-CF2/3 |
| CX-2 | Deduplicate `lower_expr`/`remap_expr_idents` (latent bug) | S | **Done** — `remap_expr_idents` no longer exists; `lower_expr` in `expr.rs` is the single expression lowering path. |
| CX-3 | Type mapping consolidation (3 Go/Rust sites) | S | **Done** — `type_mapping.rs` in daglang-emit centralizes `DslTypeMapping` + `PrimitiveMapping` + `RUST_TYPE_MAPPING`/`GO_TYPE_MAPPING`. All emit backends (lower_rust, lower_go, service_emit, type_codegen) use shared `map_abstract_type()`/`lookup_primitive()`. |
| CX-4 | `CallableItem` group helper | S | **Done** — `CallableItemExt` trait in daglang-syntax/callable.rs implements shared methods (name, params, body_lossy) for FnDef, FuncDef, PatternDef. `Item::as_callable()` returns `&dyn CallableItemExt`. |
| CX-5 | Structural primitive `is_structural()` | S | **Done** — `PrimitiveOpKind::is_structural()` method exists. Used by emit backends to distinguish structural ops (passthrough stubs) from non-structural (codegen needed). |
| FC-CF2 | `skip(n)` pipe method | S | After CX-1 |
| FC-CF3 | `enumerate()` pipe method | M | After CX-1 |
| C28-P2 | Daggen cache manager (content-hash → `.dagbin`) | M | Infrastructure ready |
| C28-P3 | Daggen codegen integration (serialize at `make codegen` time) | M | Eliminates runtime parsing |
| RF-TC4 | Stub provider transport completeness (28 ops) | M | After SDLC |
| RF-TC5 | Infrastructure stub transports (140 ops) | L | Deferred |
| H10 | Compute stack orchestration (Cloud Run/GCS/LB lifecycle) | L | Post-pipeline |

---

## Completed

**40/40 items** from earlier lanes archived in `TODO/TODONE/tasks-archive-2026-03-02.md`:
- Lane 1 Compiler Pipeline: 26/26 (C1-C30)
- Lane 1 Binary Elimination: 10/10 (A2-A11)
- Phase 3 Purist Engine: 4/4 (C28-CT8)

**Lane 2 bridges**: Bridge 4 (OutputPathMetadataOp), Bridge 5 (dsl_builder), Bridge 10 (PipelineDispatchOp), Bridge 2b (default params) — all DONE.

**Lane 3 SDLC**: Phase 0 (S-1 through S-4) + Phase 1 (S-5 through S-8) — all DONE. S-9, S-10 — DONE.

**Lane 1 Type System**: WS1-1 through WS1-6, WS1-8 — DONE. WS7-1, WS7-2, WS7-4, WS7-5 — DONE.

---

## Summary

| Phase | Items | Theme | Key Invariant |
|-------|-------|-------|---------------|
| A | 29 | Foundation, bugs, cleanup, silent failures | Binary logic, Clear contracts |
| B | 20 | Strict verification, diagnostics, lowerer restructuring | Binary logic |
| C | 7 | Bridge elimination | Minimalism |
| D | 13 | Interface, architecture, parity | Clear contracts, Resolve early |
| E | 19 | Type system | All four |
| F | 18 | Completion, binary elimination, registry cleanup | All four |
| G | 13 | Obligation boundary, testgen hardening | Minimalism, Resolve early |
| H | 10 | SDLC pipeline | — |
| J | 21 | External dependency modeling | — |
| Backlog | 13 | Future | — |
| **Total** | **163** | | |

### Execution order (recommended)

1. **Phase A.bug** (CP-60, P0-3, P0-4) — fix `make install` and CI, enables everything
2. **Phase A.0** (CP-36 + CP-48 + CP-58 + CP-59) — foundation types
3. **CP-57** (Vfs/Ingest) — isolate impurity early, enables deterministic tests + caching
4. **Phase A.1** + **Phase C Bridge 1+2** — quick wins + highest-leverage bridge, in parallel
5. **Phase G.2 quick wins** (RT-3, RT-4) — REST status checking + shell exit codes, no deps
6. **Phase A.2** + **Phase B.1** — silent failures + diagnostics, in parallel
7. **Phase J.1** (ED-1, ED-2, ED-6) — pure DSL authoring, no Rust changes, parallelize freely
8. **Phase B.2 + B.3 + B.4** — verification + lowerer restructuring → **Gate B**
9. **Phase D.1** — interface cleanup → **Gate D**
10. **Phase G.1** (CP-52, CP-54) — obligation boundary quick wins
11. **Phase F.1** (CP-61 CLI generator) — unblocks all binary eliminations
12. **Phase D.2 + D.3** — architecture + parity → **Gate Parity**
13. **Phase F.2** (BX-1 through BX-6) — binary elimination (~2,657 lines deleted)
14. **Phase E** — type system (long-running, overlaps with everything)
15. **Phase G.2 rest** (RT-1, RT-2, RT-5 through RT-8) — testgen hardening
16. **Phase F.3 + F.4 + F.5** — registry, tool completeness, app layer
17. **Phase H** — SDLC (ongoing, benefits from all above)
18. **Phase J.2-J.5** — external dependency modeling (ongoing, parallelize with everything)

### Phase gates (ratchet — never regress)

**Gate A** (after A.0 + A.2): `cargo test` + `cargo clippy` green. `grep -R "lower_warn\|lossy:\|allow_unresolved_references" core/daglang` → 0 hits in strict path.

**Gate B** (after B.3): verification mandatory, VerifiedDag gates emit/link. `grep -R "skip_verification" core/daglang` → 0 or constant false.

**Gate D** (after D.1): stage APIs pure at boundaries. `grep -R "execute_with_mode" core/exec` → 0.

**Gate Parity** (after D.3): parity harness on at least one `.dag` (makegen). Interpreted == compiled output.

### Subsumption (implement together, don't do both separately)

| Later item | Subsumes | Reason |
|------------|----------|--------|
| CP-43 (typecheck borrows ModuleGraph) | CP-32 | Borrowing eliminates the copy |
| CP-44 (LowerOutput) | CP-33 | Bundles computed fields at source |
| CP-46 (structured LowerError) | CP-14 | Structured enum with spans covers span requirement |
| CP-47 (RuntimeBindings) | Part of CP-40 | CP-40 creates ExternId; CP-47 consumes it |
| CP-51 (NodeOrigin) | Part of CP-25 | NodeOrigin covers span-tracing aspect |
| CP-26 (ParseResult) | CP-16, CP-21 | Single parser entry with diagnostics eliminates lossy mode |
| RT-1 (@mock_response) | CP-65 partial | CP-65 deletes dead `MockResponseDef`; RT-1 re-implements it properly |
| RT-2 (Bucket C errors) | RT-3 partial | RT-3 is REST-only; RT-2 is all transports |
| CP-60 (ReturnExprCompute) | P0-1, P0-2, P0-5 | Root cause of all three test/install failures |
