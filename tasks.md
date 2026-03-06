# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)
**Archive**: `TODO/TODONE/tasks-archive-2026-03-02.md` (40 completed items from earlier lanes)

---

## Four Lanes

| Lane | Doc | Goal | Open Items |
|------|-----|------|------------|
| **1. Type System** | [`TODO/type-system.md`](TODO/type-system.md) | Compositional type coverage — decisions obligate, obligations propagate. WS-1 through WS-7. | 29 open + 11 done across 7 workstreams |
| **2. Compiler Debt & App Layer** | [`TODO/gunbc-dag-simplification.md`](TODO/gunbc-dag-simplification.md) | Fix compiler gaps that force runtime bridges. 10 accidental bridges → delete. Each has specific files/LOC to remove. | 10 bridges + app layer cleanup |
| **3. SDLC Pipeline** | [`TODO/sdlc.md`](TODO/sdlc.md) | Run the SDLC pipeline end-to-end. Phase 0 (prove compilation) is a **hard gate**. | 10 done + 9 in progress across 5 phases |
| **4. Compiler Pipeline** | [`TODO/compiler-pipeline.md`](TODO/compiler-pipeline.md) | End-to-end pipeline hardening + interpreted/compiled parity. Three invariants: binary logic, minimalism, resolve early. | 42 items across 9 workstreams |

### Cross-Cutting Reliability Lane

Source of truth: [`TODO/rolling-postmortem.md`](TODO/rolling-postmortem.md)

1. **RR-1 (P0)**: Replace heuristic test-time confidence with measured runtime budget checks for `test-xs/s/m/l/xl` (maps to RC-P0-004).
2. **RR-2 (P1)**: Split monolithic exhaustive tests into bounded shards or explicit integration-only flows; default loops should stay interactive (maps to RC-P1-005/006).

### Cross-Cutting Auth Architecture

Source of truth: [`TODO/rolling-postmortem.md`](TODO/rolling-postmortem.md)

1. **AUTH-1 (P0)**: Define the final structural auth model. Services declare auth requirements semantically in their own models; workflows/tools never acquire tokens or call credential helpers directly.
2. **AUTH-2 (P1) — DONE (2026-03-05)**: Added provider auth models under `dsl/extdeps/<provider>/auth.dag` for GitHub and LLM providers. `dsl/tools/gist.dag`, `dsl/shared/gist_modes.dag`, and `dsl/funcs/sdlc_worker.dag` now consume `extdeps.github.auth::github_token()` / `extdeps.llm.auth::llm_api_key()` instead of the deleted `dsl/shared/credentials.dag` helper.
3. **AUTH-3 (P1)**: Finish lowerer/runtime auth injection so `AuthContext`/provider realization is real-mode safe for authenticated services, then delete the interim workflow-local auth materialization in `dsl/extdeps/github/auth.dag` and `dsl/extdeps/llm/auth.dag`.
4. **AUTH-4 (P1)**: Delete the temporary `dsl/profiles/sdlc.dag` compatibility path once compiler-side concrete binding/link cleanup lands. Acceptance: local SDLC real-mode proof still works without `profiles.sdlc.local`, and `rg -n 'profiles\\.sdlc\\.local|module profiles\\.sdlc' dsl gunbc-dag docs -g'*.dag' -g'*.rs' -g'*.md'` only finds historical notes.

### Cross-Cutting `.dag` Migration

Source of truth: [`TODO/gunbc-dag-simplification.md`](TODO/gunbc-dag-simplification.md)

1. **DM-1 (P0) — DONE (2026-03-05)**: Deleted the remaining dead handwritten cloud/provider crates that now have `.dag` replacements: `lib/gcp-ops` and `lib/aws-ops`, following the earlier removal of `lib/gcp-ops/src/ops.rs`, `lib/gcp-ops/src/services/local_auth.rs`, and `lib/cloud-ops/src/infra_*`. Workspace/config/guardrail references were updated in `Cargo.toml`, `dsl/config/workspace.dag`, `dsl/config/arch_rules.dag`, `dsl/extdeps/gunbc.dag`, `gunbc-dag/tests/boundary_gate.rs`, and `lib/transport/src/pragma_lint.rs`. Update later on 2026-03-05: the last scheduled handwritten survivor in this lane, `gunbc-dag/src/testgen_dag/graph.rs`, was deleted and replaced by [`dsl/tools/testgen.dag`](dsl/tools/testgen.dag), with Rust reduced to narrow discovery/render extern bridges. Follow-on cleanup the same day deleted the temporary thin shim layer in `gunbc-dag`, so this lane now ends with compiler/framework internals plus narrow app extern bindings, not handwritten provider/workflow graphs. Rule remains: no new provider/runtime logic lands in Rust unless the compiler cannot yet express it.
2. **DM-2 (P0) — DONE (2026-03-05)**: Deleted the remaining thin app-layer shim surfaces in `gunbc-dag`: `src/tool_graphs.rs`, `src/pragma/mod.rs`, `src/docgen/mod.rs`, `src/dsl_builder.rs`, `src/fs_env.rs`, `src/dry_run.rs`, `src/dsl_registry.rs`, and `src/resolve.rs`. Generated callers, tests, and tool discovery now use direct `gunbc_resolve::builder::build_dsl_graph(...)` / `gunbc_resolve::resolve_lowered_dag_with(...)` calls with the real app binding point, [`gunbc-dag/src/extern_ops.rs`](gunbc-dag/src/extern_ops.rs) `GunbcExternResolver`. Supporting logic that still belongs in app code was narrowed to [`gunbc-dag/src/makegen_support.rs`](gunbc-dag/src/makegen_support.rs) and [`gunbc-dag/src/resource_targets.rs`](gunbc-dag/src/resource_targets.rs). Follow-on cleanup the same day also deleted the dead handwritten Rust Justfile renderer in `core/codegen/src/makegen/justfile.rs` and stripped stale makegen tool-projection fields (`live_secrets`, local-profile injection, unused build toggles) that only existed for the old registry/profile path. Acceptance: `rg -n 'pub fn build_.*graph|pub fn .*signature' gunbc-dag/src` returns no results, and source/generated Rust no longer imports repo-local wrapper modules for DSL graph building or resolution.
3. **DM-3 (P0) — PARTIAL (2026-03-05)**: Repo-facing profile/auth cleanup is largely landed. Deleted `available_profiles` plumbing from tool discovery, CLI generation, and makegen projections; removed generated/user `--profile` handling; added provider auth modules under `dsl/extdeps/{github,llm}/auth.dag`; deleted `dsl/shared/credentials.dag` and `dsl/profiles/gist.dag`; and rewrote active runtime diagnostics to talk about missing concrete bindings instead of `--profile`. A temporary `dsl/profiles/sdlc.dag` compatibility path has been reintroduced only to unblock local SDLC real-mode proof while compiler cleanup happens elsewhere. Remaining residue is compiler-internal plus that temporary SDLC compatibility module: lowerer profile types, interface-stub compatibility fixtures under `dsl/profiles/`, and historical design docs.
4. **DM-4 (P0) — Large Sweep: Runtime Bridge Deletion**: Delete the remaining evaluator-era runtime bridges in one pass. Scope: Bridges 1, 2, 3, 8, and 9 from `TODO/gunbc-dag-simplification.md` — `DeclaredOutputCallableOp`, `FnBodyCallableOp`, `CollectionDelegate`, resolver-time FS injection, and generic file adapters. Final state: lowerer emits SubDags/pattern IR directly; resolver only resolves externs and executes typed IR. Acceptance: `rg -n 'DeclaredOutputCallableOp|FnBodyCallableOp|CollectionDelegate|add_fs_env_root_node|GenericFilePrepareOp|GenericFileParseOp' core/` returns 0.
5. **DM-5 (P1) — PARTIAL (2026-03-05)**: The repo-owned makegen lane moved out of `core/codegen` into `gunbc-dag/src/makegen/`, and the handwritten Rust Justfile renderer plus profile-specific tool-projection hacks were deleted. Remaining work is the real end-state cutover: remove runtime `DiscoverToolsOp`/`render_makefile_from_dsl_discovery`, and stop loading build-target/gitignore DSL data from Rust at execution time.
6. **DM-6 (P1) — Large Sweep: Extern and Artifact Collapse**: Shrink `gunbc-dag/src/extern_ops.rs` to the irreducible minimum by deleting app externs that exist only because the compiler cannot yet emit artifacts or express a pattern. Scope: `DiscoverToolsOp`, bootstrap render externs that should become generated artifacts, and the remaining repo-specific render/discovery helpers as DSL features land (`render_tree`, `build_snapshot_content`, CI config discovery, infra dispatch). Acceptance: `rg -n 'DiscoverToolsOp|render_tree|build_snapshot_content|DiscoverCiConfigOp|InfraDispatchOp' gunbc-dag/src core/codegen -g'*.rs'` trends to 0, with any remaining hits explicitly justified in `TODO/gunbc-dag-simplification.md`.
7. **DM-7 (P1) — PARTIAL (2026-03-05)**: Removed the hardcoded `CODEGEN_*` constants from `core/ir` and the app re-export from `gunbc-dag`, so output layout is no longer duplicated there. Remaining duplication lives in `core/ir/src/workspace_layout.rs`, generated-bin paths in `gunbc-dag/Cargo.toml`, and fallback/default path strings like `gunbc-dag/src/bin/codegen_cli.rs`.
8. **DM-3A (P1) — Deferred To Compiler-Cleanup Branch**: Remove the remaining compiler-internal profile machinery or rename it to the final concrete-binding model. Scope: lowerer profile enums/errors/options in `core/daglang/daglang-lower`, runtime stub diagnostics in `core/resolve`, and compatibility fixtures under `dsl/profiles/` that only exist to test the old path. Acceptance: `rg -n 'UnknownProfile|AmbiguousProfile|InvalidProfileBinding|MissingProfileBinding|profile: Option<&str>|dsl/profiles/' core dsl -g'*.rs' -g'*.dag'` only finds explicitly retained compatibility fixtures or historical docs.
9. **DM-5A (P1) — Cleanup Follow-Through**: Finish the non-compiler remainder of makegen extraction in this repo/app layer. Scope: delete `DiscoverToolsOp`, delete `render_makefile_from_dsl_discovery`, and replace Rust-side `compile_data_from_module(...build_targets.dag|gitignore.dag)` loaders with generated/static artifacts or direct DSL-owned data inputs. Acceptance: `rg -n 'DiscoverToolsOp|render_makefile_from_dsl_discovery|compile_data_from_module\\(&dsl_root, \"config/build_targets.dag\"|compile_data_from_module\\(&dsl_root, \"config/gitignore.dag\"' gunbc-dag core/codegen -g'*.rs'` returns 0.
10. **DM-7A (P1) — Cleanup Follow-Through**: Finish output/layout dedup in the app layer. Scope: move remaining `target/codegen/bin` truth out of `core/ir/src/workspace_layout.rs`, `gunbc-dag/Cargo.toml`, and `gunbc-dag/src/bin/codegen_cli.rs` so `dsl/config/codegen_paths.dag` is the only authority. Acceptance: `rg -n 'target/codegen/bin|target/codegen/lib|target/codegen/.codegen-stamp' core gunbc-dag Cargo.toml -g'*.rs' -g'Cargo.toml'` only finds generated output or the final single-source loader.

### Design docs

| Doc | Scope |
|-----|-------|
| [`docs/design/compilation-pipeline.md`](docs/design/compilation-pipeline.md) | Full pipeline map (.dag → execution), data shapes at each stage, gap analysis |
| [`docs/design/v4/compiler-densification-roadmap.md`](docs/design/v4/compiler-densification-roadmap.md) | Prioritized roadmap: kill interpreter, hermeticity, dual-encoding, service codegen |
| [`docs/design/v4/compositional-type-coverage.md`](docs/design/v4/compositional-type-coverage.md) | Type system vision, audit, gaps, workstreams, worked examples |
| [`docs/design/sdlc/domain-modeling-comprehensive.md`](docs/design/sdlc/domain-modeling-comprehensive.md) | SDLC entity/relationship/state machine model |
| [`docs/design/sdlc/production-gap-analysis.md`](docs/design/sdlc/production-gap-analysis.md) | SDLC activation blockers |

### Dependency between lanes

```
Lane 1 (type system)    ──→  Lane 3 (SDLC) uses the type system
Lane 2 (compiler debt)  ──→  Lane 3 (SDLC) needs working compilation pipeline
Lane 4 (pipeline)       ──→  Lane 2 (bridges) benefits from pipeline hardening
Lane 4 (pipeline)       ──→  Lane 3 (SDLC) needs reliable compilation + emit
```

Lane 1 and Lane 2 can proceed in parallel. Lane 3 Phase 0 (prove it compiles) can start now — it doesn't need type system improvements, just basic compiler correctness. Lane 4 is independent groundwork — hardening the pipeline benefits all other lanes.

**Recommended start order**:
1. Lane 3 Phase 0 (S-1 through S-4) — fix known bugs, prove SDLC compilation. **Hard gate.**
2. Lane 2 "Delete immediately" (bridges 4, 5, 10) — trivial cleanup, ~300 LOC deleted
3. Lane 2 "Compiler fixes" (bridges 1-3, 8-9) — each has specific deletion targets + grep verification
4. Lane 1 WS-1 + WS-3 (no blockers) — in parallel with Lane 2

**Operating principles** (from retrospective):
- Prove before building on top. No Phase N+1 work until Phase N is green.
- Each task names what gets **deleted** and a `grep` command to verify deletion.
- No intermediate abstractions. Go a→f directly.
- `@annotation` is never the final state — go straight to structural blocks.
- Check `Cargo.toml` dependency graphs before moving code between crates.

---

## Completed (archived)

40/40 items complete across earlier lanes:

- **Lane 1: Compiler Pipeline** — 26/26 (C1-C30)
- **Lane 1: Binary Elimination** — 10/10 (A2-A11)
- **Phase 3: Purist Engine** — 4/4 (C28-CT8)
