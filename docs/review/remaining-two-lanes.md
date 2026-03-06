# Remaining Work: Two Lanes

> Generated: 2026-02-28
> Baseline: All P0 bugs fixed, tests green, clippy clean (pre-existing lowerer `too_many_arguments` excluded)

## Lane Split Rationale

**Lane A (Compiler + Binaries)** owns all Rust changes: compiler pipeline completion,
CLI generator, binary elimination, registry cleanup. Changes `core/`, `gunbc-app/src/`,
`lib/`.

**Lane B (DSL Authoring)** owns all `.dag` file creation: external dependency models,
service wiring prep. Changes `dsl/` only. Zero Rust changes. Can run fully in parallel
with Lane A.

No file overlap. No merge conflicts.

---

## Lane A: Compiler Pipeline + Binary Elimination

**Rust-only.** Files: `core/`, `gunbc-app/src/`, `lib/`.
**Goal:** Complete the compiler refactoring, unblock and execute binary elimination,
finish registry cleanup. Net ~4,000 LOC deletion.

### Phase A1: Critical Path (do first, sequentially)

| # | ID | What | Acceptance Criteria | Size |
|---|-----|------|---------------------|------|
| 1 | C10-full | **Complete ReturnExprCompute desugaring.** Two remaining gaps: (a) fn-call results in func bodies (`report = summarize(...)` → `return { report: report }`) don't create `__out:report` edges — wire fn-call result bindings as callable endpoints. (b) **Local variable refs in ExprCompute** (`has_local_refs` trap): `collect_expr_leaf_refs` skips local bindings (e.g., `check = shell.Codegen.Check()`) instead of wiring them as edges. Fix: pass local variable → producer map into `collect_expr_leaf_refs`, emit `LeafRef::LocalBinding { endpoint, port }`, wire the edge in `synthesize_expr_compute`. | `make install` works. `codegen.dag` return expressions resolve at runtime. Test asserts `Value::Str` not `Value::Skipped`. | L |
| 2 | C19 | **Restore passthrough enforcement.** Remove `Value::Skipped` fallback in `execute_with_declared_output_passthrough` (resolve.rs). Replace with `ExecError` for non-optional outputs. | `resolve.rs` returns `ExecError`. All tests still pass (requires C10-full first). | S |
| 3 | C20 | **CLI generator: profile, mode, subcommand.** Generated CLIs accept `--profile <name>` (enum from `available_profiles`), `--mode ensure\|verify`, subcommand dispatch for multi-func modules. `KEY=VALUE` parsing for infra-style tools. | Generated CLI for `pipelines/sdlc.dag` accepts `--profile`. Multi-func modules get subcommands. | L |

### Phase A2: Binary Elimination (after C20, parallelizable within)

| # | ID | What | Acceptance Criteria | Size |
|---|-----|------|---------------------|------|
| 4 | A1 | **Eliminate `sdlc.rs`.** Move param_source propagation. Delete 263-line binary. | `sdlc.rs` deleted. Generated binary works with `--profile unit_test --dry-run`. | S |
| 5 | A2 | **Eliminate `deps_config.rs`.** Delete 238-line binary. | `deps_config.rs` deleted. `gunbc-deps-config --mode=ensure` works. | S |
| 6 | A3 | **Eliminate `pipeline.rs`.** Move `query_ci_status()` to DSL func. Delete 384 lines. | `pipeline.rs` deleted. `gunbc-pipeline --depth 1` works. | M |
| 7 | A4 | **Eliminate `workflow.rs`.** Move plan rendering to DSL. Delete 716 lines. | `workflow.rs` deleted. `gunbc-workflow plan` and `run` work. | L |
| 8 | A5 | **Eliminate `infra.rs`.** 8 subcommands → DSL. Delete 1,056 lines. | `infra.rs` deleted. All 8 subcommands work. | L |
| 9 | A10 | **Delete `BinaryArgs`.** Remove `BinaryArgs` enum from `core/cli/src/binary_args.rs`. Clean orphaned support. | `BinaryArgs` deleted. Only `parse()` API remains. | S |

### Phase A3: Compiler Internals (independent of A2, can interleave)

| # | ID | What | Acceptance Criteria | Size |
|---|-----|------|---------------------|------|
| 10 | C1 | **Stdlib host + caching.** `OnceLock` cache for compiled fn bodies. `include_str!` for stdlib sources. Delete per-module compile wrappers. **Also: migrate compile-on-demand pattern** in justfile.rs, gitignore.rs, shared.rs, resource_defs.rs, catalog.rs, commands.rs to `include_str!` or `StdLibHost`-style embedding. (Hack #4 from review.) | `classify_callable()` never calls `compile_from_context()`. No `../../dsl` paths. No runtime filesystem access for DSL data. | L |
| 11 | C4 | **LoweringContext struct.** Group 8-11 lowerer params into `LoweringContext`. Delete 18 `#[allow(clippy::too_many_arguments)]`. | Zero `too_many_arguments`. All `.dag` compile. | L |
| 12 | C5 | **Integrate scope.rs.** Replace `detect_*_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` with `ScopedBody` callers. | `IfBranchSite` deleted. `scope.rs` has non-test callers. | M |
| 13 | C6 | **Extract transport derivation.** `transport.rs` module returning `TransportManifest`. | `add_service_transport_triplets` returns data, not mutates builder. | M |
| 14 | C13 | **Split mock_defaults.** Generic probing (~350 lines) → `core/test/`. Delete GCP blob (~230 lines). | `mock_defaults.rs` deleted. Auto-mock works from `core/test`. | S |
| 15 | C14 | **REST status-code checking.** `GenericRestParseOp` checks status before field extraction. | 401 → structured error (not "field missing"). | M |

### Phase A4: Registry Cleanup (independent)

| # | ID | What | Acceptance Criteria | Size |
|---|-----|------|---------------------|------|
| 16 | B2 | **Makegen registry → DSL (remaining).** Migrate `BuildConfig` and `ToolInfo` from `registry.rs` to DSL data. | `registry.rs` from current to ~400 lines. Generated Makefile identical. | M |
| 17 | B10 | **Clean shared.rs + justfile.rs.** Remove `ToolInfo`/`BuildConfig` dependencies. | No references to deleted types. | S |

### Phase A5: SDLC Fix

| # | ID | What | Acceptance Criteria | Size |
|---|-----|------|---------------------|------|
| 18 | BT-E1 | **Transport node deduplication.** Make `endpoint_use_count` global across all modules. | `gunbc-sdlc --dry-run` completes all 494 nodes. | M |

**Suggested execution order:**

```
A1: C10-full → C19 → C20                    [sequential, critical path]
A2: A1 → A2 → A3 (parallel: A4 → A5) → A10 [after C20]
A3: C1, C4 → C5 → C6, C13, C14             [start anytime, parallel with A1/A2]
A4: B2 → B10                                [start anytime]
A5: BT-E1                                   [start anytime]
```

**Total: 18 tasks, ~4,000 LOC net deletion**

---

## Lane B: DSL Authoring (External Dependency Modeling)

**DSL-only.** Files: `dsl/extdeps/`. Zero Rust changes.
**Goal:** Create the "what is X?" knowledge layer for all external systems.
Unblocks Phase 2 Lane 6 (Service Layer Completion).

### Phase B1: Core Models (no deps, start first)

| # | ID | What | Size |
|---|-----|------|------|
| 1 | ED-1 | **`extdeps/cloud/core.dag`** — `Region`, `AuthScheme`, `ServiceEndpoint`, `RateLimit`, `Credential`, `IdempotencyToken` | S |
| 2 | ED-2 | **`extdeps/github/core.dag`** — `Repository`, `User`, `RateLimit`, `AuthToken`, `ApiVersion`, `Pagination` | S |
| 3 | ED-6 | **`extdeps/llm/core.dag`** — `Message`, `Role`, `TokenUsage`, `StopReason`, `Temperature`, `MaxTokens` | S |

### Phase B2: GitHub + LLM (after B1 cores)

| # | ID | What | Size |
|---|-----|------|------|
| 4 | ED-3 | **`extdeps/github/issues.dag`** — `Issue`, `IssueState`, `Label`, `IssueEvent`, `IssueComment`, `Timeline` | M |
| 5 | ED-4 | **`extdeps/github/pull_requests.dag`** — `PullRequest`, `ReviewState`, `CheckStatus`, `MergeStrategy` | M |
| 6 | ED-5 | **`extdeps/github/gists.dag`** — `Gist`, `GistFile`, `GistVisibility` | S |
| 7 | ED-7 | **`extdeps/llm/anthropic.dag`** — `Model`, `ContentBlock`, `SystemPrompt`, `ThinkingConfig` | S |
| 8 | ED-8 | **`extdeps/llm/openai.dag`** — `Model`, `ResponseFormat`, `ToolChoice` | S |

### Phase B3: GCP (after ED-1)

| # | ID | What | Size |
|---|-----|------|------|
| 9 | ED-9 | **`extdeps/cloud/gcp/core.dag`** — `Project`, `ServiceAccount`, `OAuth2Scope`, `WifPool` | M |
| 10 | ED-10 | **`extdeps/cloud/gcp/storage.dag`** — `Bucket`, `Object`, `CasPrecondition` | M |
| 11 | ED-11 | **`extdeps/cloud/gcp/pubsub.dag`** — `Topic`, `Subscription`, `AckDeadline` | M |
| 12 | ED-12 | **`extdeps/cloud/gcp/iam.dag`** — `Role`, `Binding`, `Policy` | S |
| 13 | ED-13 | **`extdeps/cloud/gcp/secret_manager.dag`** — `Secret`, `SecretVersion`, `RotationSchedule` | S |
| 14 | ED-14 | **`extdeps/cloud/gcp/cloud_run.dag`** — `Service`, `Revision`, `TrafficSplit` | M |
| 15 | ED-15 | **`extdeps/cloud/gcp/sts.dag`** — `TokenExchange`, `SubjectTokenType`, `GrantType` | S |

### Phase B4: Git + Cargo (no deps)

| # | ID | What | Size |
|---|-----|------|------|
| 16 | ED-16 | **`extdeps/git.dag`** — `Commit`, `Branch`, `Remote`, `Ref`, `MergeStrategy`, `DiffStat` | M |
| 17 | ED-17 | **`extdeps/cargo.dag`** — `Package`, `Target`, `Profile`, `Feature`, `TestHarness` | S |

### Phase B5: AWS + Azure (low priority)

| # | ID | What | Size |
|---|-----|------|------|
| 18 | ED-18 | **`extdeps/cloud/aws/core.dag`** — `Arn`, `Region`, `SigV4`, `AssumeRole` | M |
| 19 | ED-19 | **`extdeps/cloud/aws/*.dag`** (5 files) — S3, IAM, Lambda, SecretsManager, SQS | L |
| 20 | ED-20 | **`extdeps/cloud/azure/core.dag`** — `Subscription`, `Tenant`, `ManagedIdentity` | M |
| 21 | ED-21 | **`extdeps/cloud/azure/*.dag`** (5 files) — Blob, Identity, ContainerApps, KeyVault, ServiceBus | L |

**Suggested execution order:**

```
B1: ED-1, ED-2, ED-6                [parallel, start immediately]
B2: ED-3:5, ED-7:8                  [parallel, after B1 cores]
B3: ED-9 → ED-10:15                 [sequential within GCP, after ED-1]
B4: ED-16, ED-17                    [parallel, no deps]
B5: ED-18:21                        [low priority, after B3 pattern established]
```

**Total: 21 tasks, ~2,500 LOC new DSL**

---

## Combined Timeline

```
Week 1:
  Lane A: C10-full → C19 → start C20     Lane B: ED-1:3 (core files) → ED-3:8

Week 2:
  Lane A: C20 → A1:A3, start C4          Lane B: ED-9:15 (GCP) → ED-16:17

Week 3:
  Lane A: A4:A5, C5 → C6                 Lane B: ED-18:21 (AWS/Azure)

Week 4:
  Lane A: A10, B2 → B10, BT-E1, C13:14
```

### Architectural Hacks Addressed (from review feedback)

| Hack | Status | What |
|------|--------|------|
| #1 PipeMethod spread | **Fixed** | `PipeMethod::as_str()` + `Display` + `FromStr` on enum. Three duplicate `pipe_method_name()` deleted. |
| #2 Silent fallbacks | **Fixed** | `resource_defs.rs` and `gitignore.rs` now `.expect()` on DSL compile. Fallback data deleted. |
| #3 `#[path]` illusion | **Documented** | Physical move requires extracting `use super::*` reverse dep first. Lane A task. |
| #4 compile-on-demand | **Partially fixed** | All callsites now fail-closed (no silent fallbacks). `include_str!` migration folded into C1. |
| #5 `has_local_refs` | **Documented** | Root cause of `make install` failure. Fix strategy in C10-full. |

### Red-Team Review #2 — New Tasks

These were identified by a second pass scanning for "fail-open" and "heuristic as semantics" patterns.
Add to Lane A as red-team debt tokens. Each must include an RT id and deletion criteria.

| # | ID | What | Deletion Criteria | Size | Lane A Phase |
|---|-----|------|-------------------|------|-------------|
| RT-N1 | **SplitLines exit-code semantics.** `ShellOutputParsing::SplitLines` returns empty list on non-zero exit. Conflates "no results" with "command failed." | Per-operation exit-code handling: `exit { allow: [0,1] on 1 => [] }` or at minimum only treat exit=1 + empty stderr as empty. | M | A3 |
| RT-N2 | **TrimStdout optionality is structural.** `Port::is_optional()` centralizes the `?` suffix check, but optionality should be a structural type property, not a string convention. | `is_optional: bool` field on `Port` (or `Type::Optional(Box<Type>)` in the type system). Delete `ends_with('?')` from `is_optional()`. | M | A3 |
| RT-N3 | **Fidelity Enum backward-compat path.** `enum_variant()` in `core/codegen/src/fidelity.rs` handles both `Value::Enum` and `Value::Str`. The `Value::Str` path is backward-compat for pre-C3 graphs. | Delete `Value::Str` arm once all compiled graphs use `Value::Enum` exclusively. Add a test that verifies no stdlib evaluation returns `Value::Str` for variant values. | S | A3 |
| RT-N4 | **Config block unknown keys swallowed.** Parser `_ => { self.advance(); }` in service config blocks silently consumes unknown configuration keys. | Emit a parse diagnostic for unknown config keys (warn or error). Known keys: `endpoint`, `auth`, `auth_input`. | S | A3 |
| RT-N5 | **Nested field-chain wiring workaround.** `dsl/cloud/gcp/credential.dag` has `extract_secret()` helper and alias output to avoid `cred.token.token` field-chain wiring. | Implement field-chain wiring (`x.y.z`) generically in the lowerer's expression-to-source resolution. Delete `extract_secret` and alias outputs. | M | A1 (part of C10-full) |
| RT-N6 | **`#[path]` crate boundary in core/resolve.** `core/resolve/src/service_ops.rs` uses `#[path = "...gunbc-app/src/resolve_service.rs"]`. Compiles the file twice, bloats incremental builds. | `git mv` the file. Extract `use super::*` dependencies into shared types. `gunbc-app` imports from `gunbc_resolve::service_ops`. | L | A3 |
| RT-N7 | **ExprCompute is runtime interpreter.** `ExprComputeOp` builds a `LoweredFnBody` and calls `evaluate_fn_body` at runtime — "interpreter inside runtime." | Either fully desugar expressions into DAG nodes (preferred), or make compute nodes first-class compiled node type. Delete `ExprComputeOp` when all return expressions are desugared. | L | A1 (subsumes C10-full) |

### Red-Team Review #4 — New Tasks

| # | ID | What | Deletion Criteria | Size | Lane A Phase |
|---|-----|------|-------------------|------|-------------|
| RT-N8 | **Enum construction heuristic.** Evaluator uses capitalization to detect variant constructors. Payload variants become `Value::Map` with `_variant` tag, unit variants become `Value::Enum`. Unify representation. | Evaluator consults a constructor table, not casing. `Value::Enum { ty, variant, payload }` replaces both representations. | M | A3 |
| RT-N9 | **Legacy enum string match fallback.** `match_pattern` Variant arm matches `Value::Str(s) if s == variant_name` as backward compat. | Delete legacy arm. Add warning if exercised. All compiled graphs use `Value::Enum`. | S | A3 |
| RT-N10 | **AST interpreter bypass in catalog/commands.** Worker A's `expect_string_field` walks raw AST instead of using compiler evaluator. | Route through `compile_from_context` + `serde_json::from_value`. Part of C1 (StdLibHost). | M | A3 |
| RT-N11 | **In-band JSON `__enum`/`__bytes` signaling.** `value_bridge.rs` hijacks JSON keys. External API payloads could collide. | Make `from_bridge_json` type-aware (take `&TypeId`). | M | A3 |
| RT-N12 | **Generated CLIs ignore unknown flags.** Unknown args silently dropped. User typos become "works but wrong." | Generated CLIs strict: unknown flag → error + help + exit 2. | S | A2 |
| RT-N13 | **State-machine backward mapping duplication.** `retry_target` hardcodes mapping that must match `std/state_machines.dag`. | Import/call `backward_target` from DSL, or generate backward edges from state machine definition. | S | B (DSL) |
| RT-N14 | **Stub functions in `pipelines/sdlc.dag`.** `generate_implementation_plan`/`get_pr_diff` are placeholders. | Implement or delete/feature-flag so stubs can't be called accidentally. | M | B (DSL) |

**Guardrail proposal** (for preventing reinfection):
- Ban-list for `core/` and `gunbc-app/src/`: `panic!` on user input, `unwrap_or(<default>)` on parse/type/lower paths, `eprintln!` as diagnostic, `ends_with("?")` optionality checks — require `// RT-xx temporary` comment with tracked task id.
- Workaround comments must include: RT id, corresponding row in gap-analysis-tasks.md, deletion criteria.

## What's Done (from this session)

| Item | Status |
|------|--------|
| P0-1,2 | Fixed (remap_expr_idents FieldAccess flattening) |
| P0-3 | Fixed (ratchet baseline 77→93) |
| P0-4 | Fixed (PipeMethod FromStr trait) |
| P0-5 | Partial (remap_expr_idents fix for FieldAccess, `has_local_refs` trap remains → C10-full) |
| C3 compat | Fixed (Enum/Str equality, Enum+string concat, Skipped→empty list) |
| C8 | Verified done (MockResponseDef deleted, hermetic accepted/ignored, error_cases gone) |
| C9 | Verified done (no panics in lowerer lib.rs) |
| C16 | Verified partial (metadata preferred, from_node_context fallback) |
| C18 | Verified done (looks_effectful_without_kind removed, no expiry plumbing) |
| Hack fixes (round 1) | PipeMethod consolidation (-75 lines), fail-closed resource_defs/gitignore, fallback data deleted |
| Hack fixes (round 2) | LowerError::InvalidFileOp (kill eprintln), Port::is_optional() (centralize optionality), 7 new RT tasks documented |
| Merge fixes | ci.rs import, stale Cargo.toml bins, codegen regen, workspace.dag, workspace_model, pragma_lint allowlist |
| Snapshots | makegen_expand.txt, module_graph.rs, corpus_modules.rs all updated |
