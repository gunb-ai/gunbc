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
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/affected_set_ci_runner.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/workflow/affected_set_ci_runner.dag";

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
        CI_DAG.contains("RerunNodeSetFailClosed { evidence: _ } => roster"),
        "{CI_DAG_PATH}: fail-closed must return full roster on RerunNodeSetFailClosed (AI-16/R1-7)"
    );
    assert!(
        CI_DAG.contains("test_claim_ci_selection_fail_closed(c: claim)"),
        "{CI_DAG_PATH}: DiagnosticClaim rows must bypass narrow filter via fail-closed guard"
    );
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
    let step_name = expr_string(record_body_field(live_signal, "step_name"));
    let script_path = expr_string(record_body_field(live_signal, "script_path"));
    let m1_step = workflow_step_block(CI_YML, step_name);
    assert!(
        m1_step.contains("if: needs.affected.outputs.v4 == 'true' || needs.affected.outputs.workflow_policy == 'true'"),
        "{CI_YML_PATH}: `{step_name}` must run for v4 and workflow-policy changes"
    );
    assert!(
        m1_step.contains("continue-on-error: true"),
        "{CI_YML_PATH}: `{step_name}` must stay non-blocking per modeled `non_blocking: true`"
    );
    assert!(
        m1_step.contains(&format!("run: bash {script_path}")),
        "{CI_YML_PATH}: `{step_name}` must invoke the modeled probe script"
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
