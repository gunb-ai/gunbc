# Three-Worker Parallel Plan

**Status**: Active
**Date**: 2026-02-28
**Goal**: 3 workers with mutually exclusive file ownership, zero cross-worker
dependencies, each self-contained with clear acceptance criteria.

## Why Three (Not Four)

The lowerer (produces graphs) and the resolver (consumes graphs) share a
dependency: resolver enforcement (RT83) requires lowerer completeness (scope.rs
+ expr totality). Making one worker own the full compiler→resolver chain
eliminates this. The application-layer deletion splits cleanly into two workers
by file ownership (workflow/ vs makegen/+policy/+externs).

---

## File Ownership

```
Worker A:  gunbc-dag/src/bin/{sdlc,deps_config,pipeline,workflow,infra}.rs
           gunbc-dag/src/workflow/ (17 files)
           gunbc-dag/tests/workflow_*.rs (7 files)
           gunbc-dag/tests/infra_cli.rs

Worker B:  gunbc-dag/src/makegen/ (5 files)
           gunbc-dag/src/policy/ (2 files)
           gunbc-dag/src/extern_impls.rs
           gunbc-dag/src/resources.rs
           gunbc-dag/src/embedded_assets.rs
           gunbc-dag/src/docgen/
           gunbc-dag/src/{bootstrap,build,codegen,infra,gist,deps_tool}.rs (wrappers)
           gunbc-dag/tests/{tool_registration,makefile_parity,extern_ratchet}.rs

Worker C:  core/daglang/ (all 5 compiler crates)
           core/codegen/src/fidelity.rs
           core/codegen/src/testgen/
           core/exec/
           gunbc-dag/src/resolve.rs
           gunbc-dag/src/resolve_service.rs
           gunbc-dag/src/mock_defaults.rs
           gunbc-dag/src/testgen_dag/

Shared (read-only):
           gunbc-dag/src/lib.rs (module decls only)
           gunbc-dag/src/{dsl_builder,dsl_registry,fidelity,tool_runner,dry_run,fs_env}.rs
           gunbc-dag/src/bin/{ci,codegen_cli}.rs
           gunbc-dag/src/ci/
           dsl/ (DSL source files)
           tasks.md (status updates only)
```

**Zero shared write targets. Zero cross-worker dependencies.**

---

## Worker A: "Binary & Workflow Elimination"

### What this does

Deletes 5 hand-written binary entrypoints and the entire Rust workflow subsystem.
Replaces with CLI generator extensions (profile support, mode flags, subcommand
dispatch) and DSL data declarations for workflow metadata.

After this worker: every tool binary is generated from DSL metadata. The workflow
planner/executor lives in a generic `core/workflow/` crate. Adding a new workflow
requires zero Rust changes.

### Design references

- `docs/design/binary-elimination.md` — gap analysis for each binary
- `docs/design/v4/red-team-two-lane-plan.md` — Lane A details

### Tasks

| # | IDs | What | Acceptance Criteria | LOC |
|---|-----|------|---------------------|-----|
| A1 | RT59 | **Profile-aware CLI generation.** Expose `available_profiles` in `CompileOutput`. When profiles exist, CLI template generates `--profile <name>` enum flag with validation. `unit_test` profile auto-enables DryRun. | `cargo test -p daglang-cli -- compile_command_profile` passes. Generated CLI for `pipelines/sdlc.dag` accepts `--profile unit_test`. | +100 |
| A2 | RT58, RT60 | **Eliminate `sdlc.rs`.** Move `param_source_*` propagation into `detect_entrypoints()` or `BoundaryMocks`. Delete handwritten binary. Verify generated binary matches behavior. | `sdlc.rs` deleted. `cargo run -p gunbc-dag --bin gunbc-sdlc -- --profile unit_test --dry-run` works (generated binary). No manual param_source wiring. | -240 |
| A3 | RT61 | **Eliminate `deps_config.rs`.** Add `--mode ensure|verify` flag to CLI template for `content_upsert` workflows. Resource manifest update as post-execution hook. | `deps_config.rs` deleted. `gunbc-deps-config --mode=ensure` works (generated). | -210 |
| A4 | RT62 | **Eliminate `pipeline.rs`.** Move `query_ci_status()`, `query_pr_description()`, `query_issue_description()` into DSL func nodes using shell transport to `gh` CLI. | `pipeline.rs` deleted. `gunbc-pipeline --depth 1` works (generated). | -340 |
| A5 | RT63, RT64 | **Eliminate `workflow.rs`.** Implement subcommand dispatch in CLI generator: when a `.dag` module has multiple exported `func` items, generate one binary with subcommand dispatch. Move `render_plan_text`, `render_plan_json` into DSL fns. | `workflow.rs` deleted. `gunbc-workflow plan` and `gunbc-workflow run` work (generated). | -715 |
| A6 | RT65 | **Eliminate `infra.rs`.** 8 subcommands, most complex binary. Move spec rendering to DSL. Implement `KEY=VALUE` parsing + multi-value flags + `--execute` safety gate in CLI template. | `infra.rs` deleted. All 8 subcommands work via generated binary. | -1,055 |
| A7 | RT78 | **Workflow catalog → DSL data.** Create `dsl/config/workflow_catalog.dag` with `data` declarations for `WORKFLOW_VARIANTS` table and `default_claims_for_stage()`. Evaluate at runtime via DSL data import. | `catalog.rs` lines 1-400 deleted (keep `build_workflow_spec` which reads DSL data). Workflow count matches. | -400 |
| A8 | RT79 | **Unit commands → DSL data.** Create `dsl/config/workflow_commands.dag` with per-workflow command `data` records. Each unit → `{ program, args, description }`. | `unit_commands.rs` deleted. Workflow execution uses DSL-declared commands. | -425 |
| A9 | RT71 | **Extract generic workflow to `core/workflow/`.** Move 9 generic modules: planner (572), executor (415), admission (521), coordination (185), slo (303), projection (111), proof (91), errors (108), schema (165). gunbc-dag keeps `spec_builders.rs` (93) as thin adapter + `capabilities.rs` (231) + `global_plan.rs` (186). | New `core/workflow` crate compiles. gunbc-dag imports it. All workflow tests pass. | -2,470 moved, +50 adapter |
| A10 | RT66 | **Delete handwritten binary infrastructure.** Remove `BinaryArgs` from `gunbc-cli`. Remove `#[allow(clippy::disallowed_methods)]` annotations. Clean up orphaned support code. | `BinaryArgs` type deleted. No `#[allow(clippy::disallowed_methods)]` in generated binaries. | -300 |
| A11 | — | **Delete compensating tests.** 7 `workflow_*.rs` test files + `infra_cli.rs` that validated the deleted registries. Workflow behavior is tested via DSL testgen obligations. | Test files deleted. `cargo test --workspace` still passes. | -1,900 |

### Worker A total: **-8,400 net**

### Internal dependency chain
```
A1 (profile CLI) → A2 (sdlc) → A3 (deps_config) → A4 (pipeline)
                   A5 (workflow) → A6 (infra) → A10 (cleanup infrastructure)
A7 (catalog DSL) → A8 (commands DSL) → A9 (extract to core/) → A11 (delete tests)
```

---

## Worker B: "Registry & Extern Deletion"

### What this does

Deletes every Rust file that is a manual data registry, extern bridge, or thin
wrapper. Replaces with DSL `data` declarations. After this worker: adding a new
tool, new gitignore category, new resource definition, or new pragma policy
requires zero Rust changes — only a `.dag` file.

### Design references

- `TODO/TODONE/2026-Q1/design-eliminate-registration-lists.md` — full analysis of
  why each registry exists and how to eliminate it
- `docs/design/v4/red-team-two-lane-plan.md` — Lane A (A4, A5 details)

### Tasks

| # | IDs | What | Acceptance Criteria | LOC |
|---|-----|------|---------------------|-----|
| B1 | RT75 | **Gitignore patterns → DSL data.** Create `dsl/config/gitignore.dag` with gitignore category `data` declarations. The 14 categories in `gitignore.rs` become `data` records. Rendering stays in Rust (reads DSL data). | `gitignore.rs` data section deleted. Categories match. Generated `.gitignore` byte-identical. | -300 |
| B2 | RT80 | **Makegen registry → DSL data.** Move `BuildConfig`, `MetaTarget` definitions, manual `ToolInfo::workspace()` entries, and `default_core_workflows()` to DSL. Keep `ToolInfo::from_tool_def()` (DSL-derived) and Cargo command construction (~400 lines). | `registry.rs` reduced from 2,217 to ~400. Generated Makefile byte-identical. | -1,800 |
| B3 | RT74 | **Resource definitions → DSL data.** Move `REPO_SOURCE_INPUT_GLOBS`, `TESTGEN_INPUT_GLOBS`, output paths to `dsl/config/resources.dag`. `ResourceDef` construction reads DSL data. | `resources.rs` deleted. Resource freshness checks still work. | -340 |
| B4 | RT76 | **Docgen targets → DSL data.** Move `DOCGEN_READ_TARGETS` to `dsl/tools/docgen.dag` `data` declaration. | `docgen/mod.rs` data section deleted. Docgen reads from DSL. | -70 |
| B5 | RT77 | **Delete `policy/pragma.rs`.** DSL rendering functions already work (proven by tests). Delete the 546-line Rust mirror that duplicates them. | `policy/pragma.rs` deleted. `make pragma` produces byte-identical output. | -546 |
| B6 | RT23 | **Delete `extern_impls.rs`.** All 6 extern bridges are either (a) replaced by DSL `extern func` declarations (NF-7 enables this), or (b) their Rust impls deleted because DSL bodies work. For `render_tree` and `build_snapshot_content` (recursive — DSL can't express yet), convert to `extern func` and keep the Rust impl via inventory. | `extern_impls.rs` deleted. `lookup_extern_impl()` dispatch deleted. Inventory-based resolution for the 2 recursive externs. | -600 |
| B7 | RT81 | **Delete tool module wrappers.** `bootstrap/mod.rs`, `build/mod.rs`, `codegen/mod.rs`, `deps_tool.rs`, `infra/mod.rs`, `gist.rs` are all thin wrappers that call `build_dsl_graph()`. Replace with single generic lookup using structural entrypoint inference. | 7 wrapper files deleted. Tool construction uses `dsl_builder::build_dsl_graph_for_entrypoint()` directly. | -165 |
| B8 | — | **Delete `embedded_assets.rs`.** Dead after extern deletion. | File deleted. | -27 |
| B9 | — | **Delete compensating tests.** `tool_registration.rs` (registry sync), `makefile_parity.rs` (golden Makefile validated Rust registry), `extern_ratchet.rs` (extern bridge count). | 3 test files deleted. Behavior covered by DSL testgen obligations. | -1,090 |
| B10 | — | **Clean `makegen/shared.rs` + `makegen/justfile.rs`.** Remove references to deleted registries. Simplify to DSL-data consumers only. | Files reduced. No references to deleted types. | -300 |

### Worker B total: **-5,238 net**

### Internal dependency chain
```
B1 (gitignore DSL) → B2 (makegen registry DSL) → B10 (shared cleanup)
B3 (resources DSL) ─┘
B4 (docgen DSL)
B5 (pragma.rs) → B6 (extern_impls) → B7 (wrappers) → B8 (embedded_assets) → B9 (tests)
```

---

## Worker C: "Compiler Pipeline Refactor"

### What this does

Restructures the compiler pipeline into the Google-style "strict layer cake" with
pure function cores. Specifically:

1. **Lowerer**: monolith → composable pure-function modules
2. **Resolver**: fail-closed, extracted to `core/`, heuristics deleted
3. **Stdlib**: cached + hermetic, no runtime compilation
4. **Types**: pipe methods as enum, enums as values, leaf refs as typed structs
5. **Executor**: dead code deleted, passthrough enforcement restored
6. **Testgen**: extracted to `core/codegen/`

This is the "strangler refactor" — each piece is extracted, tested against the
existing corpus for parity, then the old code is deleted.

### Design references

- `docs/design/v4/lowerer-pure-function-refactor.md` — phase decomposition, target
  types, wave-by-wave migration, strangler strategy
- `docs/design/v4/red-team-two-lane-plan.md` — architectural principles (pure functions,
  clear errors, strong interfaces, no compile-then-eval, minimal language core)

### Target module layout (Google-style layer cake)

```
core/daglang/
  daglang-syntax/       # tokens, spans, AST, parser (no I/O)
    src/
      lib.rs            # AST types, PipeMethod enum (not string allowlist)
      parser.rs         # parse(source) -> Result<SourceFile, Vec<ParseError>>
      ast_utils.rs      # AST traversal helpers
  daglang-resolve/      # module graph / import resolution (I/O only here)
  daglang-typecheck/    # typed AST + type errors (no I/O)
  daglang-lower/        # typed AST -> IR (no I/O, pure functions)
    src/
      lib.rs            # public API: lower_typed_project() + re-exports
      context.rs        # LoweringContext (groups the 8-11 parameter tuples)
      callable.rs       # Phase 1: lower callables -> Vec<LoweredCallable>
      scope.rs          # ScopedBody analysis (replaces ad-hoc detect_*)
      transport.rs      # Phase 3: derive transports -> TransportManifest
      wiring.rs         # Phases 4-6: derive edges -> Vec<DerivedEdge>
      resource.rs       # Phases 7-8: derive resource lifecycle
      assembly.rs       # Final: assemble_dag(parts) -> Dag<LoweredOp>
      expr.rs           # LoweredExpr IR, LeafRef enum (not string sentinels)
      eval.rs           # Pure evaluator
      spec.rs           # Service operation spec types
  daglang-derive/       # derive extra metadata from IR (no I/O)
  daglang-emit/         # IR -> outputs (strings/files)
  daglang-driver/       # orchestration: parse->resolve->typecheck->lower->derive->emit
    src/
      lib.rs            # re-exports: Driver, CompileOptions, CompileError, CompileOutput
      driver.rs         # struct Driver + high-level compile methods
      errors.rs         # CompileError (structured, with Display formatting policy)
      options.rs        # CompileOptions, BoundServicesScope enum
      output.rs         # CompileOutput, EmittedArtifact
  daglang-cli/          # CLI-only, thin wrapper over driver
```

### Tasks

| # | IDs | What | Acceptance Criteria | LOC |
|---|-----|------|---------------------|-----|
| C1 | RT93 | **Stdlib host + caching.** Replace `compile_fidelity()` with `OnceLock<HashMap<String, LoweredFnBody>>`. Embed stdlib via `include_str!("../../dsl/std/fidelity.dag")`. Single `StdLibHost::eval_fn(module, fn_name, inputs)` interface. Delete per-module compile wrapper in `core/codegen/src/fidelity.rs`. | `classify_callable()` never calls `compile_from_context()`. No `../../dsl` disk path. Fidelity tests unchanged. | -80 |
| C2 | RT42 | **Pipe methods first-class.** `PipeMethod` enum in `daglang-syntax`. Parser resolves `expr \|> method()` to `Expr::PipeCall(PipeMethod::Map, ...)`. Delete `should_track_call_name()` string allowlist. Exhaustive match in typechecker and lowerer. Adding a pipe method → compile error at every consumer. | `should_track_call_name()` deleted. `PipeMethod` enum has all 20 methods. All `.dag` modules compile. | -80 |
| C3 | RT45, RT46 | **Typed enums end-to-end.** `Value::Enum { ty, variant }` for sum type values. Fidelity classification returns structured enum, not `Value::Str`. Delete `TestClass::parse()`, `FermiCost::parse()` string round-trips. Replace `unwrap_or(TestClass::Unit)` silent fallbacks with explicit error. | Zero `parse()` calls on classification results. Zero `unwrap_or()` fallbacks in fidelity. Classification test unchanged. | -60 |
| C4 | RT82, B1 | **LoweringContext + dead code deletion.** Create `LoweringContext` struct grouping param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, service_registry, profile_bindings, data_values. Thread `&LoweringContext` instead of 8-11 args. Delete 18 `#[allow(clippy::too_many_arguments)]`. Audit and delete all dead `_ => None` arms in wiring paths. | Zero `#[allow(clippy::too_many_arguments)]` in lowerer. Zero `_ => None` in wiring paths. All `.dag` modules compile. | -800 |
| C5 | B2 | **Integrate scope.rs.** Replace `detect_if_branches_in_stmts`, `detect_match_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` (300 lines) with `ScopedBody::from_stmts()` (589 lines, already written). Delete ad-hoc walk functions. | `IfBranchSite` type deleted. `detect_if_branches_in_stmts` deleted. `scope.rs` has non-test callers. DAG parity. | -300 |
| C6 | B3 | **Extract transport derivation.** Move `add_service_transport_triplets` + `clone_transport_triplet` + `ServiceTransportEndpoint` to `transport.rs`. Returns `TransportManifest` (pure data) instead of mutating builder. | `transport.rs` exists. `add_service_transport_triplets` takes `&LoweringContext`, returns `TransportManifest`. DAG parity. | +50 |
| C7 | B11, B12 | **Expr walker totality + typed leaf refs.** Every `match expr {` has explicit arms (no `_ => {}`). `LeafRef` enum: `Param { name, field, ty }`, `Callable { endpoint, port }`, `Service { endpoint, port }`. Delete `PARAM_REF_SENTINEL` string encoding. | Zero `_ => {}` in expression walkers. `PARAM_REF_SENTINEL` deleted. `LeafRef` enum has 3 variants. | -50 |
| C8 | RT84 | **Delete dead AST scaffolding.** Delete `MockResponseDef` (parser stub), `error_cases()` (empty trait method), `@retry` (unimplemented), orphaned `hermetic` field on `OperationDef`. Turn `hermetic` keyword into deprecation warning, then remove. | `MockResponseDef` type deleted. `error_cases()` deleted. Parser rejects `@retry`. `hermetic` warns. | -200 |
| C9 | RT38, RT39 | **No panics, no silent parse.** Replace `panic!` in `derive_file_spec()` with `LowerError::InvalidTransportSpec`. Replace silent `auth_input` parser advance with parse error. Add tests: unknown file op → clean error; `auth_input: "token"` → parse error. | Zero `panic!` on user-authored DSL in lowerer. Parser test for bad `auth_input`. | -10 |
| C10 | RT94 | **Resolve ReturnExprCompute split-brain.** Lowerer desugars complex return expressions into explicit DAG nodes (if/match → branch nodes, binop → compute node) so emit path never encounters `ReturnExprCompute`. Delete `MetadataOnly` classification. Delete `ReturnExprComputeOp` from resolver. | Zero `ReturnExprCompute` nodes in any compiled graph. `PrimitiveOpKind::ReturnExprCompute` deleted. | -100 |
| C11 | RT67, RT72 | **Move resolve_service.rs to core/.** 2,190 lines (100% generic, parameterized by `ServiceOperationSpec`). Target: `core/resolve/src/service_ops.rs`. Split `resolve.rs`: generic framework (~1,600) → `core/resolve/`, domain dispatch (~700) stays. | New `core/resolve/` crate. `resolve_service.rs` deleted from `gunbc-dag`. All resolver tests pass. | -3,790 moved |
| C12 | RT73 | **Move testgen to core/.** 5 files (2,177 lines) → `core/codegen/src/testgen/`. gunbc-dag keeps thin caller. | `testgen_dag/` deleted from `gunbc-dag`. Testgen works from `core/codegen`. | -2,177 moved |
| C13 | RT68 | **Split mock_defaults.** Generic probing/builder (~350 lines) → `core/test/src/auto_mock.rs`. Delete GCP-specific blob (~230 lines) — replace with DSL `data` for mock shapes. | `mock_defaults.rs` deleted. Auto-mock works from `core/test`. | -580 |
| C14 | RT89 | **REST status-code checking.** `GenericRestParseOp` checks HTTP status before field extraction. Non-2xx → parse error with status + body. Document ops that intentionally accept non-2xx. | 401 on GitHub API → structured error (not "field missing"). Test: mock 401 → error message includes status. | -20 |
| C15 | RT88 | **Fail-closed resolver audit.** All `_ =>` fallback arms in resolve.rs: classify as intentional vs accidental. Replace accidental with typed errors. Delete `passthrough_fallback_value()` (70 lines). | Zero undocumented fallback arms. `passthrough_fallback_value` deleted. | -100 |
| C16 | RT95 | **Transport class in node metadata.** Store `ServiceTransportClass` explicitly in lowered node metadata. Registry gen reads metadata, not `node_id.contains("shell")` substring. | `from_node_context` in `registry_gen.rs` reads node metadata, not string heuristics. | -40 |
| C17 | RT96 | **Kill `propagate_to_param_sources`.** Fix boundary detection: param source nodes are not separate entrypoints. Generated CLI auto-feeds params. | `propagate_to_param_sources` deleted. `detect_entrypoints` returns one port per input. | -100 |
| C18 | — | **Executor dead code.** Delete `looks_effectful_without_kind()` (dead after RT6). Delete unwired credential expiry plumbing (RT91 analysis: exists but nothing calls it). | Dead code deleted. `cargo clippy` clean. | -400 |
| C19 | RT83 | **Restore passthrough enforcement.** C4+C5+C7 fix the lowerer wiring for dag_util if/else branches. Restore hard `ExecError` for required output ports with no input (was suppressed for CI noise). | `resolve.rs` returns `ExecError` for required output with no wired input. CI clean (no more unwired dag_util branches). | +15 |

### Worker C total: **-8,822 net** (including moves to core/)

### Internal dependency chain
```
C1 (stdlib host) → C3 (typed enums)
C2 (pipe method enum)
C4 (context) → C5 (scope.rs) → C6 (transport) → C10 (split-brain)
C7 (expr totality + typed refs)
C8 (dead scaffolding)
C9 (panics + parse)
C11 (move resolve to core/) → C14 (REST status) → C15 (fail-closed audit) → C19 (restore RT83)
C12 (move testgen)
C13 (split mock_defaults)
C16 (transport metadata)
C17 (kill propagate)
C18 (executor dead code)
```

C1, C2, C4, C7, C8, C9, C11, C12, C13, C16, C17, C18 can all start immediately.

---

## Combined Endstate

| Area | Before | After | Delta |
|------|--------|-------|-------|
| `gunbc-dag/src/` | 22,767 | ~5,300 | **-17,467** |
| `gunbc-dag/tests/` | 5,791 | ~2,700 | **-3,091** |
| `core/workflow/` (new) | 0 | ~2,500 | +2,500 |
| `core/resolve/` (new) | 0 | ~3,800 | +3,800 |
| `core/daglang/daglang-lower/` | 15,033 | ~12,000 | -3,033 |
| `core/exec/` | 14,554 | ~14,100 | -454 |
| **Net** | | | **~-22,500** |

### What becomes impossible

| Defect class | Made impossible by | Worker |
|-------------|-------------------|--------|
| Adding a tool requires Rust changes | DSL-only tool registration | B |
| Adding a workflow requires Rust changes | DSL workflow catalog | A |
| Compiler panics on user input | `LowerError` variants | C |
| Silent wiring drops (`_ => None`) | Typed errors in lowerer | C |
| Runtime compile-then-eval | Cached stdlib host | C |
| Pipe method not in allowlist | `PipeMethod` enum | C |
| Enum values as string round-trips | `Value::Enum` | C |
| Interpreted/emitted runtime diverge | Desugar in lowerer | C |
| Silent success on HTTP 401 | Status-code checking | C |
| Transport class from node-id substring | Explicit metadata | C |
| Manual param_source propagation | Fixed boundary model | C |

### Zero cross-worker dependencies

Each worker's acceptance criteria depend only on their own files. Workers A and B
produce DSL data files that the compiler reads, but the compiler doesn't change
(that's Worker C). The DSL data files are new additions, not modifications to
files any worker owns.
