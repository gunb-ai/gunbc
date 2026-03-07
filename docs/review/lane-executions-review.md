# Lane Executions Review: Phase 1 vs tasks.md

Review date: 2026-02-28
Branches analyzed:
- `cursor/workflow-engine-structure-fbcf` (Worker A + Lane 4)
- `cursor/compiler-pipeline-design-7a27` (Worker C)
- `cursor/registry-externs-deletion-cf18` (Worker B)
- `main` (Blue Lanes 1 + 2)

## Executive Summary

| Lane | Tasks | Completed | Partial | Not Started | Completion % |
|------|-------|-----------|---------|-------------|--------------|
| Blue Lane 1 (SDLC) | 16 (BT1-12, BT-R1:3, BT-E1) | 15 | 0 | 1 | 94% |
| Blue Lane 2 (External Deps) | 21 (ED-1:21) | 0 | 0 | 21 | 0% |
| Red Worker A (Binary Elimination) | 11 (A1-A11) | 4 | 0 | 7 | 36% |
| Red Worker B (Registry Deletion) | 10 (B1-B10) | 8 | 2 | 0 | 80% |
| Red Worker C (Compiler Pipeline) | 20 (C1-C20) | 7 | 4 | 9 | 35% |
| Lane 4 (Domain Model Foundation) | 22 (DM-1:22) | 22 | 0 | 0 | 100% |

**LOC impact across branches:**
- Worker A + Lane 4: 69 files, +4,221/-2,897 (net +1,324)
- Worker B: 61 files, +2,794/-5,714 (net -2,920)
- Worker C: 41 files, +1,008/-1,533 (net -525)

**Critical finding:** Merging all three branches produces a build failure.
`expr_compute_tools_codegen_codegen_success_return_0` fails with "unbound variable: check"
— indicating cross-branch incompatibility in the lowerer's return expression handling.

---

## Lane 1: Blue Lane 1 — SDLC Activation

### Status: NEAR-COMPLETE (15/16 tasks done)

All BT1-BT12 and BT-R1:R3 are marked Done in tasks.md and verified on main.
Substantial DSL authoring (~3,600 lines across 20 .dag files).

| Task | Status | Evidence |
|------|--------|----------|
| BT1 (Compile SDLC pipeline) | Done | `dsl/pipelines/sdlc.dag` compiles (536 lines) |
| BT2 (Pipeline wiring) | Done | 3 stages wired in `workflows/sdlc.dag` |
| BT3 (Hermetic scenario) | Done | unit_test profile DryRun succeeds |
| BT4 (Per-stage handler tests) | Done | 8 handlers, structural checks |
| BT5 (Worker dispatch loop) | Done | dispatch_sdlc, claim lifecycle |
| BT6 (Transport declarations) | Done | 26 ops with DSL transport blocks |
| BT7 (Local integration) | Done | `#[ignore]` tests, local profile |
| BT8 (Full local scenario) | Done | Full lifecycle DryRun |
| BT9 (Testgen integration) | Done | 1400+ test fns generated |
| BT10 (CLI entrypoint) | Done | `gunbc-sdlc` binary works |
| BT-R1 (Testgen discovery fix) | Done | 9,710 test fns, 493 lines deleted |
| BT11 (SignalStore providers) | Done | pubsub + file providers |
| BT12 (ArtifactStore providers) | Done | gcs + inline providers |
| BT-R2 (Provider completion) | Done | Transport blocks, dead code deleted |
| BT-R3 (3 execution gaps) | Done | LLM mocks, JSON path, auth embedding |
| **BT-E1 (Transport dedup)** | **Pending** | `endpoint_use_count` still resets per-module in `daglang-lower/src/lib.rs:6019` |

### Assessment

Blue Lane 1 delivered everything through L7 (CLI entrypoint) and the three review
rounds (BT-R1:R3). The remaining BT-E1 is a real lowerer bug that causes
`gunbc-sdlc --dry-run` to fail at 408/494 nodes with duplicate transport edges.
This is a blocking issue for full SDLC execution but does not affect the
compilation/structural test levels (L0-L3).

### Risks

- BT-E1 blocks full DryRun (`gunbc-sdlc --dry-run` fails). Assigned to lowerer
  but nobody has picked it up.
- Horizon tasks (BT13-BT19, CT-1:4) are correctly deferred but have no owner.

---

## Lane 2: Blue Lane 2 — External Dependency Modeling

### Status: NOT STARTED (0/21 tasks done)

None of the ED-1:21 target files exist. The file layout described in tasks.md
(`extdeps/cloud/`, `extdeps/github/`, `extdeps/llm/{core,anthropic,openai}.dag`,
`extdeps/git.dag`, `extdeps/cargo.dag`) has zero deliverables.

| Target Directory | ED Tasks | Files Created | Status |
|------------------|----------|---------------|--------|
| `extdeps/cloud/core.dag` | ED-1 | 0 | Not started |
| `extdeps/github/{core,issues,pull_requests,gists}.dag` | ED-2:5 | 0 | Not started |
| `extdeps/llm/{core,anthropic,openai}.dag` | ED-6:8 | 0 | Not started |
| `extdeps/cloud/gcp/{core,storage,pubsub,iam,secret_manager,cloud_run,sts}.dag` | ED-9:15 | 0 | Not started |
| `extdeps/git.dag` | ED-16 | 0 | Not started |
| `extdeps/cargo.dag` | ED-17 | 0 | Not started |
| `extdeps/cloud/aws/core.dag` | ED-18 | 0 | Not started |
| `extdeps/cloud/aws/{s3,iam,lambda,secrets_manager,sqs}.dag` | ED-19 | 0 | Not started |
| `extdeps/cloud/azure/core.dag` | ED-20 | 0 | Not started |
| `extdeps/cloud/azure/{blob_storage,identity,container_apps,key_vault,service_bus}.dag` | ED-21 | 0 | Not started |

### Pre-existing extdeps (on main, not ED lane)

These files predate the ED lane and established the pattern:
- `dsl/extdeps/clippy.dag`
- `dsl/extdeps/make.dag`
- `dsl/extdeps/github_actions.dag`
- `dsl/extdeps/yaml.dag`

### Lane 4 extdeps (NOT part of ED lane)

Lane 4 (Domain Model Foundation) created extdeps files in non-overlapping
territory: `secrets/`, `coordination/`, `tools/`, `devenv/`, `llm/pricing.dag`,
`api/`. These are explicitly scoped as "categories ED doesn't cover" per tasks.md.

### Assessment

This is a completely unexecuted lane. All 21 tasks remain Pending. The ED lane
was designed as "what the SDLC scenario needs first" — GCP, GitHub, LLM models
that services would import. Without these, the Phase 2 Lane 6 (Service Layer
Completion) is blocked (SL-1:4 depend on ED-2:17).

### Risks

- Phase 2 Lane 6 is fully blocked: SL-1 depends on ED-2:5, SL-2 on ED-6:8,
  SL-3 on ED-9:13, SL-4 on ED-16:17.
- Lane 4 files exist but are vocabulary/models; without ED files, services
  have nothing to import from the knowledge layer.

---

## Lane 3: Red Worker A — Binary & Workflow Elimination

**Branch:** `cursor/workflow-engine-structure-fbcf`
**Stats:** 69 files, +4,221/-2,897

### Status: 36% COMPLETE (4/11 tasks)

Worker A correctly prioritized the non-blocking tasks (A7→A8→A9→A10→A11) since
A1-A5 are blocked on Worker C's C20 (profile/mode/subcommand CLI generation).

| Task | Status | Evidence |
|------|--------|----------|
| A1 (Eliminate sdlc.rs) | Not done | `sdlc.rs` still 263 lines. Blocked on C20. |
| A2 (Eliminate deps_config.rs) | Not done | `deps_config.rs` still 238 lines. Blocked on C20. |
| A3 (Eliminate pipeline.rs) | Not done | `pipeline.rs` still 384 lines. Blocked on C20. |
| A4 (Eliminate workflow.rs) | Not done | `workflow.rs` still 716 lines. Blocked on C20. |
| A5 (Eliminate infra.rs) | Not done | `infra.rs` still 1,056 lines. Blocked on C20. |
| **A7 (Catalog → DSL data)** | **Done** | `dsl/config/workflow_catalog.dag` created, `catalog.rs` loads from DSL |
| **A8 (Commands → DSL data)** | **Done** | `dsl/config/workflow_commands.dag` created, `unit_commands.rs` deleted |
| **A9 (Extract core/workflow/)** | **Done** | New crate with 14 modules (~2.5k lines), gunbc-app imports it |
| A10 (Delete BinaryArgs) | Not done | `BinaryArgs` still exists in `core/cli/src/binary_args.rs`. Was partially cleaned (308 → reduced but still present) |
| **A11 (Delete compensating tests)** | **Done** | 7 `workflow_*.rs` + `infra_cli.rs` deleted (-1,577 lines) |

### Assessment

Worker A executed the correct strategy: tackle the non-blocking chain (A7→A8→A9→A11)
while waiting for C20. The core/workflow/ extraction (A9) is substantial and clean.
A10 was partially addressed (BinaryArgs was reduced but not fully deleted — the
new `parse()` API was introduced alongside, but `BinaryArgs` enum remains for
compatibility).

The 5 binary eliminations (A1-A5, totaling ~2,657 lines) remain blocked on C20.
This is the documented prerequisite and the blocking dependency is legitimate.

### Risks

- C20 is only partially done (Worker C), so A1-A5 remain firmly blocked.
- A10 partial completion creates a dual-API situation (`BinaryArgs` + new `parse()`).
- Lane 4 work was bundled on this branch — clean separation for review requires
  mentally separating the 22 DM-* tasks from the 11 A-* tasks.

---

## Lane 4: Domain Model Foundation

**Branch:** `cursor/workflow-engine-structure-fbcf` (bundled with Worker A)

### Status: 100% COMPLETE (22/22 tasks)

All DM-1 through DM-22 delivered. Pure DSL authoring — no Rust changes.

| Group | Tasks | Files | Status |
|-------|-------|-------|--------|
| Part A: Standard Vocabulary | DM-1:5 | `std/{behavioral,rate_limit,coordination,errors,capability}.dag` | All done |
| Part B: Secret Providers | DM-6:10 | `extdeps/secrets/{core,gcp_secret_manager,github_secrets,env_file,vault}.dag` | All done |
| Part C: Coordination Stores | DM-11:14 | `extdeps/coordination/{core,gcs,postgres,sqlite}.dag` | All done |
| Part D: Tool Lifecycle | DM-15:17 | `extdeps/tools/{rust_toolchain,gh_cli,package_managers}.dag` | All done |
| Part E: Complementary | DM-18:21 | `extdeps/{devenv/devcontainers,llm/pricing,api/github_ops,api/gcp_ops}.dag` | All done |
| Part F: Interface Enrichment | DM-22 | 7 enriched interface files | All done |

### Assessment

Clean execution of pure DSL authoring work. All 22 files follow the established
pattern (types + data, zero functions). Interface files enriched with
`OperationBehavior` and `CapabilityRequirement` imports from std vocabulary.

### Quality Observations

- All files import from `std.behavioral`, `std.coordination`, `std.capability` as designed.
- Interface enrichment (DM-22) touches all 7 interfaces as specified.
- File sizes are reasonable (40-200 lines each), matching the tautological modeling pattern.
- No overlap with ED lane territory (cloud/, github/, llm/{core,anthropic,openai}).

---

## Lane 5: Red Worker B — Registry & Extern Deletion

**Branch:** `cursor/registry-externs-deletion-cf18`
**Stats:** 61 files, +2,794/-5,714 (net deletion: 2,920 lines)

### Status: 80% COMPLETE (8/10 tasks done)

| Task | Status | Evidence |
|------|--------|----------|
| **B1 (Gitignore → DSL)** | **Done** | `dsl/config/gitignore.dag` with 14 categories, `gitignore.rs` loads from DSL |
| B2 (Makegen registry → DSL) | Partial | MetaTarget migrated to `dsl/config/build_targets.dag`. BuildConfig and ToolInfo still in Rust `registry.rs`. |
| **B3 (Resources → DSL)** | **Done** | `dsl/config/resources.dag` with globs + output paths, `resource_defs.rs` loads from DSL |
| **B4 (Docgen → DSL)** | **Done** | `dsl/tools/docgen.dag` has `data read_targets` |
| **B5 (Delete pragma.rs)** | **Done** | No `policy/pragma.rs` found. DSL rendering path used. |
| **B6 (Delete extern_impls.rs)** | **Done** | File deleted. New `extern_ops.rs` handles DSL extern func declarations. |
| **B7 (Delete tool wrappers)** | **Done** | 7 wrapper modules deleted; callers now use direct `gunbc_resolve::builder::build_dsl_graph(...)` calls with `GunbcExternResolver` |
| **B8 (Delete embedded_assets.rs)** | **Done** | File deleted |
| **B9 (Delete compensating tests)** | **Done** | `tool_registration.rs`, `makefile_parity.rs`, `extern_ratchet.rs` all deleted |
| B10 (Clean shared.rs + justfile.rs) | Partial | Files load DSL data via `load_build_targets_data()` but still depend on `ToolInfo` and `BuildConfig` from Rust registry types |

### Assessment

Worker B achieved the best completion rate of the three red team workers. The
net -2,920 LOC deletion aligns with the -5.2k target direction. The extern
deletion chain (B5→B6→B7→B8→B9) was fully executed. The DSL data migration
chain (B1→B3→B4) was also complete.

The two partial items (B2, B10) are interrelated: `BuildConfig` and `ToolInfo`
remain in Rust because not all their fields have been migrated to DSL data
declarations. `registry.rs` is reduced but not to the ~400-line target.

### Additional Work Not in tasks.md

The branch also delivered significant transport layer improvements:
- `lib/transport/src/test_backend.rs` expanded by +510 lines (virtual backend enrichment)
- `basic_transports_integration.rs` expanded by +132 lines
- Extern call lowering in `daglang-lower/src/lib.rs`
- Module graph dependency snapshot refresh
- Config DAG files added to syntax corpus

These are valuable but not tracked against Worker B's official task list.

---

## Lane 6: Red Worker C — Compiler Pipeline Refactor

**Branch:** `cursor/compiler-pipeline-design-7a27`
**Stats:** 41 files, +1,008/-1,533 (net deletion: 525 lines)

### Status: 35% COMPLETE (7 done, 4 partial, 9 not started)

| Task | Status | Evidence |
|------|--------|----------|
| C1 (Stdlib host + caching) | Not done | No `OnceLock` cache, no `StdLibHost::eval_fn()` |
| **C2 (PipeMethod enum)** | **Done** | `PipeMethod` enum in `daglang-syntax`, `should_track_call_name` deleted |
| **C3 (Typed enums)** | **Done** | `Value::Enum { ty, variant }` in `core/ir/src/value.rs` |
| C4 (LoweringContext) | Not done | No `LoweringContext` struct, `too_many_arguments` pragmas remain |
| C5 (scope.rs integration) | Partial | `scope.rs` exists (615 lines) but `detect_*_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` still in lib.rs |
| C6 (Transport derivation) | Not done | No `transport.rs` module, no `TransportManifest` type |
| **C7 (Typed leaf refs)** | **Done** | `LeafRef` enum with `Param`, `Callable`, `Service` variants in `expr.rs` |
| C8 (Dead AST scaffolding) | Partial | Some cleanup but `MockResponseDef` and `hermetic` references remain |
| C9 (No panics, no silent parse) | Not done | No `LowerError::InvalidTransportSpec`, no `auth_input` parse guard |
| C10 (ReturnExprCompute) | Not done | `ReturnExprCompute` still referenced, `ExprCompute` exists but replacement incomplete |
| **C11 (resolve_service → core/)** | **Done** | `core/resolve/` crate created (transitional `#[path]` wrapper) |
| **C12 (testgen → core/)** | **Done** | `core/codegen/src/testgen/` with 9 files |
| C13 (Split mock_defaults) | Not done | Generic probing not confirmed moved to `core/test/` |
| C14 (REST status-code checking) | Not done | `GenericRestParseOp` still doesn't check status before extraction |
| **C15 (Fail-closed resolver)** | **Done** | `passthrough_fallback_value` deleted |
| C16 (Transport class metadata) | Partial | `ServiceTransportClass` enum exists, `ServiceCallMetadata` has transport field, but registry gen may still use substrings |
| **C17 (Kill propagate_to_param_sources)** | **Done** | Function deleted |
| C18 (Executor dead code) | Not done | `looks_effectful_without_kind()` still in `core/exec/src/execute/mod.rs` |
| C19 (Restore passthrough enforcement) | Not done | No `ExecError` for missing required outputs |
| C20 (CLI gen: profile/mode/subcommand) | Partial | `enable_mode` flag exists in `cli_gen.rs`, but no `--profile` enum, no subcommand dispatch |

### Assessment

Worker C has the lowest completion rate (35%) but was tasked with the hardest
work. The type system improvements (C2, C3, C7) are clean and impactful. The
code extraction tasks (C11, C12) established new crate boundaries. The dead
code deletion tasks (C15, C17) were effective.

However, the core refactoring vision — `LoweringContext` (C4), transport
derivation extraction (C6), stdlib caching (C1) — remains undelivered. These
are the tasks that would enable the "Google-style layer cake" architecture
described in tasks.md.

**C20 is the critical-path blocker**: Worker A's A1-A5 (binary elimination)
are gated on C20. C20 is only partially done (mode flag exists, but profile
enum and subcommand dispatch do not). This blocks 5 Worker A tasks representing
~2,657 lines of deletion.

### Risks

- C20 partial completion blocks all of Worker A's binary elimination.
- C11 uses `#[path]` transitional wrapper — not a true crate extraction yet.
- C10 (ReturnExprCompute) not done — this is the root cause of the `gunbc-ci`
  false failure documented in the postmortem. `BinOp`, `If`, `Match` return
  expressions still silently dropped by lowerer.
- scope.rs (C5) infrastructure exists but isn't wired — dead code risk.

---

## Cross-Branch Compatibility

### Build Failure on Merge

Merging all three branches into one produces a build failure:

```
✗ expr_compute_tools_codegen_codegen_success_return_0 [UNKNOWN]: unbound variable: check
```

This indicates the lowerer changes from Worker C (`ExprCompute` replacement for
`ReturnExprCompute`) interact poorly with the return expression wiring that
Worker B's extern call lowering expects. The `check` variable reference in
`tools/codegen.dag`'s return expression isn't being resolved in the scoped
analysis after the C3/C7 changes.

### File Ownership Violations

Per the file ownership table in tasks.md:

| Files | Owner | Violations |
|-------|-------|------------|
| `gunbc-app/src/bin/ci.rs` | Shared read-only | Worker A modified (new CLI parsing), Worker B also modified (import changes) — merge conflict |
| `gunbc-app/src/lib.rs` | Shared read-only | Worker B added `resolve_service` module — merge conflict |
| `gunbc-app/Cargo.toml` | Shared | Both Worker A and C added crate deps — merge conflict |
| `gunbc-app/tests/workflow_executor_contracts.rs` | Worker A | Worker A deleted, Worker B modified — delete/modify conflict |
| `tasks.md` | All | All three branches modified — auto-merged |

3 of 4 conflicts were in shared/read-only territory, indicating the ownership
model mostly worked. The `workflow_executor_contracts.rs` conflict was a true
cross-worker conflict (A deleted what B modified).

---

## Summary of Gaps and Recommendations

### Critical Path Issues

1. **C20 blocks A1-A5**: Worker C must complete profile/mode/subcommand CLI
   generation before Worker A can eliminate 5 handwritten binaries (~2,657 LOC).
   This is the single largest blocked work item.

2. **ED lane (21 tasks) is completely unexecuted**: No worker was assigned.
   This blocks all of Phase 2 Lane 6 (Service Layer Completion) and transitively
   Lane 7 (SDLC Production).

3. **Build failure on merge**: The three branches don't compose cleanly. The
   `ExprCompute` / scoped analysis changes from Worker C conflict with the
   return expression patterns in codegen.dag.

### Work Quality Assessment

- **Best execution**: Worker B (80% done, net -2,920 LOC, clean deletion chain)
- **Best strategy**: Worker A (correctly deferred blocked work, completed non-blocked chain)
- **Most impactful type work**: Worker C (PipeMethod, typed enums, LeafRef)
- **Most complete**: Lane 4 (22/22, pure DSL authoring)
- **Lowest velocity**: Worker C (35%, but hardest scope)

### Recommended Next Steps

1. **Fix build failure**: Investigate `unbound variable: check` in `ExprCompute`
   node — likely scoped analysis regression from C3/C7.
2. **Complete C20**: Unblock Worker A's binary elimination chain.
3. **Start ED lane**: Assign a worker to ED-1:8 (cloud/core, github, llm/core).
   These are S/M tasks with zero dependencies.
4. **Complete B2/B10**: Migrate remaining `BuildConfig`/`ToolInfo` to DSL.
5. **Wire C5**: `scope.rs` exists but isn't integrated — either integrate or delete.
6. **Fix BT-E1**: Transport node deduplication blocks full SDLC DryRun.
