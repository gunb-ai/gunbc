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
const BANKRUPTCY_TIER0_BINDING_TEST_FILTER: &str =
    "v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_bankruptcy_tier0_modeled_and_legacy_jobs_deleted";
const CI_MODEL_YAML_BINDING_STEP_NAME: &str = "M1 v4 workflow CI model/YAML binding smoke";
const T15_SELF_HOST_STEP_NAME: &str = "T-15 self-host fixed-point harness (stage1==stage2)";
const T15_SELF_HOST_HARNESS_TEST_FILTER: &str =
    "v4_t15_self_host_fixed_point_harness_test::t_15_self_host_fixed_point";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/affected_set_ci_runner.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/workflow/affected_set_ci_runner.dag";
const CI_AFFECTED_COMPONENTS_LIB: &str =
    include_str!("../../../../../tools/ci_affected_components/src/lib.rs");

const CI_CHANGED_PATH_AFFECTS_FNS: &[&str] = &[
    "ci_changed_path_affects_v2",
    "ci_changed_path_affects_v3",
    "ci_changed_path_affects_v4",
    "ci_changed_path_affects_workflow_policy",
    "ci_changed_path_affects_release_distribution",
];

struct CiAffectedFixture {
    path: &'static str,
    v2: bool,
    v3: bool,
    v4: bool,
    workflow_policy: bool,
    release_distribution: bool,
}

/// Behavioral fixtures aligned with `src/v4/test/claim/workflow/ci_component_affected.dag`.
const CI_AFFECTED_BEHAVIORAL_FIXTURES: &[CiAffectedFixture] = &[
    CiAffectedFixture {
        path: "src/v2/stage0/Cargo.toml",
        v2: true,
        v3: false,
        v4: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "dsl/gunbc/ci.dag",
        v2: false,
        v3: true,
        v4: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "dsl/std/node.dag",
        v2: false,
        v3: true,
        v4: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "scripts/v4-m1-rust-emit-probe.sh",
        v2: false,
        v3: false,
        v4: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/workflow/ci.dag",
        v2: false,
        v3: false,
        v4: true,
        workflow_policy: true,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/workflow/release.dag",
        v2: false,
        v3: false,
        v4: true,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "Cargo.lock",
        v2: true,
        v3: false,
        v4: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "docs/README.md",
        v2: false,
        v3: false,
        v4: false,
        workflow_policy: false,
        release_distribution: false,
    },
];

fn fn_body_end(rest: &str) -> usize {
    ["\npub fn ", "\nfn ", "\n#[cfg(test)]"]
        .iter()
        .filter_map(|prefix| rest.find(prefix))
        .min()
        .unwrap_or(rest.len())
}

fn extract_fn_body<'a>(source: &'a str, fn_name: &str) -> &'a str {
    let marker = format!("fn {fn_name}");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing fn `{fn_name}`"));
    let after = start + marker.len();
    let rest = &source[after..];
    &source[start..after + fn_body_end(rest)]
}

fn literals_after_marker(line: &str, marker: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let mut search = line;
    while let Some(idx) = search.find(marker) {
        let rest = &search[idx + marker.len()..];
        if let Some(end) = rest.find('"') {
            out.insert(rest[..end].to_string());
            search = &rest[end + 1..];
        } else {
            break;
        }
    }
    out
}

fn dag_prefix_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, "prefix: \""));
    }
    out
}

fn rust_prefix_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, ".starts_with(\""));
    }
    out
}

fn dag_exact_path_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, "changed == \""));
    }
    out
}

fn rust_exact_path_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, "path == \""));
    }
    out
}

fn assert_ci_dag_rust_bucket_parity(fn_name: &str) {
    let dag_body = extract_fn_body(CI_DAG, fn_name);
    let rust_body = extract_fn_body(CI_AFFECTED_COMPONENTS_LIB, fn_name);
    assert_eq!(
        dag_prefix_literals_in_fn(dag_body),
        rust_prefix_literals_in_fn(rust_body),
        "{fn_name}: prefix literal sets must match between {CI_DAG_PATH} and ci_affected_components lib"
    );
    assert_eq!(
        dag_exact_path_literals_in_fn(dag_body),
        rust_exact_path_literals_in_fn(rust_body),
        "{fn_name}: exact-path literal sets must match between {CI_DAG_PATH} and ci_affected_components lib"
    );
}

fn assert_ci_dag_rust_mirror_behavioral_parity() {
    use ci_affected_components::ci_component_affected_from_changed_paths;
    for fixture in CI_AFFECTED_BEHAVIORAL_FIXTURES {
        let flags = ci_component_affected_from_changed_paths([fixture.path]);
        assert_eq!(flags.v2, fixture.v2, "path `{}`: v2 flag", fixture.path);
        assert_eq!(flags.v3, fixture.v3, "path `{}`: v3 flag", fixture.path);
        assert_eq!(flags.v4, fixture.v4, "path `{}`: v4 flag", fixture.path);
        assert_eq!(
            flags.workflow_policy, fixture.workflow_policy,
            "path `{}`: workflow_policy flag",
            fixture.path
        );
        assert_eq!(
            flags.release_distribution, fixture.release_distribution,
            "path `{}`: release_distribution flag",
            fixture.path
        );
    }
}

fn assert_ci_dag_rust_mirror_full_parity() {
    for fn_name in CI_CHANGED_PATH_AFFECTS_FNS {
        assert_ci_dag_rust_bucket_parity(fn_name);
    }
    assert_ci_dag_rust_mirror_behavioral_parity();
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
        "ci_select_ci_jobs_from_affected_set",
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
    assert_ci_dag_rust_bucket_parity("ci_changed_path_affects_workflow_policy");
}

#[test]
fn v4_workflow_ci_component_bucket_prefixes_align_rust_mirror() {
    for fn_name in [
        "ci_changed_path_affects_v2",
        "ci_changed_path_affects_v3",
        "ci_changed_path_affects_v4",
    ] {
        assert_ci_dag_rust_bucket_parity(fn_name);
    }
}

#[test]
fn v4_workflow_ci_release_distribution_exact_paths_align_rust_mirror() {
    assert_ci_dag_rust_bucket_parity("ci_changed_path_affects_release_distribution");
}

#[test]
fn v4_workflow_ci_m1_rust_emit_probe_modeled_and_bound_to_ci_yml() {
    // ci.dag ↔ Rust mirror parity (P2 consumer — set equality + behavioral fixtures; CI `--exact` step).
    assert_ci_dag_rust_mirror_full_parity();

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
        CI_DAG.contains("arch: ci_runner_arch_arm64"),
        "{CI_DAG_PATH}: fleet arch must be a typed Symbol field on SelfHostedRunnerPool"
    );
    assert!(
            !CI_DAG.contains("RunnerArch"),
            "{CI_DAG_PATH}: fleet arch uses Symbol field — no RunnerArch coproduct in Lens-CI entry compile"
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
    let m1_jobs = ci_affected_components::runner_pool::m1_probe_cargo_check_jobs();
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
fn v4_workflow_ci_bankruptcy_tier0_modeled_and_legacy_jobs_deleted() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    for name in ["CiBuildProfile", "CiSchedulePolicy"] {
        assert!(
            module.items.iter().any(|item| match item {
                SurfaceItem::TypeSum {
                    name: item_name, ..
                } => item_name == name,
                _ => false,
            }),
            "{CI_DAG_PATH}: must model bankruptcy Tier-0 enum `{name}`"
        );
    }
    for arm in [
        "| V2BootstrapCompileCommand",
        "| V3DeterminismCommand",
        "| V3SelfHostFixedPointCommand",
        "| V4T15SelfHostFixedPointCommand",
    ] {
        assert!(
            CI_DAG.contains(arm),
            "{CI_DAG_PATH}: CiCommand must include bankruptcy Tier-0 arm `{arm}`"
        );
    }
    assert!(
        CI_DAG.contains("fn ci_select_ci_jobs_from_affected_set("),
        "{CI_DAG_PATH}: must declare ci_select_ci_jobs_from_affected_set (I1 / A2)"
    );
    assert!(
        CI_DAG.contains("v3_determinism_execution"),
        "{CI_DAG_PATH}: ci_pipeline must include v3_determinism_execution"
    );
    assert!(
        CI_DAG.contains("v3_self_host_fixed_point_execution"),
        "{CI_DAG_PATH}: ci_pipeline must include v3_self_host_fixed_point_execution"
    );
    assert!(
        CI_DAG.contains("v4_t15_self_host_fixed_point_execution"),
        "{CI_DAG_PATH}: ci_pipeline must include v4_t15_self_host_fixed_point_execution (I7 / T-15)"
    );
    for legacy_job in ["  v2:", "  v3:", "  v4:", "  self_host_ratchet:"] {
        assert!(
            !CI_YML.contains(legacy_job),
            "{CI_YML_PATH}: bankruptcy B0 must delete legacy job `{legacy_job}`"
        );
    }
    assert!(
        CI_YML.contains("v3 determinism (Tier-0 I3)"),
        "{CI_YML_PATH}: Tier-0 I3 must run inside the ci harness job"
    );
    assert!(
        CI_YML.contains("v3 self-host fixed point (Tier-0 I4)"),
        "{CI_YML_PATH}: Tier-0 I4 must run inside the ci harness job"
    );
    let t15_step = workflow_step_block(CI_YML, T15_SELF_HOST_STEP_NAME);
    assert!(
        t15_step.contains(T15_SELF_HOST_HARNESS_TEST_FILTER),
        "{CI_YML_PATH}: `{T15_SELF_HOST_STEP_NAME}` must run the T-15 self-host fixed-point harness (I7)"
    );
    let binding_step = workflow_step_block(CI_YML, CI_MODEL_YAML_BINDING_STEP_NAME);
    assert!(
        binding_step.contains(BANKRUPTCY_TIER0_BINDING_TEST_FILTER),
        "{CI_YML_PATH}: `{CI_MODEL_YAML_BINDING_STEP_NAME}` must execute the bankruptcy D3 ratchet \
         (`{BANKRUPTCY_TIER0_BINDING_TEST_FILTER}` with --exact) so legacy job reintroduction fails CI"
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
