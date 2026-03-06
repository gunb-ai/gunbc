# Binary Elimination: Handwritten → Generated

**Goal**: Eliminate all handwritten binary entrypoints in `gunbc-dag/src/bin/` except the two bootstrap exceptions (`ci.rs`, `codegen_cli.rs`). The CLI generator (`cli_gen.rs`) should produce every binary from DSL metadata.

**Guiding principle**: The DSL `.dag` file is the specification. The binary is derived code. If a binary needs hand-written Rust, that's a compiler feature gap.

## Current State

| Binary | Lines | Status | Verdict |
|--------|-------|--------|---------|
| `codegen_cli.rs` | 1,215 | IS the generator | **Keep** (bootstrap) |
| `ci.rs` | 210 | Runs before codegen | **Keep** (bootstrap) |
| `infra.rs` | 1,055 | 8 subcommands, custom arg parsing, type coercion | **Eliminate** (needs compiler features) |
| `workflow.rs` | 715 | Planner/orchestrator, plan/run modes, JSON/text rendering | **Eliminate** (needs compiler features) |
| `pipeline.rs` | 341 | Profile support, pre-execution GitHub API, depth validation | **Eliminate** (needs template extension) |
| `sdlc.rs` | 266 | Profile selection, param_source wiring, conditional mock spec | **Eliminate** (needs template extension) |
| `deps_config.rs` | 209 | Resource manifest update, verify/ensure modes | **Eliminate** (needs resource mode support) |

**Target**: Delete 5 binaries (2,586 lines) by extending the CLI generator.

## What the Generated Template Already Handles

The `cli_gen.rs` template (producing files like `target/codegen/bin/gist/main.rs`) already provides:

- CLI flag generation from `func` parameter signatures (type-aware, short flags, defaults)
- Argument parsing via `gunbc_cli::CliParam` schema
- `--dry-run` / `-n` flag with `auto_mock_spec` integration
- `--print-inputs json` for debugging
- `--help` generation from parameter metadata
- Entrypoint detection + boundary input wiring
- Freshness composition (`check_and_plan_freshness()`)
- Animated progress display with terminal detection
- Step mode for CI (subcommands: run/step/list-steps)

This covers ~80% of a typical tool binary. The remaining 20% is what the handwritten binaries add.

## Gap Analysis: What Handwritten Binaries Do That Generated Can't

### Gap 1: Profile Support (sdlc.rs, pipeline.rs)

**What**: `sdlc.rs` accepts `--profile unit_test|local|cloud_run`, validates the enum, and threads it into `gunbc_resolve::BuildOpts.profile` when calling `build_dsl_graph(...)`. Profile determines which interface bindings are active.

**DSL source**: The `.dag` file already declares profiles:
```
profile unit_test { bind IssueProvider { impl: StubIssueProvider, ... } }
profile local { bind IssueProvider { impl: GitHubIssueProvider, ... } }
```

**Solution**: The compiler already knows the profile names from `CompileOutput`. The CLI generator should:
1. Detect when a `.dag` module has `profile` declarations
2. Auto-generate a `--profile` CLI param with enum validation
3. Call `gunbc_resolve::builder::build_dsl_graph(path, &GunbcExternResolver, BuildOpts { entry_func: Some(func), profile: Some(&profile) })`
4. When `profile == "unit_test"` or `--dry-run`, use `auto_mock_spec` for DryRun mode

**Compiler change**: `CompileOutput` should expose `available_profiles: Vec<String>`. The CLI template checks `!profiles.is_empty()` and generates profile dispatch.

### Gap 2: Pre-Execution Side Effects (pipeline.rs)

**What**: `pipeline.rs` calls `gh run list`, `gh pr view`, `gh issue view` BEFORE building the DAG. These gather context (CI status, PR description, issue body) and wire them as entrypoint inputs.

**Why this is wrong**: These are effectful I/O operations happening outside the DAG. They should BE DAG nodes — the `func` should declare `context: String?` as an input and a separate `func gather_context(pr: String?) -> { context: String }` should call `gh` operations via shell transport.

**Solution**: Model context gathering as a DAG func in the `.dag` file. The pipeline binary just needs standard CLI params (`--pr`, `--issue`, `--depth`). No special pre-execution hooks needed if the DAG handles context gathering internally.

**Compiler change**: None — this is a DSL authoring fix, not a compiler feature.

### Gap 3: Resource Mode (deps_config.rs, ci.rs)

**What**: `deps_config.rs` accepts `--mode=verify|ensure`. In `verify` mode, it compares expected vs. actual and exits 1 on drift. In `ensure` mode, it writes the file. After writing, it updates the resource manifest.

**Current template support**: The generated template already supports `enable_step_mode` for CI. But the verify/ensure pattern is different — it's about resource management, not step-based execution.

**Solution**: This pattern is a `content_upsert` workflow — the DSL already models it! `deps_config.rs` should compile from `dsl/tools/deps.dag` (which already exists and generates `deps.toml`). The verify/ensure modes map directly to the content_upsert compare+write pattern. The `--mode=verify` behavior is just "run the compare node and exit 1 if different."

**Compiler change**: Add `resource_mode: Option<Vec<String>>` to `ToolMeta`. When set, generate `--mode=verify|ensure` flag. In verify mode, skip the write node. In ensure mode, run the full DAG. The resource manifest update should be a post-execution hook in the DAG, not manual Rust code.

### Gap 4: Subcommand Dispatch (infra.rs, workflow.rs)

**What**: `infra.rs` has 8 subcommands (`spec`, `graph`, `plan`, `apply`, `reconcile`, `bootstrap`, `login`, `status`). `workflow.rs` has 2 modes (`--plan`, positional run).

**Current template support**: None. The template assumes a single entrypoint func.

**Solution**: Multiple `func` items in a `.dag` module can each become a subcommand. The compiler already discovers multiple entrypoints per module via `infer_entrypoints()`. Instead of generating one binary per func, generate one binary with subcommand dispatch when a module has multiple user-facing funcs.

**Compiler change**: When `inferred_entrypoints.len() > 1` for a module, generate a dispatcher binary:
```rust
match subcommand {
    "plan" => { /* build + execute plan func */ }
    "apply" => { /* build + execute apply func */ }
    _ => print_help(),
}
```

Each subcommand gets its own CLI schema from the corresponding func's parameters. This is structurally similar to `enable_step_mode` but generalized.

### Gap 5: Domain Argument Transformation (pipeline.rs, sdlc.rs)

**What**: `sdlc.rs` splits `--repo gunb-ai/gunbc` into `(owner, repo_name)`. `pipeline.rs` validates that `--depth` is a valid FermiDepth enum value.

**Current template support**: 1:1 mapping from CLI param to port value. No validation beyond type coercion.

**Solution**: These are type-level concerns. If the DSL declares `owner: String` and `repo: String` as separate params, no splitting needed. If it declares `depth: FermiDepth`, the type system provides the enum variants for validation.

**Compiler change**: When a func param has a sum type (enum), generate CLI validation from the variant names:
```rust
let depth = match depth_str.as_str() {
    "XS" | "xs" => Value::Str("XS".into()),
    "S" | "s" => Value::Str("S".into()),
    // ...
    other => { eprintln!("invalid depth: {other}"); process::exit(1); }
};
```

For compound params like `repo`, prefer decomposing into separate params at the DSL level (owner + repo_name).

### Gap 6: Param Source Safety Net (sdlc.rs)

**What**: `sdlc.rs` manually propagates entrypoint values to `param_source_*` interior nodes — a workaround for the lowerer not wiring callable parameters to param_source nodes.

**Solution**: Fix the lowerer (Gap 3 in `detect_entrypoints`), not the binary. The param_source propagation should happen inside `BoundaryMocks::set_input()` or `detect_entrypoints()`, not in each binary.

**Compiler change**: Move param_source propagation into `detect_entrypoints()` — it already finds param_source nodes. When setting an input on a boundary node, also check for param_source nodes with the same port name and propagate.

### Gap 7: Custom Output Rendering (workflow.rs, infra.rs)

**What**: `workflow.rs` renders plan output in text or JSON format. `infra.rs` renders spec as JSON or DOT graph.

**Current template support**: The template uses `execute_and_display()` which renders a progress spinner and final output. No structured output rendering.

**Solution**: Output rendering should be DSL-driven. `workflow.rs` manually formats `PlanExplain` into text/JSON — this formatting logic should be a DSL fn. The binary just needs a `--format text|json` flag and passes the format choice as a DAG input.

**Compiler change**: Add `output_format: Option<Vec<String>>` to `ToolMeta`. When set, generate `--format` flag and pass the value as a DAG entrypoint input. The DAG's func handles rendering in the requested format.

### Gap 8: Complex CLI Parsing (infra.rs)

**What**: `infra.rs` supports `KEY=VALUE` pair parsing for `--input project_id=my-project`, `--target` and `--skip` multi-value flags, and `--execute` as a safety gate.

**Current template support**: Basic flag parsing. No `KEY=VALUE`, no multi-value accumulation.

**Solution**: Multi-value flags map to `List<String>` params. `KEY=VALUE` maps to `Map<String, String>` params. The CLI parser already supports these types via `ParamType`.

**Compiler change**: When a func param has type `List<String>`, generate an accumulator flag (`--target A --target B`). When a func param has type `Map<String, String>`, generate a `KEY=VALUE` parser. These are generic CLI conventions, not infra-specific.

## Implementation Plan

### Phase 1: Infrastructure (enables all subsequent phases)

**RT58: Param source propagation in detect_entrypoints** (S)
Move the `param_source_*` propagation from `sdlc.rs` into `detect_entrypoints()` or `BoundaryMocks` so all generated binaries get it automatically.

**RT59: Profile-aware CLI generation** (M)
- Expose `available_profiles` in `CompileOutput`
- CLI template: when profiles exist, generate `--profile` enum flag
- Use `gunbc_resolve::builder::build_dsl_graph(..., BuildOpts { entry_func, profile })`
- Profile `unit_test` auto-enables DryRun mode

### Phase 2: Simple Binary Elimination

**RT60: Eliminate sdlc.rs** (S, depends: RT58, RT59)
With profile-aware CLI gen and param_source fix, `sdlc.rs` becomes a standard generated binary. The `.dag` file already declares everything needed. Delete `sdlc.rs`, verify generated binary matches behavior.

**RT61: Eliminate deps_config.rs** (S)
Model verify/ensure as a DAG execution parameter. `deps.dag` already handles content generation. Add `--mode` flag support to template for content_upsert workflows.

**RT62: Eliminate pipeline.rs** (M, depends: RT59)
- Move `query_ci_status()`, `query_pr_description()`, `query_issue_description()` into DAG func nodes (shell transport to `gh` CLI)
- With profile support + standard CLI params, the binary is generated

### Phase 3: Complex Binary Elimination

**RT63: Subcommand dispatch in CLI generator** (M)
When a `.dag` module has multiple exported `func` items, generate one binary with subcommand dispatch instead of N separate binaries. Each subcommand gets its own parameter schema.

**RT64: Eliminate workflow.rs** (L, depends: RT63)
- Requires subcommand dispatch (plan vs run modes)
- Move `render_plan_text`, `render_plan_json` into DSL fns
- Move SLO checking into the DAG

**RT65: Eliminate infra.rs** (L, depends: RT63)
- 8 subcommands (most complex binary)
- Move `InfraSpec` rendering to DSL
- Needs `KEY=VALUE` parsing + multi-value flags + `--execute` safety gate in CLI template
- This is the last and hardest elimination

### Phase 4: Cleanup

**RT66: Delete handwritten binary test infrastructure** (S, depends: RT64, RT65)
After all binaries are generated, delete the `BinaryArgs` type from `gunbc-cli` (replaced by `CliParam` schema), remove `#[allow(clippy::disallowed_methods)]` annotations, and clean up any orphaned support code.

## Dependency Graph

```
RT58 (param_source) ─────────┐
                              ├─→ RT60 (sdlc.rs)
RT59 (profile CLI gen) ──────┤
                              ├─→ RT62 (pipeline.rs)
                              │
RT61 (deps_config.rs) ────────│   (independent)
                              │
RT63 (subcommand dispatch) ──┼─→ RT64 (workflow.rs)
                              └─→ RT65 (infra.rs)
                                    │
                                    └─→ RT66 (cleanup)
```

## Net Impact

| Metric | Before | After |
|--------|--------|-------|
| Handwritten binary files | 7 | 2 (bootstrap only) |
| Handwritten binary LOC | 4,009 | 1,425 (ci.rs + codegen_cli.rs) |
| Lines deleted | — | ~2,584 |
| New compiler features | — | 3 (profile CLI, subcommand dispatch, resource mode) |
| DSL authoring changes | — | 2 (pipeline context → DAG nodes, workflow render → DSL fns) |

## Verification

For each eliminated binary, the acceptance test is:
1. Generate the binary from the `.dag` file
2. Run with `--dry-run` — same output structure as the handwritten version
3. Run with `--help` — documents same parameters
4. If applicable, `--print-inputs json` matches
5. Delete the handwritten file; `cargo test --workspace` passes
