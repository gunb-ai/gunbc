# v4 CI Rust+DAG Shared Closure Worksheet

> **Status:** RATIFIED — Modeling DFS Arbiter follow-up `node://adhoc-197c65c6-8cd`.
> **PR:** #4171.
> **Work item:** `node://adhoc-bde1378f-b7e` / #4091 §1.2 four-compile redundancy collapse.
> **Gate:** load-bearing v2 stage0 emit pipeline; no merge until arbiter sign-off and CI-manager release.

## §10.0-adapted worksheet

```text
Migration class:        V2-STAGE0-CLI-SHARED-TARGET-EMIT (rust+dag orchestration)
Representative failure:  Required v4 CI floor compiles the same 332-source src/v4 closure twice:
                         M1 rust emit and bootstrap dag emit. A shell-only reuse patch can make
                         bootstrap false-green if the reused DAG artifact is not byte/diagnostic
                         identical to standalone --target dag.
Immediate local patch:   Have v4-bootstrap-viability.sh trust M1's DAG output without proving emit parity,
                         or reimplement resolved->emit logic in stage0/main.rs for rust+dag.
Why forbidden:           P2 single-authority violation. The emit pipeline is load-bearing bootstrap seed
                         territory (SELF_HOSTING.md); CLI orchestration must not own a parallel emitter.
DFS path:
  v2 emit authority:
    - src/v2/stage0/src/v2_compiler_compile.rs — owns compile_to_resolved,
      default_artifact_plan, emit_from_artifact_plan, and the new emit_resolved_for_target.
  v2 CLI orchestration:
    - src/v2/stage0/src/main.rs — parses --target, writes per-target output dirs, and calls
      v2_compiler_compile::emit_resolved_for_target for each target. It does not construct
      ArtifactPlan or call emit_from_artifact_plan directly.
  v4 CI model:
    - src/v4/workflow/ci.dag — records the shared DAG artifact env contract for M1/bootstrap.
  host transport:
    - scripts/v4-rust-full-tree-emit-probe.sh — uses --target rust+dag when V4_RUST_DAG_SHARED_CLOSURE_OUT is set.
    - scripts/v4-bootstrap-viability.sh — validates the proven shared DAG receipt via
      V4_BOOTSTRAP_REUSE_LOG, otherwise preserves independent --target dag compile behavior.
Deepest unsound boundary:
  Resolved graph to per-target artifact emission must have exactly one authority. If compile_sources
  and rust+dag multi-target emission each build their own artifact plan, parity is coincidental and
  the bootstrap gate can lose fail-closed independence.
Systemic fix:
  Factor emit_resolved_for_target(resolved, target) inside v2_compiler_compile and make both
  compile_sources and the CLI multi-target loop consume it. The CLI only sequences targets and
  writes files. Add a P5(b)-style parity receipt proving standalone --target dag output equals
  resolved->Dag output on a fixed v4 slice.
Non-goals:
  - New substrate target model, new target language semantics, or changes to emit_rust / emit_dag_artifact.
  - Replacing the broader CI interpreter / CiUpsertStep program.
  - Making rust+dag a general target algebra in .dag source before the bootstrap seed supports it.
Falsification probe:
  Compile the fixed fixtures/v4-mvp1 v4 slice through compile_sources(..., Dag) and through
  compile_to_resolved + emit_resolved_for_target(..., Dag). Assert diagnostics and emitted files are
  exactly equal. Any divergence blocks V4_BOOTSTRAP_REUSE_LOG use.
Metric allowed only as secondary:
  Required-path wall-clock reduction (~14m). Acceptance is single-authority emit + parity receipt.
```

## §1 Authority Placement

The new home for target emission from an already-resolved graph is
`v2_compiler_compile::emit_resolved_for_target`. It composes the existing emit authorities:

- `default_artifact_plan(module_names, target)`
- `emit_from_artifact_plan(typed_graph, artifact_plan)`
- existing diagnostic concatenation and emit-error file suppression

`compile_sources` now delegates to:

```text
compile_to_resolved(sources) -> emit_resolved_for_target(resolved, target)
```

The CLI multi-target path delegates to the same helper for each target. The CLI does not derive an
artifact plan, inspect typed modules for emit, or call `emit_from_artifact_plan`.

## §2 CI Gate Semantics

`scripts/v4-rust-full-tree-emit-probe.sh` keeps its existing Rust cargo-check role. When CI supplies
`V4_RUST_DAG_SHARED_CLOSURE_OUT`, it asks `gunbc` for `--target rust+dag`, moves the Rust and DAG halves to
their modeled output dirs, and copies the shared compile receipt for bootstrap.

`scripts/v4-bootstrap-viability.sh` only reuses that DAG half when `V4_BOOTSTRAP_REUSE_LOG` is set
and the output dir exists with a clean compile receipt. Without that env var, it still performs the
old independent `--target dag` compile.

The reuse is acceptable only with the parity receipt in §3; otherwise the bootstrap gate must retain
the independent compile.

## §3 Receipt

`src/v2/tests/src/pipeline.rs::dag_emit_from_resolved_matches_compile_sources_for_v4_slice` is the
P5(b)-style receipt:

- fixed input: `fixtures/v4-mvp1`
- standalone path: `compile_sources(sources, RenderTarget::Dag)`
- shared-closure path: `compile_to_resolved(sources)` then `emit_resolved_for_target(..., Dag)`
- assertion: diagnostics equal and emitted files equal byte-for-byte

The paired `src/v3/compiler/tests/integration/v4_workflow_ci_runner_dag_smoke_test.rs` change is
not a new substrate authority. It is a bounded hand-Rust binding smoke for the CI/YAML bridge: the
modeled `src/v4/workflow/ci.dag` command/env facts must appear in `.github/workflows/ci.yml`, and
the parity receipt command must precede bootstrap reuse. P5 receipt form: concrete ROADMAP-row
deferral to `ROADMAP.md` row T-PB-B / `pb_rust_tests_outside_residual_zero`, with existing modeled
workflow receipts in `src/v4/test/claim/workflow/{ci_component_affected,affected_set_ci_runner,runner_pool_m1_probe}.dag`.
Dissolution trigger: when workflow emission from `ci.dag` plus `.dag` `TestClaim` execution covers
the same facts, this Rust string-ordering smoke is deleted or replaced by the generated
workflow/TestClaim receipt in the same lane; it must not become a permanent floor gate.

## §4 Lane H Lens Dispositions

Lane H does not create a CI-side lens authority for this worksheet. The live dispositions are:

- `src/v4/workflow/ci.dag` testgen slots consume `Generator<TestgenConcept>` from `v4.lens.testgen`;
  no flattened Symbol slot/category authority is accepted.
- `src/v4/extdeps/languages/typescript.dag` keeps TypeScript SG-2 on the shared
  `TargetTypeExpressionProjection` carrier. Record labels are now live through
  `TargetGenericApply.field_label_separator`; arrow domain parameter-list fidelity remains gated
  to the shared `target-arrow-domain-param-list-carrier` follow-on.
- Any remaining hand-Rust lens or workflow binding smoke is a T-PB-B same-path receipt and dissolves
  when the corresponding `.dag` `TestClaim` or generated workflow runner executes the modeled facts.

## §8 Modeling DFS Arbiter Checklist

- [x] `emit_resolved_for_target` is the only resolved→target emit authority.
- [x] CLI rust+dag path performs orchestration only.
- [x] Parity receipt passes locally and in CI.
- [x] Bootstrap reuse remains disabled unless the parity receipt exists.
- [x] CI Manager gate resolved by #4171 merge; no surviving worksheet reference depends on the
  removed v4 task ledger.
- [x] Testgen §8 receipt shape is ratified: `TestgenSlotSelection` carries
  `Generator<TestgenConcept>`, `ClaimAnchorKey`, selected state, and closed reason; no parallel
  Symbol-only testgen lane.
