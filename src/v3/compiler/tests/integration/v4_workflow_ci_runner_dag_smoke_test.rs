//! **Layer:** integration
//!
//! T-21/T-24 Wave-0: `src/v4/workflow/ci.dag` (`v4.workflow.ci`) exposes affected-set-driven
//! `TestClaim` roster selection over `RerunNodeSet` (kernel `filter`/`any`/`contains`;
//! evaluation-node projection in `verification.dag`). Receipt
//! claims live in `src/v4/test/claim/workflow/affected_set_ci_runner.dag`.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (same posture as peer v4 smoke tests).
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-21 + T-24.
//!
//! **Dissolution:** remove when `.dag` TestClaim execution covers these claims without
//! this hand-Rust parse harness.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem, SurfaceLiteral, SurfaceRecordField};
use v3_compiler::tokenize_for_test;

const CI_DAG: &str = include_str!("../../../../v4/workflow/ci.dag");
const CI_DAG_PATH: &str = "src/v4/workflow/ci.dag";
const CI_YML: &str = include_str!("../../../../../.github/workflows/ci.yml");
const CI_YML_PATH: &str = ".github/workflows/ci.yml";
const M1_BINDING_TEST_FILTER: &str =
    "v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_m1_rust_emit_probe_modeled_and_bound_to_ci_yml";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/affected_set_ci_runner.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/workflow/affected_set_ci_runner.dag";
const V4_CI_COMPONENT_AFFECTED_RS: &str = include_str!("../../src/v4_ci_component_affected.rs");

/// Single-authority workflow_policy path buckets (`ci.dag` predicates ↔ Rust host mirror).
const CI_WORKFLOW_POLICY_PREFIXES: &[&str] = &[
    ".github/workflows/",
    "src/v4/workflow/ci",
    "src/v3/compiler/src/v4_ci_component_affected",
    "src/v3/compiler/src/v4_ci_runner_pool",
    "src/v3/compiler/src/bin/detect_ci_affected_components",
    "scripts/check-workflow-path-regex-inventory",
    "scripts/workflow-path-regex-forbidden-substrings",
];

const CI_V2_PREFIXES: &[&str] = &["src/v2/"];
const CI_V2_EXACT_PATHS: &[&str] = &["Cargo.toml", "Cargo.lock"];
const CI_V3_PREFIXES: &[&str] = &["src/v3/", "dsl/"];
const CI_V4_PREFIXES: &[&str] = &[
    "src/v4/",
    "fixtures/v4-mvp1/",
    "scripts/v4-mvp1/",
    "scripts/v4-m1/",
    "scripts/v4-testclaim-",
    "dsl/std/",
];
const CI_V4_EXACT_PATHS: &[&str] = &["Cargo.toml", "Cargo.lock"];

fn assert_ci_dag_rust_prefix_parity(prefixes: &[&str]) {
    for prefix in prefixes {
        assert!(
            CI_DAG.contains(&format!("prefix: \"{prefix}\"")),
            "{CI_DAG_PATH}: authority must declare prefix `{prefix}`"
        );
        assert!(
            V4_CI_COMPONENT_AFFECTED_RS.contains(&format!("\"{prefix}\"")),
            "v4_ci_component_affected.rs mirror must declare prefix `{prefix}`"
        );
    }
}

fn assert_ci_dag_rust_exact_path_parity(paths: &[&str]) {
    for path in paths {
        assert!(
            CI_DAG.contains(&format!("changed == \"{path}\"")),
            "{CI_DAG_PATH}: authority must declare exact path `{path}`"
        );
        assert!(
            V4_CI_COMPONENT_AFFECTED_RS.contains(&format!("\"{path}\"")),
            "v4_ci_component_affected.rs mirror must declare exact path `{path}`"
        );
    }
}

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return false;
        };
        item_path.len() == path.len()
            && item_path
                .iter()
                .zip(path.iter())
                .all(|(a, &b)| a.as_str() == b)
            && names.iter().any(|n| n == name)
    })
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn data_body<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a SurfaceExpr {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body: Some(body),
                ..
            } if item_name == name => Some(body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data body `{name}`"))
}

fn record_body_field<'a>(body: &'a SurfaceExpr, field_name: &str) -> &'a SurfaceExpr {
    let SurfaceExpr::Record { fields, .. } = body else {
        panic!("expected record body, got {body:?}");
    };
    fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|SurfaceRecordField { value, .. }| value)
        .unwrap_or_else(|| panic!("record body missing `{field_name}` field"))
}

fn expr_string(expr: &SurfaceExpr) -> &str {
    match expr {
        SurfaceExpr::Literal {
            value: SurfaceLiteral::String(value),
            ..
        } => value,
        other => panic!("expected string literal expr, got {other:?}"),
    }
}

fn expr_int(expr: &SurfaceExpr) -> i64 {
    match expr {
        SurfaceExpr::Literal {
            value: SurfaceLiteral::Int(value),
            ..
        } => value
            .parse()
            .unwrap_or_else(|_| panic!("expected int literal expr, got non-integer `{value}`")),
        other => panic!("expected int literal expr, got {other:?}"),
    }
}

fn expr_bool(expr: &SurfaceExpr) -> bool {
    match expr {
        SurfaceExpr::Literal {
            value: SurfaceLiteral::Bool(value),
            ..
        } => *value,
        SurfaceExpr::Var { name, .. } if name == "true" => true,
        SurfaceExpr::Var { name, .. } if name == "false" => false,
        other => panic!("expected bool literal expr, got {other:?}"),
    }
}

fn workflow_step_block<'a>(workflow_yml: &'a str, step_name: &str) -> &'a str {
    let marker = format!("    - name: {step_name}");
    let start = workflow_yml
        .find(&marker)
        .unwrap_or_else(|| panic!("{CI_YML_PATH}: missing workflow step `{step_name}`"));
    let rest = &workflow_yml[start..];
    let end = rest.find("\n    - name: ").unwrap_or(rest.len());
    &rest[..end]
}

fn surface_declares_test_claim_data(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    use v3_compiler::parse_surface::SurfaceType;
    module.items.iter().any(|item| match item {
        SurfaceItem::Data {
            name: item_name,
            ty: SurfaceType::Named { name: ty_name, .. },
            ..
        } => item_name == name && ty_name == "TestClaim",
        _ => false,
    })
}

#[test]
fn v4_workflow_ci_dag_tokenizes_and_parses() {
    let _module = parse_module(CI_DAG, CI_DAG_PATH);
}

#[test]
fn v4_workflow_ci_test_claim_selection_entrypoints() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "workflow", "ci"]],
        "{CI_DAG_PATH}: module authority path"
    );
    for name in [
        "ci_select_from_rerun_nodes",
        "ci_select_from_affected_set",
        "ci_component_affected_from_git_diff",
        "test_claim_in_rerun_frontier",
        "ci_all_gate_run_policies_resolve",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{CI_DAG_PATH}: must declare {name}"
        );
    }
    assert!(
        import_includes_name(
            &module,
            &["v4", "lens", "affected_set"],
            "affected_set_rerun_nodes"
        ),
        "{CI_DAG_PATH}: must import affected_set_rerun_nodes from T-21 lens"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "verification"],
            "test_claim_evaluation_touches_rerun_frontier"
        ),
        "{CI_DAG_PATH}: must import subtree-aware frontier membership from verification"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "verification"],
            "test_claim_ci_selection_fail_closed"
        ),
        "{CI_DAG_PATH}: must import fail-closed diagnostic selection guard from verification"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "algebra"], "filter"),
        "{CI_DAG_PATH}: must import filter from std.algebra"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "algebra"], "any"),
        "{CI_DAG_PATH}: must import any from std.algebra for path-bucket existence"
    );
    assert!(
        CI_DAG.contains("any(xs: git_diff.changed_paths, predicate: ci_changed_path_affects_v2)"),
        "{CI_DAG_PATH}: component affected-set must use std.algebra any"
    );
    assert!(
        !CI_DAG.contains("CiGitDiffReadOutcome"),
        "{CI_DAG_PATH}: git diff read must use canonical Outcome<GitDiffNameOnly>, not a parallel coproduct"
    );
    assert!(
        CI_DAG.contains(
            "ci_component_affected_from_git_diff_read(outcome: Outcome<GitDiffNameOnly>)"
        ),
        "{CI_DAG_PATH}: git diff read boundary must project through std.diagnostic Outcome"
    );
    assert!(
        !CI_DAG.contains("ci_m1_probe_cargo_fanout_slots_from_fleet"),
        "{CI_DAG_PATH}: M1 cargo jobs must not use reverse-engineered fleet fanout derivation"
    );
    assert!(
        CI_DAG.contains("data m1_probe_cargo_check_jobs: Int = 4"),
        "{CI_DAG_PATH}: M1 cargo parallelism must be an explicit operator constant in Wave-0"
    );
    assert!(
        CI_DAG.contains("RerunNodeSetFailClosed { evidence: _ } => roster"),
        "{CI_DAG_PATH}: fail-closed must return full roster on RerunNodeSetFailClosed (AI-16/R1-7)"
    );
    assert!(
        CI_DAG.contains("test_claim_ci_selection_fail_closed(c: claim)"),
        "{CI_DAG_PATH}: DiagnosticClaim rows must bypass narrow filter via fail-closed guard"
    );
}

#[test]
fn v4_workflow_ci_workflow_policy_prefixes_align_rust_mirror() {
    assert_ci_dag_rust_prefix_parity(CI_WORKFLOW_POLICY_PREFIXES);
}

#[test]
fn v4_workflow_ci_component_bucket_prefixes_align_rust_mirror() {
    assert_ci_dag_rust_prefix_parity(CI_V2_PREFIXES);
    assert_ci_dag_rust_prefix_parity(CI_V3_PREFIXES);
    assert_ci_dag_rust_prefix_parity(CI_V4_PREFIXES);
    assert_ci_dag_rust_exact_path_parity(CI_V2_EXACT_PATHS);
    assert_ci_dag_rust_exact_path_parity(CI_V4_EXACT_PATHS);
}

#[test]
fn v4_workflow_ci_m1_rust_emit_probe_modeled_and_bound_to_ci_yml() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiCommand"
        )),
        "{CI_DAG_PATH}: CiCommand must exist"
    );
    assert!(
        CI_DAG.contains("| M1RustEmitProbeCommand"),
        "{CI_DAG_PATH}: M1 probe must be a CiCommand arm"
    );
    assert!(
        CI_DAG.contains("feature:project-github-actions-landed")
            && CI_DAG.contains("consumer:v4.workflow.ci m1_ci_live_workflow_signal")
            && CI_DAG.contains("bind src/v4/TASKS.md T-24"),
        "{CI_DAG_PATH}: M1 live-workflow bridge must carry checkable P5 dissolution tags"
    );
    assert!(
        surface_declares_fn(&module, "ci_command_authority_ok"),
        "{CI_DAG_PATH}: command authority must cover M1RustEmitProbeCommand"
    );
    assert!(
        CI_DAG.contains("M1RustEmitProbeCommand => true"),
        "{CI_DAG_PATH}: M1 probe command must pass authority check"
    );
    assert!(
        CI_DAG.contains("id: m1_rust_emit_probe_execution")
            && CI_DAG.contains("id: m1_rust_emit_probe_signal"),
        "{CI_DAG_PATH}: ci_pipeline must declare M1 job and gate"
    );
    let live_signal = data_body(&module, "m1_ci_live_workflow_signal");
    let binding_smoke_step_name =
        expr_string(record_body_field(live_signal, "binding_smoke_step_name"));
    let step_name = expr_string(record_body_field(live_signal, "step_name"));
    let script_path = expr_string(record_body_field(live_signal, "script_path"));
    let non_blocking = expr_bool(record_body_field(live_signal, "non_blocking"));
    let timeout_minutes = expr_int(record_body_field(live_signal, "timeout_minutes"));
    let binding_smoke_step = workflow_step_block(CI_YML, binding_smoke_step_name);
    assert!(
        binding_smoke_step.contains(&format!(
            "cargo test -p v3-compiler --test integration {M1_BINDING_TEST_FILTER} -- --exact --quiet"
        )),
        "{CI_YML_PATH}: `{binding_smoke_step_name}` must execute the M1 model/YAML binding receipt (gunbc#846 zero-test-filter bypass)"
    );
    let m1_step = workflow_step_block(CI_YML, step_name);
    assert!(
        m1_step.contains("if: needs.affected.outputs.v4 == 'true' || needs.affected.outputs.workflow_policy == 'true'"),
        "{CI_YML_PATH}: `{step_name}` must run for v4 and workflow-policy changes"
    );
    if non_blocking {
        assert!(
            m1_step.contains("continue-on-error: true"),
            "{CI_YML_PATH}: `{step_name}` must set continue-on-error when modeled non_blocking is true"
        );
    } else {
        assert!(
            !m1_step.contains("continue-on-error: true"),
            "{CI_YML_PATH}: `{step_name}` must not set continue-on-error when modeled non_blocking is false"
        );
    }
    assert!(
        m1_step.contains(&format!("timeout-minutes: {timeout_minutes}")),
        "{CI_YML_PATH}: `{step_name}` must set timeout-minutes from modeled timeout_minutes"
    );
    assert!(
        m1_step.contains(&format!("run: bash {script_path}")),
        "{CI_YML_PATH}: `{step_name}` must invoke the modeled probe script"
    );
    assert!(
        CI_DAG.contains("type SelfHostedRunnerPool"),
        "{CI_DAG_PATH}: must model self-hosted runner pools (T-24 addendum)"
    );
    assert!(
        !CI_DAG.contains("RunnerArchUnknown"),
        "{CI_DAG_PATH}: runner arch must be closed Arm64 fleet fact, not an open fallback enum"
    );
    assert!(
        CI_DAG.contains("type RunnerArch")
            && CI_DAG.contains("= RunnerArchArm64")
            && !CI_DAG.contains("RunnerArchUnknown"),
        "{CI_DAG_PATH}: runner arch must be a closed single-variant coproduct"
    );
    assert!(
        CI_DAG.contains("data ci_srv1_pool: SelfHostedRunnerPool")
            && CI_DAG.contains("runner_count: 20")
            && CI_DAG.contains("jobserver_token_cap: 25")
            && CI_DAG.contains("data ci_srv2_pool: SelfHostedRunnerPool")
            && CI_DAG.contains("runner_count: 30")
            && CI_DAG.contains("jobserver_token_cap: 36"),
        "{CI_DAG_PATH}: srv1/srv2 pool rows must match operator spec"
    );
    assert!(
        CI_DAG.contains("data m1_probe_cargo_check_jobs: Int = 4"),
        "{CI_DAG_PATH}: M1 cargo parallelism must be an explicit operator constant"
    );
    let m1_jobs = v3_compiler::v4_ci_runner_pool::m1_probe_cargo_check_jobs();
    assert_eq!(
        m1_jobs, 4,
        "Rust transport mirror of m1_probe_cargo_check_jobs must match ci.dag operator constant"
    );
    assert!(
        m1_step.contains(&format!("V4_M1_CARGO_CHECK_JOBS: \"{m1_jobs}\"")),
        "{CI_YML_PATH}: M1 step must project modeled cargo-check job cap"
    );
}

#[test]
fn v4_workflow_ci_bootstrap_gate_skip_policy_is_modeled() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiGateRunPolicy"
        )),
        "{CI_DAG_PATH}: must model live gate run policy"
    );
    assert!(
        CI_DAG.contains("| RequiresJobAttempt { job: Symbol }"),
        "{CI_DAG_PATH}: gate run policy must carry the required attempted job"
    );
    assert!(
        CI_DAG.contains("run_policy: RequiresJobAttempt { job: v2_compile_src_v4 }"),
        "{CI_DAG_PATH}: structural v2 compile gate must require the bootstrap compile attempt"
    );
    assert!(
        CI_DAG.contains("ci_all_gate_run_policies_resolve(gates: p.gates, jobs: p.jobs)"),
        "{CI_DAG_PATH}: ci_pipeline_well_formed must reject dangling gate run-policy jobs"
    );
    assert!(
        CI_YML.contains(
            "if: always() && needs.affected.outputs.v4 == 'true' && steps.v4_bootstrap_compile.outcome != 'skipped'"
        ),
        "{CI_YML_PATH}: bootstrap gate result skip guard must stay projected from modeled gate run policy"
    );
}

#[test]
fn v4_workflow_affected_set_ci_runner_claim_dag_tokenizes_and_parses() {
    let _module = parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_workflow_affected_set_ci_runner_claim_wiring() {
    let module = parse_module(CLAIM_DAG, CLAIM_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec![
            "v4",
            "test",
            "claim",
            "workflow",
            "affected_set_ci_runner"
        ]],
        "{CLAIM_PATH}: module authority path"
    );
    for name in ["ci_select_from_rerun_nodes", "ci_select_from_affected_set"] {
        assert!(
            import_includes_name(&module, &["v4", "workflow", "ci"], name),
            "{CLAIM_PATH}: must import {name} from canonical workflow/ci authority"
        );
    }
    for claim in [
        "ci_runner_narrow_selection_claim",
        "ci_runner_fail_closed_superset_claim",
        "ci_runner_shape_collision_claim",
        "ci_runner_inner_frontier_claim",
    ] {
        assert!(
            surface_declares_test_claim_data(&module, claim),
            "{CLAIM_PATH}: must declare structural receipt `{claim}`"
        );
    }
    assert!(
        surface_declares_fn(&module, "ci_runner_narrow_selection_holds")
            && surface_declares_fn(&module, "ci_runner_fail_closed_superset_holds")
            && surface_declares_fn(&module, "ci_runner_shape_collision_holds")
            && surface_declares_fn(&module, "ci_runner_inner_frontier_holds"),
        "{CLAIM_PATH}: receipt predicates must exercise ci selection entrypoints"
    );
}

#[test]
fn v4_workflow_ci_t38_dissolution_step_modeled_and_wired() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        CI_DAG.contains("| TestClaimCorpusEvalCommand"),
        "{CI_DAG_PATH}: T-38 dissolution step must declare TestClaimCorpusEvalCommand arm in CiCommand"
    );
    assert!(
        CI_DAG.contains("feature:t38-testclaim-corpus-eval"),
        "{CI_DAG_PATH}: TestClaimCorpusEvalCommand must carry t38-testclaim-corpus-eval dissolution tag"
    );
    assert!(
        CI_DAG.contains("ci_select_from_affected_set narrows roster to content_hash frontier"),
        "{CI_DAG_PATH}: TestClaimCorpusEvalCommand dissolution comment must name ci_select_from_affected_set \
         as the IRT-1 narrowing authority (checks the new dissolution comment, not the pre-existing helper)"
    );
    assert!(
        CI_DAG.contains("fn_name == ci_testclaim_corpus_selection_fn"),
        "{CI_DAG_PATH}: ci_command_authority_ok must enforce selection_fn == ci_testclaim_corpus_selection_fn (not unconditional true)"
    );
    assert!(
        CI_DAG.contains("data ci_testclaim_corpus_selection_fn: Symbol = ci_select_from_affected_set"),
        "{CI_DAG_PATH}: ci_testclaim_corpus_selection_fn must be declared as ci_select_from_affected_set (IRT-1 P2 single authority)"
    );
    assert!(
        CI_DAG.contains("testclaim_corpus_eval_execution"),
        "{CI_DAG_PATH}: ci_pipeline must include testclaim_corpus_eval_execution job"
    );
    assert!(
        CI_DAG.contains("testclaim_corpus_eval_signal"),
        "{CI_DAG_PATH}: ci_pipeline must include testclaim_corpus_eval_signal gate"
    );
    assert!(
        CI_DAG.contains("command: TestClaimCorpusEvalCommand { selection_fn: ci_testclaim_corpus_selection_fn }"),
        "{CI_DAG_PATH}: testclaim_corpus_eval_execution job must bind selection_fn to the canonical authority"
    );
    assert!(
        CI_DAG.contains("ci_cache_cmd_testclaim_corpus_eval_tag"),
        "{CI_DAG_PATH}: ci_command_cache_digest must cover TestClaimCorpusEvalCommand"
    );
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiCommand"
        )),
        "{CI_DAG_PATH}: CiCommand sum type must exist"
    );
}
