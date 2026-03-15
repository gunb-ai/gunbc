# Repo Workboard

This is the repo-root control plane for the OpenClaw worktree workflow.

The intended loop is:

1. Refresh the root workboard.
2. Refresh the isolated automation worktree from recent `main` when safe.
3. Take one manual task if any are open.
4. Otherwise fall back to one scouting pass against `INVARIANTS.md`.
5. Commit automation changes in the isolated worktree for later review.

## Inputs

- Rubric: `INVARIANTS.md`
- Manual runner: `python3 scripts/openclaw/run_worktree_cycle.py`
- Workboard refresh: `python3 scripts/openclaw/sync_workboard.py`
- Existing v2 planning board: `src/v2/WORKBOARD.md`

## Manual Task Queue

Add explicit tasks here. Put a file path in backticks when you want the runner
to scope Codex to a single file first.

<!-- openclaw:manual:start -->
<!-- Add unchecked checkbox items here. Put the target path in backticks when you want file-scoped work. -->
<!-- openclaw:manual:end -->

## Managed Summary

<!-- openclaw:summary:start -->
- Manual tasks open: 0
- Scout files remaining: 306
- Last event: 2026-03-14T00:00:00-04:00 initialized workboard scaffold
<!-- openclaw:summary:end -->

## Managed Scout Queue

When no manual task is open, the runner takes the next unchecked file from this
queue and asks Codex to scout it against `INVARIANTS.md`.

<!-- openclaw:scout:start -->
- [ ] `src/v1/00_foundation/daglang-contract/src/lib.rs`
- [ ] `src/v1/00_foundation/delegate-macros/src/lib.rs`
- [ ] `src/v1/00_foundation/delegate-macros/tests/delegation.rs`
- [ ] `src/v1/00_foundation/infra/src/dagbin_cache.rs`
- [ ] `src/v1/00_foundation/infra/src/freshness.rs`
- [ ] `src/v1/00_foundation/infra/src/hash.rs`
- [ ] `src/v1/00_foundation/infra/src/lib.rs`
- [ ] `src/v1/00_foundation/infra/src/manifest.rs`
- [ ] `src/v1/00_foundation/infra/src/workspace_model.rs`
- [ ] `src/v1/00_foundation/ir/src/algebra.rs`
- [ ] `src/v1/00_foundation/ir/src/boundary.rs`
- [ ] `src/v1/00_foundation/ir/src/builder.rs`
- [ ] `src/v1/00_foundation/ir/src/cargo.rs`
- [ ] `src/v1/00_foundation/ir/src/code_ir/c_ir.rs`
- [ ] `src/v1/00_foundation/ir/src/code_ir/lower.rs`
- [ ] `src/v1/00_foundation/ir/src/code_ir/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/code_ir/register_ir.rs`
- [ ] `src/v1/00_foundation/ir/src/codegen_bridge.rs`
- [ ] `src/v1/00_foundation/ir/src/coerce.rs`
- [ ] `src/v1/00_foundation/ir/src/contract.rs`
- [ ] `src/v1/00_foundation/ir/src/dag.rs`
- [ ] `src/v1/00_foundation/ir/src/dag_topology.rs`
- [ ] `src/v1/00_foundation/ir/src/entrypoint.rs`
- [ ] `src/v1/00_foundation/ir/src/filename.rs`
- [ ] `src/v1/00_foundation/ir/src/generated/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/git.rs`
- [ ] `src/v1/00_foundation/ir/src/invocation_contract.rs`
- [ ] `src/v1/00_foundation/ir/src/language/categories/config.rs`
- [ ] `src/v1/00_foundation/ir/src/language/categories/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/language/categories/turing.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/css.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/gitignore.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/html.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/makefile.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/markdown.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/rust.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/toml.rs`
- [ ] `src/v1/00_foundation/ir/src/language/languages/yaml.rs`
- [ ] `src/v1/00_foundation/ir/src/language/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/language/patterns/glob.rs`
- [ ] `src/v1/00_foundation/ir/src/language/patterns/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/language/patterns/regex.rs`
- [ ] `src/v1/00_foundation/ir/src/language/patterns/variable.rs`
- [ ] `src/v1/00_foundation/ir/src/language/traits/comment.rs`
- [ ] `src/v1/00_foundation/ir/src/language/traits/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/language/traits/naming.rs`
- [ ] `src/v1/00_foundation/ir/src/language/traits/type_system.rs`
- [ ] `src/v1/00_foundation/ir/src/layout.rs`
- [ ] `src/v1/00_foundation/ir/src/lib.rs`
- [ ] `src/v1/00_foundation/ir/src/log_detail.rs`
- [ ] `src/v1/00_foundation/ir/src/makefile_render.rs`
- [ ] `src/v1/00_foundation/ir/src/node.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/atomic.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/authenticate.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/branch.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/collection.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/content_upsert.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/emit.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/loop_pattern.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/pattern_op.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/repeat.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/transaction.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/transport_triplet.rs`
- [ ] `src/v1/00_foundation/ir/src/patterns/upsert.rs`
- [ ] `src/v1/00_foundation/ir/src/plain_render.rs`
- [ ] `src/v1/00_foundation/ir/src/platform.rs`
- [ ] `src/v1/00_foundation/ir/src/render_ir.rs`
- [ ] `src/v1/00_foundation/ir/src/resource/def.rs`
- [ ] `src/v1/00_foundation/ir/src/resource/defs.rs`
- [ ] `src/v1/00_foundation/ir/src/resource/handle.rs`
- [ ] `src/v1/00_foundation/ir/src/resource/managed.rs`
- [ ] `src/v1/00_foundation/ir/src/resource/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/resource/registry.rs`
- [ ] `src/v1/00_foundation/ir/src/resource/state.rs`
- [ ] `src/v1/00_foundation/ir/src/signature.rs`
- [ ] `src/v1/00_foundation/ir/src/symbol.rs`
- [ ] `src/v1/00_foundation/ir/src/symbols.rs`
- [ ] `src/v1/00_foundation/ir/src/system_model.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/agent.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/agent_adapter.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/behavior.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/command.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/provider.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/providers/github.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/providers/gitlab.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/providers/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/providers/plain.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/ci/runner.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/cli.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/cloud.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/credential.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/credential_policy.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/file.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/gcp.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/git.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/github/api.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/github/cli.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/github/issues.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/github/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/github/pull_request.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/github_actions.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/http.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/llm/anthropic.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/llm/chat.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/llm/mock.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/llm/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/llm/openai.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/llm/openai_responses.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/llm/provider.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/middleware.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/mod.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/rest.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/review.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/scope.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/tcp.rs`
- [ ] `src/v1/00_foundation/ir/src/transport/tool.rs`
- [ ] `src/v1/00_foundation/ir/src/type_lib.rs`
- [ ] `src/v1/00_foundation/ir/src/type_op.rs`
- [ ] `src/v1/00_foundation/ir/src/type_registry.rs`
- [ ] `src/v1/00_foundation/ir/src/type_shape.rs`
- [ ] `src/v1/00_foundation/ir/src/typed_io.rs`
- [ ] `src/v1/00_foundation/ir/src/types.rs`
- [ ] `src/v1/00_foundation/ir/src/validate.rs`
- [ ] `src/v1/00_foundation/ir/src/value.rs`
- [ ] `src/v1/00_foundation/ir/src/value_bridge.rs`
- [ ] `src/v1/00_foundation/ir/src/value_expr.rs`
- [ ] `src/v1/00_foundation/ir/src/verified.rs`
- [ ] `src/v1/00_foundation/ir/src/workspace_layout.rs`
- [ ] `src/v1/00_foundation/ir/tests/generated_staleness.rs`
- [ ] `src/v1/01_surfaces/cli/src/binary_args.rs`
- [ ] `src/v1/01_surfaces/cli/src/lib.rs`
- [ ] `src/v1/01_surfaces/codegen/src/bin/codegen_cli.rs`
- [ ] `src/v1/01_surfaces/codegen/src/bin/testgen_cli.rs`
- [ ] `src/v1/01_surfaces/codegen/src/cli_gen.rs`
- [ ] `src/v1/01_surfaces/codegen/src/entrypoint.rs`
- [ ] `src/v1/01_surfaces/codegen/src/fidelity.rs`
- [ ] `src/v1/01_surfaces/codegen/src/file_writer.rs`
- [ ] `src/v1/01_surfaces/codegen/src/lambda_gen.rs`
- [ ] `src/v1/01_surfaces/codegen/src/lib.rs`
- [ ] `src/v1/01_surfaces/codegen/src/registry.rs`
- [ ] `src/v1/01_surfaces/codegen/src/rest_gen.rs`
- [ ] `src/v1/01_surfaces/codegen/src/template.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/analyze.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/cardinality.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/codegen.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/mock_corpus.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/mod.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/obligation.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/probe_observer.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/registry_gen.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen/render_rust.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen_dag/dag_test_discovery.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen_dag/mock_interpreter.rs`
- [ ] `src/v1/01_surfaces/codegen/src/testgen_dag/mod.rs`
- [ ] `src/v1/01_surfaces/codegen/src/tool_discovery.rs`
- [ ] `src/v1/01_surfaces/codegen/tests/backend_compile_smoke.rs`
- [ ] `src/v1/01_surfaces/codegen/tests/emitted_binary_smoke.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/commands.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/compile.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/compile/context.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/compile/render.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/compile/tests.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/compile/triplets.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/lib.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/main.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/path_utils.rs`
- [ ] `src/v1/01_surfaces/daglang-cli/src/pipeline.rs`
- [ ] `src/v1/01_surfaces/workflow/src/admission.rs`
- [ ] `src/v1/01_surfaces/workflow/src/coordination.rs`
- [ ] `src/v1/01_surfaces/workflow/src/errors.rs`
- [ ] `src/v1/01_surfaces/workflow/src/executor.rs`
- [ ] `src/v1/01_surfaces/workflow/src/global_plan.rs`
- [ ] `src/v1/01_surfaces/workflow/src/key.rs`
- [ ] `src/v1/01_surfaces/workflow/src/lib.rs`
- [ ] `src/v1/01_surfaces/workflow/src/planner.rs`
- [ ] `src/v1/01_surfaces/workflow/src/process_registry.rs`
- [ ] `src/v1/01_surfaces/workflow/src/projection.rs`
- [ ] `src/v1/01_surfaces/workflow/src/proof.rs`
- [ ] `src/v1/01_surfaces/workflow/src/schema.rs`
- [ ] `src/v1/01_surfaces/workflow/src/slo.rs`
- [ ] `src/v1/02_pipeline/daglang-driver/src/lib.rs`
- [ ] `src/v1/02_pipeline/daglang-driver/src/pipeline.rs`
- [ ] `src/v1/02_pipeline/daglang-driver/src/prepare.rs`
- [ ] `src/v1/02_pipeline/daglang-driver/src/receipt.rs`
- [ ] `src/v1/03_source/daglang-resolve/src/lib.rs`
- [ ] `src/v1/03_source/daglang-resolve/tests/module_graph.rs`
- [ ] `src/v1/03_source/daglang-syntax/src/ast_utils.rs`
- [ ] `src/v1/03_source/daglang-syntax/src/callable.rs`
- [ ] `src/v1/03_source/daglang-syntax/src/diagnostic.rs`
- [ ] `src/v1/03_source/daglang-syntax/src/lexer.rs`
- [ ] `src/v1/03_source/daglang-syntax/src/lib.rs`
- [ ] `src/v1/03_source/daglang-syntax/src/parser.rs`
- [ ] `src/v1/04_semantics/daglang-typecheck/src/lib.rs`
- [ ] `src/v1/04_semantics/daglang-typecheck/src/tests.rs`
- [ ] `src/v1/05_graph/daglang-eval/src/eval.rs`
- [ ] `src/v1/05_graph/daglang-eval/src/eval_core.rs`
- [ ] `src/v1/05_graph/daglang-eval/src/eval_stack.rs`
- [ ] `src/v1/05_graph/daglang-eval/src/expr.rs`
- [ ] `src/v1/05_graph/daglang-eval/src/lib.rs`
- [ ] `src/v1/05_graph/daglang-eval/src/v2_tests.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/anf.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/eval.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/expr.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/lib.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/scope.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/spec.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/tests.rs`
- [ ] `src/v1/05_graph/daglang-lower/src/transport.rs`
- [ ] `src/v1/06_artifacts/daglang-derive/src/lib.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/backend_harness.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/computation.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/dag_emit.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/fn_codegen.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/language_model.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/lib.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/lower_c.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/lower_go.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/lower_mips.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/lower_rust.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/lower_to_ir.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/plan.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/render_c.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/render_go.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/render_mips.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/render_rust.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/rust_exec_runtime.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/service_emit.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/test_gen.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/test_mock_emit.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/transport_analysis.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/type_codegen.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/type_mapping.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/v2_crate_emit.rs`
- [ ] `src/v1/07_emit/daglang-emit/src/v2_runtime_shim.rs`
- [ ] `src/v1/08_materialize/blob/src/lib.rs`
- [ ] `src/v1/08_materialize/interp/src/lib.rs`
- [ ] `src/v1/08_materialize/resolve/src/builder.rs`
- [ ] `src/v1/08_materialize/resolve/src/dry_run.rs`
- [ ] `src/v1/08_materialize/resolve/src/fs_env.rs`
- [ ] `src/v1/08_materialize/resolve/src/lib.rs`
- [ ] `src/v1/08_materialize/resolve/src/resolve.rs`
- [ ] `src/v1/08_materialize/resolve/src/service_ops/mod.rs`
- [ ] `src/v1/08_materialize/resolve/src/service_ops/service_ops_impl.rs`
- [ ] `src/v1/08_materialize/transport/src/backend.rs`
- [ ] `src/v1/08_materialize/transport/src/classify.rs`
- [ ] `src/v1/08_materialize/transport/src/cli.rs`
- [ ] `src/v1/08_materialize/transport/src/credential.rs`
- [ ] `src/v1/08_materialize/transport/src/executor.rs`
- [ ] `src/v1/08_materialize/transport/src/freshness_policy.rs`
- [ ] `src/v1/08_materialize/transport/src/lib.rs`
- [ ] `src/v1/08_materialize/transport/src/metrics.rs`
- [ ] `src/v1/08_materialize/transport/src/middleware/mod.rs`
- [ ] `src/v1/08_materialize/transport/src/ops.rs`
- [ ] `src/v1/08_materialize/transport/src/pipeline.rs`
- [ ] `src/v1/08_materialize/transport/src/preflight.rs`
- [ ] `src/v1/08_materialize/transport/src/rate_limit.rs`
- [ ] `src/v1/08_materialize/transport/src/resource_io.rs`
- [ ] `src/v1/08_materialize/transport/src/retry.rs`
- [ ] `src/v1/08_materialize/transport/src/system_models.rs`
- [ ] `src/v1/08_materialize/transport/src/test_backend.rs`
- [ ] `src/v1/08_materialize/transport/src/transport_types.rs`
- [ ] `src/v1/08_materialize/transport/tests/basic_transports_integration.rs`
- [ ] `src/v1/09_execute/exec/src/box_draw.rs`
- [ ] `src/v1/09_execute/exec/src/ci_context.rs`
- [ ] `src/v1/09_execute/exec/src/diagnostic.rs`
- [ ] `src/v1/09_execute/exec/src/display.rs`
- [ ] `src/v1/09_execute/exec/src/env.rs`
- [ ] `src/v1/09_execute/exec/src/error.rs`
- [ ] `src/v1/09_execute/exec/src/execute/mod.rs`
- [ ] `src/v1/09_execute/exec/src/execute/tests.rs`
- [ ] `src/v1/09_execute/exec/src/frame_build.rs`
- [ ] `src/v1/09_execute/exec/src/frame_write.rs`
- [ ] `src/v1/09_execute/exec/src/freshness.rs`
- [ ] `src/v1/09_execute/exec/src/helpers.rs`
- [ ] `src/v1/09_execute/exec/src/intercept.rs`
- [ ] `src/v1/09_execute/exec/src/ledger.rs`
- [ ] `src/v1/09_execute/exec/src/lib.rs`
- [ ] `src/v1/09_execute/exec/src/lower.rs`
- [ ] `src/v1/09_execute/exec/src/pattern_op.rs`
- [ ] `src/v1/09_execute/exec/src/progress.rs`
- [ ] `src/v1/09_execute/exec/src/render.rs`
- [ ] `src/v1/09_execute/exec/src/terminal.rs`
- [ ] `src/v1/09_execute/exec/src/topo.rs`
- [ ] `src/v1/10_test/generated-tests/build.rs`
- [ ] `src/v1/10_test/generated-tests/src/lib.rs`
- [ ] `src/v1/10_test/test/src/auto_mock.rs`
- [ ] `src/v1/10_test/test/src/boundary.rs`
- [ ] `src/v1/10_test/test/src/composition.rs`
- [ ] `src/v1/10_test/test/src/corpus.rs`
- [ ] `src/v1/10_test/test/src/fermi.rs`
- [ ] `src/v1/10_test/test/src/fidelity.rs`
- [ ] `src/v1/10_test/test/src/json.rs`
- [ ] `src/v1/10_test/test/src/lib.rs`
- [ ] `src/v1/10_test/test/src/mock.rs`
- [ ] `src/v1/10_test/test/src/mock_requirements.rs`
- [ ] `src/v1/10_test/test/src/mock_spec.rs`
- [ ] `src/v1/10_test/test/src/mock_synthesis.rs`
- [ ] `src/v1/10_test/test/src/mockable.rs`
- [ ] `src/v1/10_test/test/src/simulator.rs`
- [ ] `src/v1/10_test/test/src/temp.rs`
- [ ] `src/v1/10_test/test/src/window.rs`
- [ ] `src/v1/10_test/testgen-registry-macros/src/lib.rs`
- [ ] `src/v1/10_test/testgen-registry/src/lib.rs`
<!-- openclaw:scout:end -->

## Managed Findings Log

<!-- openclaw:findings:start -->
- 2026-03-14T00:00:00-04:00 initialized workboard scaffold
<!-- openclaw:findings:end -->

## Managed Tree Snapshot

<!-- openclaw:tree:start -->
```text
└── src
    ├── v1
    │   ├── 00_foundation
    │   │   ├── daglang-contract
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       └── lib.rs
    │   │   ├── delegate-macros
    │   │   │   ├── Cargo.toml
    │   │   │   ├── src
    │   │   │   │   └── lib.rs
    │   │   │   └── tests
    │   │   │       └── delegation.rs
    │   │   ├── infra
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       ├── dagbin_cache.rs
    │   │   │       ├── freshness.rs
    │   │   │       ├── hash.rs
    │   │   │       ├── lib.rs
    │   │   │       ├── manifest.rs
    │   │   │       └── workspace_model.rs
    │   │   └── ir
    │   │       ├── Cargo.toml
    │   │       ├── src
    │   │       │   ├── algebra.rs
    │   │       │   ├── boundary.rs
    │   │       │   ├── builder.rs
    │   │       │   ├── cargo.rs
    │   │       │   ├── code_ir
    │   │       │   │   ├── c_ir.rs
    │   │       │   │   ├── lower.rs
    │   │       │   │   ├── mod.rs
    │   │       │   │   └── register_ir.rs
    │   │       │   ├── codegen_bridge.rs
    │   │       │   ├── coerce.rs
    │   │       │   ├── contract.rs
    │   │       │   ├── dag.rs
    │   │       │   ├── dag_topology.rs
    │   │       │   ├── entrypoint.rs
    │   │       │   ├── filename.rs
    │   │       │   ├── generated
    │   │       │   │   └── mod.rs
    │   │       │   ├── git.rs
    │   │       │   ├── invocation_contract.rs
    │   │       │   ├── language
    │   │       │   │   ├── categories
    │   │       │   │   │   ├── config.rs
    │   │       │   │   │   ├── mod.rs
    │   │       │   │   │   └── turing.rs
    │   │       │   │   ├── languages
    │   │       │   │   │   ├── css.rs
    │   │       │   │   │   ├── gitignore.rs
    │   │       │   │   │   ├── html.rs
    │   │       │   │   │   ├── makefile.rs
    │   │       │   │   │   ├── markdown.rs
    │   │       │   │   │   ├── mod.rs
    │   │       │   │   │   ├── rust.rs
    │   │       │   │   │   ├── toml.rs
    │   │       │   │   │   └── yaml.rs
    │   │       │   │   ├── mod.rs
    │   │       │   │   ├── patterns
    │   │       │   │   │   ├── glob.rs
    │   │       │   │   │   ├── mod.rs
    │   │       │   │   │   ├── regex.rs
    │   │       │   │   │   └── variable.rs
    │   │       │   │   └── traits
    │   │       │   │       ├── comment.rs
    │   │       │   │       ├── mod.rs
    │   │       │   │       ├── naming.rs
    │   │       │   │       └── type_system.rs
    │   │       │   ├── layout.rs
    │   │       │   ├── lib.rs
    │   │       │   ├── log_detail.rs
    │   │       │   ├── makefile_render.rs
    │   │       │   ├── node.rs
    │   │       │   ├── patterns
    │   │       │   │   ├── atomic.rs
    │   │       │   │   ├── authenticate.rs
    │   │       │   │   ├── branch.rs
    │   │       │   │   ├── collection.rs
    │   │       │   │   ├── content_upsert.rs
    │   │       │   │   ├── emit.rs
    │   │       │   │   ├── loop_pattern.rs
    │   │       │   │   ├── mod.rs
    │   │       │   │   ├── pattern_op.rs
    │   │       │   │   ├── repeat.rs
    │   │       │   │   ├── transaction.rs
    │   │       │   │   ├── transport_triplet.rs
    │   │       │   │   └── upsert.rs
    │   │       │   ├── plain_render.rs
    │   │       │   ├── platform.rs
    │   │       │   ├── render_ir.rs
    │   │       │   ├── resource
    │   │       │   │   ├── def.rs
    │   │       │   │   ├── defs.rs
    │   │       │   │   ├── handle.rs
    │   │       │   │   ├── managed.rs
    │   │       │   │   ├── mod.rs
    │   │       │   │   ├── registry.rs
    │   │       │   │   └── state.rs
    │   │       │   ├── signature.rs
    │   │       │   ├── symbol.rs
    │   │       │   ├── symbols.rs
    │   │       │   ├── system_model.rs
    │   │       │   ├── transport
    │   │       │   │   ├── agent.rs
    │   │       │   │   ├── agent_adapter.rs
    │   │       │   │   ├── behavior.rs
    │   │       │   │   ├── ci
    │   │       │   │   │   ├── command.rs
    │   │       │   │   │   ├── mod.rs
    │   │       │   │   │   ├── provider.rs
    │   │       │   │   │   ├── providers
    │   │       │   │   │   │   ├── github.rs
    │   │       │   │   │   │   ├── gitlab.rs
    │   │       │   │   │   │   ├── mod.rs
    │   │       │   │   │   │   └── plain.rs
    │   │       │   │   │   └── runner.rs
    │   │       │   │   ├── cli.rs
    │   │       │   │   ├── cloud.rs
    │   │       │   │   ├── credential.rs
    │   │       │   │   ├── credential_policy.rs
    │   │       │   │   ├── file.rs
    │   │       │   │   ├── gcp.rs
    │   │       │   │   ├── git.rs
    │   │       │   │   ├── github
    │   │       │   │   │   ├── api.rs
    │   │       │   │   │   ├── cli.rs
    │   │       │   │   │   ├── issues.rs
    │   │       │   │   │   ├── mod.rs
    │   │       │   │   │   └── pull_request.rs
    │   │       │   │   ├── github_actions.rs
    │   │       │   │   ├── http.rs
    │   │       │   │   ├── llm
    │   │       │   │   │   ├── anthropic.rs
    │   │       │   │   │   ├── chat.rs
    │   │       │   │   │   ├── mock.rs
    │   │       │   │   │   ├── mod.rs
    │   │       │   │   │   ├── openai.rs
    │   │       │   │   │   ├── openai_responses.rs
    │   │       │   │   │   └── provider.rs
    │   │       │   │   ├── middleware.rs
    │   │       │   │   ├── mod.rs
    │   │       │   │   ├── rest.rs
    │   │       │   │   ├── review.rs
    │   │       │   │   ├── scope.rs
    │   │       │   │   ├── tcp.rs
    │   │       │   │   └── tool.rs
    │   │       │   ├── type_lib.rs
    │   │       │   ├── type_op.rs
    │   │       │   ├── type_registry.rs
    │   │       │   ├── type_shape.rs
    │   │       │   ├── typed_io.rs
    │   │       │   ├── types.rs
    │   │       │   ├── validate.rs
    │   │       │   ├── value.rs
    │   │       │   ├── value_bridge.rs
    │   │       │   ├── value_expr.rs
    │   │       │   ├── verified.rs
    │   │       │   └── workspace_layout.rs
    │   │       └── tests
    │   │           └── generated_staleness.rs
    │   ├── 01_surfaces
    │   │   ├── cli
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       ├── binary_args.rs
    │   │   │       └── lib.rs
    │   │   ├── codegen
    │   │   │   ├── Cargo.toml
    │   │   │   ├── src
    │   │   │   │   ├── bin
    │   │   │   │   │   ├── codegen_cli.rs
    │   │   │   │   │   └── testgen_cli.rs
    │   │   │   │   ├── cli_gen.rs
    │   │   │   │   ├── entrypoint.rs
    │   │   │   │   ├── fidelity.rs
    │   │   │   │   ├── file_writer.rs
    │   │   │   │   ├── lambda_gen.rs
    │   │   │   │   ├── lib.rs
    │   │   │   │   ├── registry.rs
    │   │   │   │   ├── rest_gen.rs
    │   │   │   │   ├── template.rs
    │   │   │   │   ├── testgen
    │   │   │   │   │   ├── analyze.rs
    │   │   │   │   │   ├── cardinality.rs
    │   │   │   │   │   ├── codegen.rs
    │   │   │   │   │   ├── mock_corpus.rs
    │   │   │   │   │   ├── mod.rs
    │   │   │   │   │   ├── obligation.rs
    │   │   │   │   │   ├── probe_observer.rs
    │   │   │   │   │   ├── registry_gen.rs
    │   │   │   │   │   └── render_rust.rs
    │   │   │   │   ├── testgen_dag
    │   │   │   │   │   ├── dag_test_discovery.rs
    │   │   │   │   │   ├── mock_interpreter.rs
    │   │   │   │   │   └── mod.rs
    │   │   │   │   └── tool_discovery.rs
    │   │   │   └── tests
    │   │   │       ├── backend_compile_smoke.rs
    │   │   │       └── emitted_binary_smoke.rs
    │   │   ├── daglang-cli
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       ├── commands.rs
    │   │   │       ├── compile
    │   │   │       │   ├── context.rs
    │   │   │       │   ├── render.rs
    │   │   │       │   ├── tests.rs
    │   │   │       │   └── triplets.rs
    │   │   │       ├── compile.rs
    │   │   │       ├── lib.rs
    │   │   │       ├── main.rs
    │   │   │       ├── path_utils.rs
    │   │   │       └── pipeline.rs
    │   │   └── workflow
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           ├── admission.rs
    │   │           ├── coordination.rs
    │   │           ├── errors.rs
    │   │           ├── executor.rs
    │   │           ├── global_plan.rs
    │   │           ├── key.rs
    │   │           ├── lib.rs
    │   │           ├── planner.rs
    │   │           ├── process_registry.rs
    │   │           ├── projection.rs
    │   │           ├── proof.rs
    │   │           ├── schema.rs
    │   │           └── slo.rs
    │   ├── 02_pipeline
    │   │   └── daglang-driver
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           ├── lib.rs
    │   │           ├── pipeline.rs
    │   │           ├── prepare.rs
    │   │           └── receipt.rs
    │   ├── 03_source
    │   │   ├── daglang-resolve
    │   │   │   ├── Cargo.toml
    │   │   │   ├── src
    │   │   │   │   └── lib.rs
    │   │   │   └── tests
    │   │   │       └── module_graph.rs
    │   │   └── daglang-syntax
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           ├── ast_utils.rs
    │   │           ├── callable.rs
    │   │           ├── diagnostic.rs
    │   │           ├── lexer.rs
    │   │           ├── lib.rs
    │   │           └── parser.rs
    │   ├── 04_semantics
    │   │   └── daglang-typecheck
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           ├── lib.rs
    │   │           └── tests.rs
    │   ├── 05_graph
    │   │   ├── daglang-eval
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       ├── eval.rs
    │   │   │       ├── eval_core.rs
    │   │   │       ├── eval_stack.rs
    │   │   │       ├── expr.rs
    │   │   │       ├── lib.rs
    │   │   │       └── v2_tests.rs
    │   │   └── daglang-lower
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           ├── anf.rs
    │   │           ├── eval.rs
    │   │           ├── expr.rs
    │   │           ├── lib.rs
    │   │           ├── scope.rs
    │   │           ├── spec.rs
    │   │           ├── tests.rs
    │   │           └── transport.rs
    │   ├── 06_artifacts
    │   │   └── daglang-derive
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           └── lib.rs
    │   ├── 07_emit
    │   │   └── daglang-emit
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           ├── backend_harness.rs
    │   │           ├── computation.rs
    │   │           ├── dag_emit.rs
    │   │           ├── fn_codegen.rs
    │   │           ├── language_model.rs
    │   │           ├── lib.rs
    │   │           ├── lower_c.rs
    │   │           ├── lower_go.rs
    │   │           ├── lower_mips.rs
    │   │           ├── lower_rust.rs
    │   │           ├── lower_to_ir.rs
    │   │           ├── plan.rs
    │   │           ├── render_c.rs
    │   │           ├── render_go.rs
    │   │           ├── render_mips.rs
    │   │           ├── render_rust.rs
    │   │           ├── rust_exec_runtime.rs
    │   │           ├── service_emit.rs
    │   │           ├── test_gen.rs
    │   │           ├── test_mock_emit.rs
    │   │           ├── transport_analysis.rs
    │   │           ├── type_codegen.rs
    │   │           ├── type_mapping.rs
    │   │           ├── v2_crate_emit.rs
    │   │           └── v2_runtime_shim.rs
    │   ├── 08_materialize
    │   │   ├── blob
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       └── lib.rs
    │   │   ├── interp
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       └── lib.rs
    │   │   ├── resolve
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       ├── builder.rs
    │   │   │       ├── dry_run.rs
    │   │   │       ├── fs_env.rs
    │   │   │       ├── lib.rs
    │   │   │       ├── resolve.rs
    │   │   │       └── service_ops
    │   │   │           ├── mod.rs
    │   │   │           └── service_ops_impl.rs
    │   │   └── transport
    │   │       ├── Cargo.toml
    │   │       ├── src
    │   │       │   ├── backend.rs
    │   │       │   ├── classify.rs
    │   │       │   ├── cli.rs
    │   │       │   ├── credential.rs
    │   │       │   ├── executor.rs
    │   │       │   ├── freshness_policy.rs
    │   │       │   ├── lib.rs
    │   │       │   ├── metrics.rs
    │   │       │   ├── middleware
    │   │       │   │   └── mod.rs
    │   │       │   ├── ops.rs
    │   │       │   ├── pipeline.rs
    │   │       │   ├── preflight.rs
    │   │       │   ├── rate_limit.rs
    │   │       │   ├── resource_io.rs
    │   │       │   ├── retry.rs
    │   │       │   ├── system_models.rs
    │   │       │   ├── test_backend.rs
    │   │       │   └── transport_types.rs
    │   │       └── tests
    │   │           └── basic_transports_integration.rs
    │   ├── 09_execute
    │   │   └── exec
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           ├── box_draw.rs
    │   │           ├── ci_context.rs
    │   │           ├── diagnostic.rs
    │   │           ├── display.rs
    │   │           ├── env.rs
    │   │           ├── error.rs
    │   │           ├── execute
    │   │           │   ├── mod.rs
    │   │           │   └── tests.rs
    │   │           ├── frame_build.rs
    │   │           ├── frame_write.rs
    │   │           ├── freshness.rs
    │   │           ├── helpers.rs
    │   │           ├── intercept.rs
    │   │           ├── ledger.rs
    │   │           ├── lib.rs
    │   │           ├── lower.rs
    │   │           ├── pattern_op.rs
    │   │           ├── progress.rs
    │   │           ├── render.rs
    │   │           ├── terminal.rs
    │   │           └── topo.rs
    │   ├── 10_test
    │   │   ├── generated-tests
    │   │   │   ├── Cargo.toml
    │   │   │   ├── build.rs
    │   │   │   └── src
    │   │   │       └── lib.rs
    │   │   ├── test
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       ├── auto_mock.rs
    │   │   │       ├── boundary.rs
    │   │   │       ├── composition.rs
    │   │   │       ├── corpus.rs
    │   │   │       ├── fermi.rs
    │   │   │       ├── fidelity.rs
    │   │   │       ├── json.rs
    │   │   │       ├── lib.rs
    │   │   │       ├── mock.rs
    │   │   │       ├── mock_requirements.rs
    │   │   │       ├── mock_spec.rs
    │   │   │       ├── mock_synthesis.rs
    │   │   │       ├── mockable.rs
    │   │   │       ├── simulator.rs
    │   │   │       ├── temp.rs
    │   │   │       └── window.rs
    │   │   ├── testgen-registry
    │   │   │   ├── Cargo.toml
    │   │   │   └── src
    │   │   │       └── lib.rs
    │   │   └── testgen-registry-macros
    │   │       ├── Cargo.toml
    │   │       └── src
    │   │           └── lib.rs
    │   ├── ARCHITECTURE.md
    │   ├── README.md
    │   └── SUSTAINABILITY.md
    └── v2
        ├── 00_core.dag
        ├── 01_tokenize.dag
        ├── 02_parse.dag
        ├── 03_resolve.dag
        ├── 04_typecheck.dag
        ├── 05_emit.dag
        ├── 06_pipeline.dag
        ├── DESIGN-parse-split.md
        ├── DESIGN-typed-ast.md
        ├── DESIGN.md
        ├── POSTMORTEM.md
        ├── WORKBOARD.md
        ├── tests
        │   ├── Cargo.toml
        │   └── src
        │       └── lib.rs
        └── workstreams
            ├── WS-B-parser-tokenizer.md
            ├── WS-C-typecheck-resolve.md
            ├── WS-D-emitter.md
            ├── WS-E-pipeline-core.md
            ├── WS-F-rust-codegen.md
            └── WS-G-runtime-shims.md
```
<!-- openclaw:tree:end -->
