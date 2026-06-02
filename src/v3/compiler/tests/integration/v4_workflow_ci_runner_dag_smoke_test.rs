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
//! **TASKS.md** T-21 + T-24; bankruptcy B0/B1 Tier-0 binding smoke: `docs/design-ci-bankruptcy-rebuild.md` §4.1
//! Wave 1 §11.7.1 floor: `docs/planning/ci-required-surface-cut-2026-06-01.md` (`v4_workflow_ci_wave1_*`).
//! Affected-set component receipt promotion: `affected` is now a live fail-closed receipt; Wave 3
//! node-frontier TestClaim selection remains shadow until the whole-program Dag CI input lands.
//! Wave 3 §11.7.2 shadow receipt Phase 1: same doc (`v4_workflow_ci_wave3_*`; P5(b) receipt table).
//! (P5 same-path expansion — `_internal/INVARIANTS_OPS.md` → this file, PR #4101 / #4174 / #4214).
//! T-38 PR2 same-path assertion expansion: explicit P5 deferral to T-PB-B test sub-ratchet;
//! ROADMAP.md § "Milestone shape" row 4 ("Self-host fixed point") tracks hand-maintained file count -> 0.
//!
//! **INVARIANTS P5 — checkable receipt for this PR:** feature `affected-component-live-receipt`;
//! consumers `v4_workflow_ci_wave1_*` and `v4_workflow_ci_wave3_node_selection_still_shadow_*`.
//! Dissolve-on: A15 Shape-B/T-24 emitted `ci.yml` plus `.dag` TestClaim execution covers the
//! live component receipt and Wave 3 deferral without this hand-Rust parse harness.
//!
//! **INVARIANTS P5 — checkable receipt for PR #4251 (`infra_isolation` required-path):** feature
//! `infra-isolation-required-gate`; consumers `v4_workflow_ci_bankruptcy_tier0_discipline_off_required_ci_path`,
//! `v4_workflow_ci_wave1_*`, `v4_workflow_ci_wave3_node_selection_still_shadow_*`. SAME-PATH edit:
//! updates the existing `ci`-aggregator-needs assertions to the `[affected, ci_floor, infra_isolation]`
//! triple — no new test fn or authority surface (the modeled de-priv guard's coverage is the
//! byte-for-byte carrier structural-match, not a hand-Rust binding test; ROADMAP row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero`). Dissolve-on: same A15 Shape-B/T-24 lane as above.
//!
//! **Dissolution:** remove when `.dag` TestClaim execution covers these claims without
//! this hand-Rust parse harness (A15 Shape-B emitted `ci.yml` retires `v4_workflow_ci_bankruptcy_tier0_*`).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem, SurfaceLiteral, SurfaceRecordField};
use v3_compiler::tokenize_for_test;

const CI_DAG: &str = include_str!("../../../../v4/workflow/ci.dag");
const CI_DAG_PATH: &str = "src/v4/workflow/ci.dag";
const CI_YML: &str = include_str!("../../../../../.github/workflows/ci.yml");
const CI_YML_PATH: &str = ".github/workflows/ci.yml";
const CI_WORKFLOW_DAG: &str =
    include_str!("../../../../../dsl/gunbc/ci_github_actions_workflow.dag");
const CI_WORKFLOW_DAG_PATH: &str = "dsl/gunbc/ci_github_actions_workflow.dag";
const TESTCLAIM_CORPUS_EVAL_SCRIPT: &str =
    include_str!("../../../../../scripts/v4-testclaim-corpus-eval.sh");
const TESTCLAIM_CORPUS_EVAL_SCRIPT_PATH: &str = "scripts/v4-testclaim-corpus-eval.sh";
const M1_RUST_EMIT_PROBE_SCRIPT: &str =
    include_str!("../../../../../scripts/v4-m1-rust-emit-probe.sh");
const M1_BINDING_TEST_FILTER: &str =
    "v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_m1_rust_emit_probe_modeled_and_bound_to_ci_yml";
const BANKRUPTCY_TIER0_BINDING_TEST_FILTER: &str =
    "v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_bankruptcy_tier0_";
const CI_MODEL_YAML_BINDING_STEP_NAME: &str = "M1 v4 workflow CI model/YAML binding smoke";
const T15_SELF_HOST_STEP_NAME: &str = "T-15 self-host fixed-point harness (stage1==stage2)";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/affected_set_ci_runner.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/workflow/affected_set_ci_runner.dag";
const WAVE3_ROSTER_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/wave3_shadow_roster.dag");
const WAVE3_ROSTER_PATH: &str = "src/v4/test/claim/workflow/wave3_shadow_roster.dag";
const CI_AFFECTED_COMPONENTS_LIB: &str =
    include_str!("../../../../../tools/ci_affected_components/src/lib.rs");

const CI_CHANGED_PATH_AFFECTS_FNS: &[&str] = &[
    "ci_changed_path_affects_v2",
    "ci_changed_path_affects_v3",
    "ci_changed_path_affects_v4",
    "ci_changed_path_affects_testclaim_corpus",
    "ci_changed_path_affects_workflow_policy",
    "ci_changed_path_affects_release_distribution",
];

struct CiAffectedFixture {
    path: &'static str,
    v2: bool,
    v3: bool,
    v4: bool,
    testclaim_corpus: bool,
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
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "dsl/gunbc/ci.dag",
        v2: false,
        v3: true,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "dsl/std/node.dag",
        v2: false,
        v3: true,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "scripts/v4-m1-rust-emit-probe.sh",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/workflow/ci.dag",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: true,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/workflow/release.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "src/v4/test/claim/lens_affected_set/irt1_leaf_claim_suite.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/test/claim/workflow/affected_set_ci_runner.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/test/claim/manual/mvp1_rust_add_translate.dag",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/test/coercion_fold_int_rust_fixture.dag",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "install.sh",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "scripts/release-target-triples.sh",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "src/v4/install/install.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "Cargo.lock",
        v2: true,
        v3: true,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "Cargo.toml",
        v2: true,
        v3: true,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "docs/README.md",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
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

fn assert_ci_dag_rust_mirror_release_distribution_only_parity() {
    use ci_affected_components::{
        ci_component_affected_from_changed_paths, ci_release_distribution_only_from_changed_paths,
    };
    assert!(ci_release_distribution_only_from_changed_paths([
        "install.sh",
        "src/v4/install/install.dag",
    ]));
    assert!(!ci_release_distribution_only_from_changed_paths([
        "install.sh",
        "scripts/v4-phase1-nat-semiring-rung-gate.sh",
    ]));
    assert!(!ci_release_distribution_only_from_changed_paths([
        "install.sh",
        "src/v4/test/claim/workflow/affected_set_ci_runner.dag",
    ]));
    let mixed = ci_component_affected_from_changed_paths([
        "install.sh",
        "scripts/v4-phase1-nat-semiring-rung-gate.sh",
    ]);
    assert!(mixed.release_distribution);
    assert!(mixed.v4);
    assert!(!mixed.release_distribution_only);
}

fn assert_ci_dag_rust_mirror_behavioral_parity() {
    use ci_affected_components::ci_component_affected_from_changed_paths;
    for fixture in CI_AFFECTED_BEHAVIORAL_FIXTURES {
        let flags = ci_component_affected_from_changed_paths([fixture.path]);
        assert_eq!(flags.v2, fixture.v2, "path `{}`: v2 flag", fixture.path);
        assert_eq!(flags.v3, fixture.v3, "path `{}`: v3 flag", fixture.path);
        assert_eq!(flags.v4, fixture.v4, "path `{}`: v4 flag", fixture.path);
        assert_eq!(
            flags.testclaim_corpus, fixture.testclaim_corpus,
            "path `{}`: testclaim_corpus flag",
            fixture.path
        );
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
    assert_ci_dag_rust_mirror_release_distribution_only_parity();
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
    let fields = match body {
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => fields,
        other => panic!("expected record body, got {other:?}"),
    };
    record_field_from_fields(fields, field_name)
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

fn record_field_from_fields<'a>(
    fields: &'a [SurfaceRecordField],
    field_name: &str,
) -> &'a SurfaceExpr {
    fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|SurfaceRecordField { value, .. }| value)
        .unwrap_or_else(|| panic!("record body missing `{field_name}` field"))
}

fn strict_env_binding(expr: &SurfaceExpr) -> Option<(&str, &str)> {
    match expr {
        SurfaceExpr::Var { name, .. } if name == "NoStrictEnvBinding" => None,
        SurfaceExpr::VariantRecord { target, fields, .. } if target == "StrictEnvBinding" => {
            Some((
                expr_string(record_field_from_fields(fields, "var")),
                expr_string(record_field_from_fields(fields, "value")),
            ))
        }
        other => panic!("expected strict env binding expr, got {other:?}"),
    }
}

/// True when `job_id` is a deleted bankruptcy legacy *workflow job* block (not `affected` outputs).
fn ci_yml_has_deleted_legacy_top_level_job(workflow_yml: &str, job_id: &str) -> bool {
    // Legacy jobs used `if:` before `needs:` / `runs-on:` (see main pre-bankruptcy ci.yml).
    workflow_yml.contains(&format!("\n  {job_id}:\n    if:"))
        || workflow_yml.contains(&format!("\n  {job_id}:\n    needs:"))
        || workflow_yml.contains(&format!("\n  {job_id}:\n    runs-on:"))
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

fn workflow_dag_job_block<'a>(workflow_dag: &'a str, job_id: &str) -> &'a str {
    let marker = format!("id: \"{job_id}\"");
    let id_start = workflow_dag
        .find(&marker)
        .unwrap_or_else(|| panic!("{CI_WORKFLOW_DAG_PATH}: missing `{job_id}` job"));
    let block_start = workflow_dag[..id_start]
        .rfind('{')
        .unwrap_or_else(|| panic!("{CI_WORKFLOW_DAG_PATH}: malformed `{job_id}` job"));
    let mut depth = 0usize;
    for (offset, ch) in workflow_dag[block_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth
                    .checked_sub(1)
                    .unwrap_or_else(|| panic!("{CI_WORKFLOW_DAG_PATH}: malformed job braces"));
                if depth == 0 {
                    let end = block_start + offset + ch.len_utf8();
                    return &workflow_dag[block_start..end];
                }
            }
            _ => {}
        }
    }
    panic!("{CI_WORKFLOW_DAG_PATH}: unterminated `{job_id}` job")
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
        CI_DAG.contains("data m1_probe_cargo_check_jobs_ceiling: Int = 64"),
        "{CI_DAG_PATH}: M1 cargo parallelism ceiling must be an explicit operator constant in Wave-0"
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
        "ci_changed_path_affects_testclaim_corpus",
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
    let live_step = record_body_field(live_signal, "step");
    let binding_smoke_step_name =
        expr_string(record_body_field(live_step, "binding_smoke_step_name"));
    let step_name = expr_string(record_body_field(live_step, "step_name"));
    let script_path = expr_string(record_body_field(live_step, "script_path"));
    let non_blocking = expr_bool(record_body_field(live_step, "non_blocking"));
    let timeout_minutes = expr_int(record_body_field(live_step, "timeout_minutes"));
    let (strict_env_var, strict_env_value) =
        strict_env_binding(record_body_field(live_step, "strict_env_binding"))
            .unwrap_or_else(|| panic!("{CI_DAG_PATH}: M1 probe must model a strict env binding"));
    let rust_emit_probe_policy = record_body_field(live_signal, "rust_emit_probe_policy");
    let emit_preconditions_block_required_path = expr_bool(record_body_field(
        rust_emit_probe_policy,
        "emit_preconditions_block_required_path",
    ));
    let rustc_residuals_block_required_path = expr_bool(record_body_field(
        rust_emit_probe_policy,
        "rustc_residuals_block_required_path",
    ));
    let binding_smoke_step = workflow_step_block(CI_YML, binding_smoke_step_name);
    assert!(
        binding_smoke_step.contains(&format!(
            "cargo test -p v3-compiler --test integration {M1_BINDING_TEST_FILTER} -- --exact --quiet"
        )),
        "{CI_YML_PATH}: `{binding_smoke_step_name}` must execute the M1 model/YAML binding receipt (gunbc#846 zero-test-filter bypass)"
    );
    let m1_step = workflow_step_block(CI_YML, step_name);
    assert!(
        !m1_step.contains("needs.affected.outputs"),
        "{CI_YML_PATH}: Wave 1 §11.7.1 — `{step_name}` runs unconditionally on the safety floor (no component `if:`)"
    );
    assert!(
        m1_step.contains(&format!("{strict_env_var}: \"{strict_env_value}\"")),
        "{CI_YML_PATH}: `{step_name}` strictness env must come from {CI_DAG_PATH}:m1_ci_live_workflow_signal"
    );
    assert_eq!(
        strict_env_value == "1",
        rustc_residuals_block_required_path,
        "{CI_DAG_PATH}: strictness env and rustc residual required-path policy must agree"
    );
    assert!(
        emit_preconditions_block_required_path,
        "{CI_DAG_PATH}: required M1 probe must fail closed on missing compiler, v2 emit failure, and skipped cargo-check preconditions"
    );
    assert!(
        CI_DAG.contains(
            "data m1_rust_emit_probe_shared_dag_out_env: String = \"V4_M1_DAG_EMIT_OUT\""
        ) && CI_DAG.contains(
            "data m1_rust_emit_probe_shared_dag_log_env: String = \"V4_M1_DAG_EMIT_LOG\""
        ) && CI_DAG.contains(
            "data m1_rust_dag_emit_parity_receipt_test: String = \"cargo test -p v2-compiler-tests pipeline::dag_emit_from_resolved_matches_compile_sources_for_v4_slice -- --exact --quiet\""
        ) && CI_DAG.contains("data v4_bootstrap_reuse_log_env: String = \"V4_BOOTSTRAP_REUSE_LOG\""),
        "{CI_DAG_PATH}: shared M1/bootstrap closure env names must be modeled"
    );
    assert!(
        m1_step.contains("V4_M1_DAG_EMIT_OUT:") && m1_step.contains("V4_M1_DAG_EMIT_LOG:"),
        "{CI_YML_PATH}: `{step_name}` must emit the DAG artifact from the shared rust+dag closure"
    );
    assert!(
        CI_DAG.contains(
            "M1RustEmitProbeCommand =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true,\n        testclaim_corpus: true,\n        workflow_policy: true,\n        release_distribution: true\n      )"
        ),
        "{CI_DAG_PATH}: M1RustEmitProbeCommand mask must match ci.yml step if (I8 / T-22 upstream)"
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
    // Governor ceiling is the ONLY M1 parallelism constant — no static fallback. Actual jobs are
    // jobserver-coupled (inherited MAKEFLAGS on GHA / ctrl-build in session containers) and pared
    // below this ceiling by the host token pool.
    assert!(
        CI_DAG.contains("data m1_probe_cargo_check_jobs_ceiling: Int = 64"),
        "{CI_DAG_PATH}: M1 governor ceiling must be an explicit operator constant"
    );
    let m1_ceiling = ci_affected_components::runner_pool::m1_probe_cargo_check_jobs_ceiling();
    assert_eq!(
        m1_ceiling, 64,
        "Rust transport mirror of m1_probe_cargo_check_jobs_ceiling must match ci.dag operator constant"
    );
    assert!(
        m1_step.contains(&format!("V4_M1_CARGO_CHECK_JOBS_CEILING: \"{m1_ceiling}\"")),
        "{CI_YML_PATH}: M1 step must project modeled governor ceiling (CTRL_BUILD_DYNAMIC_JOBS_MAX)"
    );
    assert!(
        !m1_step.contains("V4_M1_CARGO_CHECK_JOBS:"),
        "{CI_YML_PATH}: M1 step must NOT project a static cargo-check fallback (fail-closed, no fallback)"
    );
    // The probe must route the emitted-tree check through the host compute governor, not a hand cap.
    assert!(
        CI_DAG.contains("feature:elastic-compute-fabric")
            && CI_DAG.contains("dsl/std/compute_fabric.dag"),
        "{CI_DAG_PATH}: M1 parallelism note must cite the compute_fabric dissolve-on-arrival authority"
    );
    // The probe must run jobserver-coupled: inherited MAKEFLAGS (GHA runner unit) or ctrl-build
    // (session containers), and it still understands the ctrl-build governor for the latter.
    assert!(
        M1_RUST_EMIT_PROBE_SCRIPT.contains("jobserver-auth")
            && M1_RUST_EMIT_PROBE_SCRIPT.contains("ctrl-build")
            && M1_RUST_EMIT_PROBE_SCRIPT.contains("CTRL_BUILD_DYNAMIC_JOBS_MAX"),
        "scripts/v4-m1-rust-emit-probe.sh: emitted-tree check must couple to the host jobserver (MAKEFLAGS or ctrl-build)"
    );
    // No fallback: the probe must fail closed when NEITHER coupling source is present (operator policy).
    assert!(
        M1_RUST_EMIT_PROBE_SCRIPT.contains("requires a host jobserver coupling"),
        "scripts/v4-m1-rust-emit-probe.sh: probe must fail closed when no jobserver coupling is present"
    );
    let bootstrap_step =
        workflow_step_block(CI_YML, "v2 -> v4 bootstrap compile (fail-closed full)");
    let parity_step = workflow_step_block(
        CI_YML,
        "v2 DAG emit parity receipt (required before bootstrap reuse)",
    );
    assert!(
        parity_step.contains(
            "cargo test -p v2-compiler-tests pipeline::dag_emit_from_resolved_matches_compile_sources_for_v4_slice -- --exact --quiet"
        ),
        "{CI_YML_PATH}: bootstrap reuse must be preceded by the v2 DAG emit parity receipt"
    );
    assert!(
        CI_YML.find("v2 DAG emit parity receipt (required before bootstrap reuse)")
            < CI_YML.find("v2 -> v4 bootstrap compile (fail-closed full)"),
        "{CI_YML_PATH}: parity receipt must run before bootstrap reuse"
    );
    assert!(
        bootstrap_step.contains("V4_BOOTSTRAP_REUSE_LOG:"),
        "{CI_YML_PATH}: bootstrap step must validate the shared M1 DAG artifact instead of recompiling src/v4"
    );
}

// P5(b) receipt: `ROADMAP.md` § **Nine lanes** **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
// same-path expansion in this file (SG-0 delta 0 — path already in `EXPECTED_HAND_AUTHORED_TEST`;
// #4091 §1.2 four-compile collapse modeled UpstreamUpsert pin for m1 / lens-CI / phase1 rung gate).

/// #4091 §1.2 four-compile collapse: the modeled ci_pipeline jobs that re-ran a full src/v4
/// 332-source closure compile must consume `v2_compile_src_v4`'s resolved closure via
/// UpstreamUpsert (front-end shared once; each job re-emits its own target) instead of each
/// independently re-running parse/lower/infer. Pins the artifact-consumption edge for m1,
/// lens-CI, and the phase1 rung gate. (bootstrap-dag is split out: its dissolution target is
/// `ModelMissingSubstrate { what: v4_bootstrap_dag_emit_ci_job }` — substrate, follow-up.)
#[test]
fn v4_workflow_ci_four_compile_collapse_jobs_consume_v2_compile_src_v4_closure() {
    // Slice the `data <symbol>...` definition up to the next top-level `\ndata ` so the
    // upstream-edge assertion is scoped to each job's own inputs block.
    let block_for = |symbol: &str| -> &str {
        let start = CI_DAG
            .find(&format!("data {symbol}"))
            .unwrap_or_else(|| panic!("{CI_DAG_PATH}: missing `data {symbol}`"));
        let rest = &CI_DAG[start..];
        let end = rest[1..]
            .find("\ndata ")
            .map(|i| i + 1)
            .unwrap_or(rest.len());
        &rest[..end]
    };
    let upstream_edge = "ci_upsert_upstream_job_input(job: v2_compile_src_v4)";
    // Each formerly-recompiling job declares the upstream artifact edge on its CiUpsertStep
    // inputs, mirroring the v4_t15 / testclaim_corpus_eval pattern at ci.dag:807.
    assert!(
        block_for("ci_upsert_m1_rust_emit_probe_execution_inputs").contains(upstream_edge),
        "{CI_DAG_PATH}: M1 rust emit probe must consume v2_compile_src_v4's resolved closure via UpstreamUpsert (#4091 §1.2 four-compile collapse)"
    );
    assert!(
        block_for("ci_upsert_phase1_nat_semiring_rung_gate_execution_inputs")
            .contains(upstream_edge),
        "{CI_DAG_PATH}: phase1 rung gate must consume v2_compile_src_v4's resolved closure via UpstreamUpsert (#4091 §1.2 four-compile collapse)"
    );
    // lens-CI inputs are computed (list_snoc_item over per-lens refs); the upstream edge is
    // appended in the data initializer.
    assert!(
        block_for("ci_upsert_lens_ci_registry_execution_inputs").contains(upstream_edge),
        "{CI_DAG_PATH}: lens-CI registry execution must consume v2_compile_src_v4's resolved closure via UpstreamUpsert (#4091 §1.2 four-compile collapse)"
    );
    // The three jobs already declare v2_compile_src_v4 in `needs` (ordering); the collapse turns
    // those ordering deps into real artifact-consumption edges (above). Spot-check needs stay.
    assert!(
        CI_DAG.contains("id: m1_rust_emit_probe_execution,")
            && CI_DAG.contains("id: lens_ci_registry_execution,")
            && CI_DAG.contains("id: phase1_nat_semiring_rung_gate_execution,"),
        "{CI_DAG_PATH}: collapse must not remove the three modeled closure-compile jobs"
    );
}

#[test]
fn v4_workflow_ci_testclaim_corpus_eval_modeled_and_bound_to_ci_yml() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        CI_DAG.contains(
            "data testclaim_corpus_eval_ci_live_workflow_signal: CiLiveWorkflowStepSignal"
        ),
        "{CI_DAG_PATH}: must model live-workflow binding for testclaim corpus eval"
    );
    assert!(
        CI_DAG.contains("ci_upsert_testclaim_corpus_eval_upstream_inputs")
            &&         CI_DAG.contains("ci_upsert_upstream_job_input(job: m1_rust_emit_probe_execution)"),
        "{CI_DAG_PATH}: testclaim corpus eval execution must consume M1 rust emit via UpstreamUpsert (#4091 §1.2)"
    );
    assert!(
        CI_DAG.contains("needs: [v2_compile_src_v4, m1_rust_emit_probe_execution]"),
        "{CI_DAG_PATH}: testclaim corpus eval must declare M1 in needs for selector needs-closure (I8)"
    );
    let live_signal = data_body(&module, "testclaim_corpus_eval_ci_live_workflow_signal");
    let step_name = expr_string(record_body_field(live_signal, "step_name"));
    let script_path = expr_string(record_body_field(live_signal, "script_path"));
    let non_blocking = expr_bool(record_body_field(live_signal, "non_blocking"));
    let timeout_minutes = expr_int(record_body_field(live_signal, "timeout_minutes"));
    assert!(
        !CI_YML.contains(&format!("- name: {step_name}")),
        "{CI_YML_PATH}: Wave 1 §11.7.1 — `{step_name}` demoted from required path (modeled in ci.dag; Class C)"
    );
    assert!(
        CI_DAG.contains(&format!("script_path: \"{script_path}\"")),
        "{CI_DAG_PATH}: T-22 corpus eval host transport remains modeled as `{script_path}`"
    );
    let _ = (non_blocking, timeout_minutes);
    assert!(
        !CI_YML.contains("v4-testclaim-corpus-gate.sh"),
        "{CI_YML_PATH}: shell bridge script must be absent from workflow"
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
        CI_YML.contains("bash scripts/v4-bootstrap-viability.sh"),
        "{CI_YML_PATH}: Wave 1 floor runs bootstrap viability directly (no advisory two-step gate)"
    );
    assert!(
        CI_DAG.contains("workflow_local_bootstrap_dag_upstream")
            && CI_DAG.contains("v4_bootstrap_dag_emit_ci_job"),
        "{CI_DAG_PATH}: testclaim corpus eval must document workflow-local bootstrap via ci_always_run_carveouts (P5 §1.2)"
    );
    assert!(
        !CI_YML.contains("T-22 TestClaim corpus structural bridge"),
        "{CI_YML_PATH}: retired bridge step name must not appear anywhere in workflow YAML"
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
fn v4_workflow_ci_bankruptcy_tier0_types_and_command_arms_modeled() {
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
        CI_DAG.contains("fn ci_select_ci_jobs_from_affected_set(\n  pipeline: CiPipeline,"),
        "{CI_DAG_PATH}: job selector must take well-formed CiPipeline (not bare job list)"
    );
    for step_id in [
        "v3_determinism_execution",
        "v3_self_host_fixed_point_execution",
        "v4_t15_self_host_fixed_point_execution",
    ] {
        assert!(
            CI_DAG.contains(step_id),
            "{CI_DAG_PATH}: ci_pipeline must include bankruptcy Tier-0 step `{step_id}`"
        );
    }
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_schedule_policy_carries_disposition() {
    assert!(
        CI_DAG.contains("feature:ci-bankruptcy-schedule-policy")
            && CI_DAG.contains("fn ci_job_scheduled_by_policy("),
        "{CI_DAG_PATH}: schedule-policy Bool dispatch must carry Practice-10 disposition"
    );
    assert!(
        CI_DAG.contains("feature:ci-bankruptcy-component-fail-closed")
            && CI_DAG.contains("fn ci_component_affected_is_fail_closed("),
        "{CI_DAG_PATH}: component fail-closed Bool classifier must carry Practice-10 disposition"
    );
    assert!(
        CI_DAG.contains("feature:ci-bankruptcy-tier0-gha-step-if")
            && CI_DAG.contains("ci_v3_self_host_fixed_point_ci_live_workflow_binding")
            && CI_DAG.contains("ci_v4_bootstrap_gate_result_skip_guard_if"),
        "{CI_DAG_PATH}: Tier-0 GHA if bridge carriers must carry Practice-10 disposition"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_legacy_top_level_jobs_deleted() {
    for legacy_job in ["v2", "v3", "v4", "self_host_ratchet"] {
        assert!(
            !ci_yml_has_deleted_legacy_top_level_job(CI_YML, legacy_job),
            "{CI_YML_PATH}: bankruptcy B0 must delete legacy top-level job `{legacy_job}`"
        );
    }
    assert!(
        !CI_YML.contains("v3 determinism (Tier-0 I3)")
            && !CI_YML.contains("v3 self-host fixed point (Tier-0 I4)"),
        "{CI_YML_PATH}: Wave 1 §11.7.3 — v3 integration permanently deleted from required CI"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_ci_integration_job_if_includes_release_distribution() {
    assert!(
        !CI_YML.contains("  ci_integration:"),
        "{CI_YML_PATH}: Wave 1 — `ci_integration` job dissolved into `ci_floor`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_v3_bucket_includes_workspace_deps() {
    let v3_body = extract_fn_body(CI_DAG, "ci_changed_path_affects_v3");
    for path in ["Cargo.toml", "Cargo.lock"] {
        assert!(
            v3_body.contains(&format!("changed == \"{path}\"")),
            "{CI_DAG_PATH}: ci_changed_path_affects_v3 must include `{path}` (V3DeterminismCommand Upsert inputs at ci_upsert_v3_determinism_execution_inputs)"
        );
    }
    assert!(
        CI_DAG.contains("segment == \"dsl/**\""),
        "{CI_DAG_PATH}: ci_glob_segment_matches_changed_path must support dsl/** Upsert segment (P2 parity with ci_changed_path_affects_v3 dsl/ prefix)"
    );
    let i3_inputs = CI_DAG
        .split("data ci_upsert_v3_determinism_execution_inputs:")
        .nth(1)
        .and_then(|rest| {
            rest.split("fn ci_upsert_v3_determinism_execution_mk")
                .next()
        })
        .unwrap_or_else(|| {
            panic!("{CI_DAG_PATH}: missing data ci_upsert_v3_determinism_execution_inputs")
        });
    for segment in ["src/v3/**", "dsl/**", "Cargo.toml", "Cargo.lock"] {
        assert!(
            i3_inputs.contains(&format!("segment: \"{segment}\"")),
            "{CI_DAG_PATH}: ci_upsert_v3_determinism_execution_inputs must include `{segment}` (P2 parity with affected.v3)"
        );
    }
    assert_ci_dag_rust_bucket_parity("ci_changed_path_affects_v3");
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_discipline_off_required_ci_path() {
    assert!(
        CI_YML.contains("needs: [affected, ci_floor, infra_isolation]"),
        "{CI_YML_PATH}: branch-protection `ci` aggregator must need live `affected` receipt, `ci_floor`, and the `infra_isolation` de-priv guard"
    );
    assert!(
        CI_YML.contains("needs.affected.result"),
        "{CI_YML_PATH}: `affected` must be checked by the fail-closed aggregator"
    );
    assert!(
        !CI_YML.contains("needs.discipline.result") && !CI_YML.contains("  discipline:"),
        "{CI_YML_PATH}: discipline job deleted per §11.7.3"
    );
    let ci_floor_block = CI_YML
        .split("  ci_floor:")
        .nth(1)
        .and_then(|rest| rest.split("\n  ci:").next())
        .unwrap_or("");
    assert!(
        !ci_floor_block.contains("needs: [affected]"),
        "{CI_YML_PATH}: `ci_floor` must not depend on `affected`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_i4_if_matches_live_workflow_binding() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    let i4_binding = data_body(
        &module,
        "ci_v3_self_host_fixed_point_ci_live_workflow_binding",
    );
    let i4_step_name = expr_string(record_body_field(i4_binding, "step_name"));
    assert!(
        !CI_YML.contains(&format!("- name: {i4_step_name}")),
        "{CI_YML_PATH}: Wave 1 — I4 v3 self-host step not on required path (binding remains in ci.dag)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_corpus_eval_uses_one_canonical_ci_job_row() {
    assert!(
        CI_DAG.contains("data ci_job_testclaim_corpus_eval_execution_row: CiJob = ci_job_testclaim_corpus_eval_execution_mk()")
            && CI_DAG.contains("create: ci_job_projection_node(j: ci_job_testclaim_corpus_eval_execution_row)")
            && CI_DAG.contains("ci_job_testclaim_corpus_eval_execution_row,"),
        "{CI_DAG_PATH}: corpus eval must use one canonical CiJob row for pipeline + Upsert create (P2)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_t15_schedule_matches_ci_yml() {
    assert!(
        !CI_YML.contains(T15_SELF_HOST_STEP_NAME),
        "{CI_YML_PATH}: Wave 1 — T-15 harness demoted from required path"
    );
    assert!(
        CI_DAG.contains(
            "V4T15SelfHostFixedPointCommand =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true,\n        testclaim_corpus: false,\n        workflow_policy: true,\n        release_distribution: false\n      )"
        ),
        "{CI_DAG_PATH}: V4T15SelfHostFixedPointCommand mask remains modeled"
    );
    assert!(
        CI_DAG.contains("fn ci_v4_t15_scheduled(affected: CiComponentAffected, schedule: CiSchedulePolicy) -> Bool"),
        "{CI_DAG_PATH}: T-15 must model main-push schedule authority"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_ci_v4_job_if_matches_generated_workflow() {
    assert!(
        !CI_YML.contains("  ci_v4:"),
        "{CI_YML_PATH}: Wave 1 — `ci_v4` job dissolved into `ci_floor`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_release_distribution_mask_axes_modeled() {
    assert!(
        CI_DAG.contains("release_distribution && affected.release_distribution"),
        "{CI_DAG_PATH}: ci_component_mask_intersects must include release_distribution axis"
    );
    assert!(
        CI_DAG.contains("testclaim_corpus && affected.testclaim_corpus"),
        "{CI_DAG_PATH}: ci_component_mask_intersects must include testclaim_corpus axis (I8 / IRT-1)"
    );
    let fail_closed_body = extract_fn_body(CI_DAG, "ci_component_affected_is_fail_closed");
    for axis in [
        "affected.v2",
        "affected.v3",
        "affected.v4",
        "affected.testclaim_corpus",
        "affected.workflow_policy",
        "affected.release_distribution",
    ] {
        assert!(
            fail_closed_body.contains(axis),
            "{CI_DAG_PATH}: ci_component_affected_is_fail_closed must require axis `{axis}`"
        );
    }
    assert!(
        CI_DAG.contains(
            "BootstrapStageCompile { produces: _ } =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true"
        ),
        "{CI_DAG_PATH}: BootstrapStageCompile mask must not select on v2-only (I2 is ci_integration; gunbc build is ci_v4 v4-axis)"
    );
    assert!(
        CI_DAG.contains("fn ci_job_component_mask_row(")
            && CI_DAG.contains("release_distribution_only: false"),
        "{CI_DAG_PATH}: job masks must populate release_distribution_only (P2 carrier completeness)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_upsert_slice_registers_tier0_jobs() {
    assert!(
        CI_DAG.contains("data ci_upsert_steps_bankruptcy_tier0_slice_step_ids:")
            && CI_DAG.contains("ci_upsert_v2_bootstrap_smoke_execution")
            && CI_DAG.contains("ci_upsert_v3_determinism_execution")
            && CI_DAG.contains("ci_upsert_v3_self_host_fixed_point_execution")
            && CI_DAG.contains("ci_upsert_v4_t15_self_host_fixed_point_execution"),
        "{CI_DAG_PATH}: bankruptcy Tier-0 jobs must register in ci_upsert_steps + full-in-scope slice"
    );
    assert!(
        CI_DAG.contains("ci_job_v2_bootstrap_smoke_execution_row")
            && CI_DAG.contains("ci_job_v3_determinism_execution_row")
            && CI_DAG.contains("ci_job_v3_self_host_fixed_point_execution_row")
            && CI_DAG.contains("ci_job_v4_t15_self_host_fixed_point_execution_row"),
        "{CI_DAG_PATH}: Tier-0 CiJob rows must be canonical (pipeline + Upsert create)"
    );
    let v2_i2_inputs = CI_DAG
        .split("data ci_upsert_v2_bootstrap_smoke_execution_inputs:")
        .nth(1)
        .and_then(|rest| {
            rest.split("fn ci_upsert_v2_bootstrap_smoke_execution_mk")
                .next()
        })
        .unwrap_or_else(|| {
            panic!("{CI_DAG_PATH}: missing data ci_upsert_v2_bootstrap_smoke_execution_inputs")
        });
    for segment in ["src/v2/**", "Cargo.toml", "Cargo.lock"] {
        assert!(
            v2_i2_inputs.contains(&format!("segment: \"{segment}\"")),
            "{CI_DAG_PATH}: ci_upsert_v2_bootstrap_smoke_execution_inputs must include `{segment}` (P2 parity with ci_changed_path_affects_v2 / I2 selection)"
        );
    }
    assert!(
        !v2_i2_inputs.contains("segment: \"scripts/**\""),
        "{CI_DAG_PATH}: I2 Upsert must not declare scripts/** — script paths route via v4/workflow_policy buckets, not ci_changed_path_affects_v2 (design-ci-bankruptcy-rebuild.md I2 frontier v2/compiler/**)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_v2_build_step_includes_main_push() {
    let v2_build = workflow_step_block(CI_YML, "Build v2 compiler (v4 floor)");
    assert!(
        !v2_build.contains("needs.affected.outputs"),
        "{CI_YML_PATH}: Wave 1 — v2 build runs unconditionally in `ci_floor`"
    );
    let v2_bin_cache = workflow_step_block(CI_YML, "Cache gunbc binary (v4 floor)");
    assert!(
        !v2_bin_cache.contains("needs.affected.outputs"),
        "{CI_YML_PATH}: Wave 1 — gunbc cache runs unconditionally in `ci_floor`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_needs_closure_is_bounded_and_skips_unresolved() {
    assert!(
        CI_DAG.contains("fn ci_select_ci_jobs_needs_closure_pass("),
        "{CI_DAG_PATH}: needs closure must be bounded (P4) — not unbounded recursion on unresolved needs"
    );
    assert!(
        CI_DAG.contains(
            "ci_symbol_resolves(s: n, jobs: jobs) && ci_symbol_not_in_ids(s: n, ids: selected_ids)"
        ),
        "{CI_DAG_PATH}: needs closure must ignore unresolved need symbols"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_lens_ci_mask_matches_ci_yml() {
    assert!(
        CI_DAG.contains(
            "LensCiCommand { required_lenses: _ } =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true,\n        testclaim_corpus: false,\n        workflow_policy: true,\n        release_distribution: false\n      )"
        ),
        "{CI_DAG_PATH}: LensCiCommand mask must match ci.yml step if (v4|workflow_policy; no v3 — I1 parity)"
    );
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    let live_signal = data_body(&module, "lens_ci_live_workflow_signal");
    let smoke_step_name = expr_string(record_body_field(live_signal, "smoke_step_name"));
    let semantic_step_name = expr_string(record_body_field(live_signal, "semantic_step_name"));
    assert!(
        !CI_YML.contains(&format!("- name: {smoke_step_name}")),
        "{CI_YML_PATH}: Wave 1 — `{smoke_step_name}` demoted from required path"
    );
    assert!(
        !CI_YML.contains(&format!("- name: {semantic_step_name}")),
        "{CI_YML_PATH}: Wave 1 — `{semantic_step_name}` demoted from required path"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_binding_step_matches_generated_workflow() {
    let binding_step = workflow_step_block(CI_YML, CI_MODEL_YAML_BINDING_STEP_NAME);
    assert!(
        binding_step.contains(&format!(
            "cargo test -p v3-compiler --test integration {BANKRUPTCY_TIER0_BINDING_TEST_FILTER} -- --quiet"
        )),
        "{CI_YML_PATH}: binding step must run bankruptcy D3 prefix filter on the Wave 1 floor"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_d3_ratchet_invoked_from_ci_yml_binding_step() {
    let binding_step = workflow_step_block(CI_YML, CI_MODEL_YAML_BINDING_STEP_NAME);
    assert!(
        binding_step.contains(&format!(
            "cargo test -p v3-compiler --test integration {M1_BINDING_TEST_FILTER} -- --exact --quiet"
        )),
        "{CI_YML_PATH}: `{CI_MODEL_YAML_BINDING_STEP_NAME}` must run M1 binding with one TESTNAME per cargo invocation"
    );
    assert!(
        binding_step.contains(&format!(
            "cargo test -p v3-compiler --test integration {BANKRUPTCY_TIER0_BINDING_TEST_FILTER} -- --quiet"
        )),
        "{CI_YML_PATH}: `{CI_MODEL_YAML_BINDING_STEP_NAME}` must run bankruptcy D3 ratchet tests (prefix filter, one claim per test)"
    );
    assert!(
        binding_step.contains("v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_wave1_"),
        "{CI_YML_PATH}: binding step must run Wave 1 floor prefix filter"
    );
}

#[test]
fn v4_workflow_ci_wave1_safety_floor_ci_yml_shape() {
    assert!(
        CI_YML.contains("Wave 1 §11.7.1 safety floor"),
        "{CI_YML_PATH}: must declare Wave 1 §11.7.1 safety floor"
    );
    assert!(
        CI_YML.contains("  ci_floor:"),
        "{CI_YML_PATH}: must define `ci_floor` job"
    );
    assert!(
        !CI_YML.contains("  ci_integration:") && !CI_YML.contains("  ci_v4:"),
        "{CI_YML_PATH}: legacy parallel lanes dissolved"
    );
    assert!(
        CI_YML.contains("needs: [affected, ci_floor, infra_isolation]"),
        "{CI_YML_PATH}: `ci` aggregator must depend on live affected receipt, `ci_floor`, and the `infra_isolation` de-priv guard"
    );
    assert!(
        CI_YML.contains("  affected:") && !CI_YML.contains("  affected:\n    if: github.event.pull_request.draft != true\n    continue-on-error: true"),
        "{CI_YML_PATH}: component `affected` receipt must be live, not continue-on-error shadow"
    );
    for forbidden in [
        "check-pr-sg0-net-shrink-discipline.sh",
        "determinism_test",
        "self_host_fixed_point",
        "v4-testclaim-corpus-eval.sh",
        "v4-mvp1-e2e-gate.sh",
        "v4-phase1-nat-semiring-rung-gate.sh",
    ] {
        assert!(
            !CI_YML.contains(forbidden),
            "{CI_YML_PATH}: Wave 1 cut — must not invoke `{forbidden}` on required path"
        );
    }
}

#[test]
fn v4_workflow_ci_wave1_generated_workflow_dag_matches_ci_yml_shape() {
    assert!(
        CI_WORKFLOW_DAG.contains("id: \"ci_floor\""),
        "{CI_WORKFLOW_DAG_PATH}: regen artifact must model `ci_floor`"
    );
    assert!(
        CI_WORKFLOW_DAG.contains("id: \"ci\"")
            && CI_WORKFLOW_DAG.contains("needs: [\"affected\", \"ci_floor\", \"infra_isolation\"]"),
        "{CI_WORKFLOW_DAG_PATH}: `ci` job must need live `affected` receipt, `ci_floor`, and the `infra_isolation` de-priv guard"
    );
    let affected_job = workflow_dag_job_block(CI_WORKFLOW_DAG, "affected");
    assert!(
        affected_job.contains("continue_on_error: false"),
        "{CI_WORKFLOW_DAG_PATH}: component `affected` receipt must be live"
    );
    assert!(
        !CI_WORKFLOW_DAG.contains("id: \"ci_integration\"")
            && !CI_WORKFLOW_DAG.contains("id: \"ci_v4\""),
        "{CI_WORKFLOW_DAG_PATH}: legacy parallel lanes dissolved in regen artifact"
    );
}

#[test]
fn v4_workflow_ci_wave1_no_new_shell_ratchet_wired() {
    let ratchet_step = workflow_step_block(CI_YML, "no-new-shell ratchet (required CI path)");
    assert!(
        ratchet_step.contains("check-ci-no-new-shell.sh"),
        "{CI_YML_PATH}: gate 5 must invoke no-new-shell ratchet"
    );
}

// P5(b) receipt: `docs/planning/ci-required-surface-cut-2026-06-01.md` § P5(b) `v4_workflow_ci_wave3_*`
// (SG-0 delta 0; ROADMAP T-PB-B; live emit deferred `node://adhoc-331899f9-19a`).

#[test]
fn v4_workflow_ci_wave3_roster_dag_tokenizes_and_parses() {
    let _module = parse_module(WAVE3_ROSTER_DAG, WAVE3_ROSTER_PATH);
}

#[test]
fn v4_workflow_ci_wave3_ci_dag_extension_and_fixture_receipt() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        !CI_DAG.contains("CiWave3ShadowReceipt"),
        "{CI_DAG_PATH}: Wave 3 must extend CiSelectionReceipt — no parallel receipt type"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "eval"],
            "test_claim_claim_hash_digest"
        ),
        "{CI_DAG_PATH}: claim_projection_hash must import IRT-4 `test_claim_claim_hash_digest` from v4.compiler.eval"
    );
    assert!(
        import_includes_name(&module, &["v4", "lens", "testgen"], "Generator")
            && import_includes_name(&module, &["v4", "lens", "testgen"], "TestgenConcept"),
        "{CI_DAG_PATH}: TestgenSlotSelection must import `Generator` + `TestgenConcept` from v4.lens.testgen"
    );
    for sym in [
        "type CiActiveFloorSkipEvidence",
        "type CiSelectionMode",
        "type CiSelectionReceiptProvenance",
        "type CiClaimSelectionReason",
        "type CiTestClaimSelection",
        "type TestgenSlotSelection",
        "generator: Generator<TestgenConcept>",
        "Active { skip_evidence: CiActiveFloorSkipEvidence }",
        "fn ci_floor_held",
        "fn ci_wave3_shadow_testclaims_selected",
        "claim_projection_hash: test_claim_claim_hash_digest(c: claim)",
        "data ci_wave3_shadow_fixture_fail_closed_receipt",
        "data ci_wave3_shadow_fixture_receipt_ok",
    ] {
        assert!(
            CI_DAG.contains(sym),
            "{CI_DAG_PATH}: Wave 3 Phase 1 must declare `{sym}`"
        );
    }
    let shadow_marker = "fn ci_selection_receipt_shadow(";
    let shadow_start = CI_DAG
        .find(shadow_marker)
        .unwrap_or_else(|| panic!("{CI_DAG_PATH}: missing `{shadow_marker}`"));
    let shadow_rest = &CI_DAG[shadow_start + shadow_marker.len()..];
    let shadow_end = shadow_rest
        .find("\nfn ")
        .map(|idx| shadow_start + shadow_marker.len() + idx)
        .unwrap_or(CI_DAG.len());
    let shadow_body = &CI_DAG[shadow_start..shadow_end];
    assert!(
        !shadow_body.contains("provenance: CiSelectionReceiptProvenance")
            && shadow_body.contains("provenance: FixtureReceipt")
            && shadow_body.contains("testclaim_decisions: Empty"),
        "{CI_DAG_PATH}: step-only ci_selection_receipt_shadow must hardcode FixtureReceipt (no caller provenance; empty claim/testgen until live entry)"
    );
    let heartbeat_body = extract_fn_body(CI_DAG, "ci_selection_receipt_shadow_heartbeat");
    assert!(
        heartbeat_body.contains("provenance: FixtureReceipt"),
        "{CI_DAG_PATH}: shadow heartbeat must tag FixtureReceipt (synthetic path, not live PR git_diff)"
    );
    assert!(
        !heartbeat_body.contains("provenance: LivePrGitDiff"),
        "{CI_DAG_PATH}: shadow heartbeat must not mis-label synthetic receipt as LivePrGitDiff"
    );
    let fixture_binding = CI_DAG
        .split("data ci_wave3_shadow_fixture_fail_closed_receipt:")
        .nth(1)
        .and_then(|rest| rest.split("\ndata ").next())
        .unwrap_or("");
    assert!(
        fixture_binding.contains("provenance: FixtureReceipt"),
        "{CI_DAG_PATH}: Wave 3 fixture receipt must construct with FixtureReceipt provenance"
    );
    assert!(
        !fixture_binding.contains("provenance: LivePrGitDiff"),
        "{CI_DAG_PATH}: Wave 3 fixture receipt must not use LivePrGitDiff"
    );
    assert!(
        !CI_DAG.contains("floor_skip:"),
        "{CI_DAG_PATH}: arbiter ruling — do not persist floor_skip; derive via ci_floor_held"
    );
    assert!(
        CI_DAG.contains("reason: CiClaimSelectionReason"),
        "{CI_DAG_PATH}: claim rows must use closed CiClaimSelectionReason coproduct"
    );
    assert!(
        CI_DAG.contains("AffectedIntersectionNonempty")
            && CI_DAG.contains("AffectedIntersectionEmpty")
            && !CI_DAG.contains("| AffectedFrontierNonempty"),
        "{CI_DAG_PATH}: claim reasons must use per-claim ∩ frontier semantics (not global-only AffectedFrontierNonempty)"
    );
    let reason_body = extract_fn_body(CI_DAG, "ci_wave3_shadow_testclaim_selection_reason");
    assert!(
        reason_body.contains("test_claim_in_rerun_frontier")
            && reason_body.contains("AffectedIntersectionEmpty"),
        "{CI_DAG_PATH}: unselected rows must distinguish global frontier empty vs claim outside nonempty frontier"
    );
    assert!(
        !CI_DAG.contains("fn ci_test_claim_shadow_projection_node"),
        "{CI_DAG_PATH}: claim_projection_hash must use IRT-4 test_claim_claim_hash_digest, not input-only projection"
    );
    assert!(
        CI_DAG.contains("🟡 coproduct dissolution — feature:wave3-shadow-selection-receipt"),
        "{CI_DAG_PATH}: Wave 3 receipt carriers require dissolution disposition marks"
    );
    assert!(
        !CI_DAG.contains("generator_slot: Symbol") && !CI_DAG.contains("concept_category: Symbol"),
        "{CI_DAG_PATH}: TestgenSlotSelection must not flatten `Generator<TestgenConcept>` into Symbol labels"
    );
    for field in [
        "generator: Generator<TestgenConcept>",
        "emits_claim_anchor: ClaimAnchorKey",
        "selected: Bool",
    ] {
        assert!(
            CI_DAG.contains(field),
            "{CI_DAG_PATH}: TestgenSlotSelection must carry `{field}` per worksheet §2.1 + v4.lens.testgen authority"
        );
    }
    assert!(
        CI_DAG.contains("provenance: FixtureReceipt"),
        "{CI_DAG_PATH}: W1.5 + Wave 3 fixture receipts must tag FixtureReceipt (not LivePrGitDiff)"
    );
    assert!(
        !CI_DAG.contains("provenance: LivePrGitDiff"),
        "{CI_DAG_PATH}: Phase 1 — `LivePrGitDiff` is coproduct-only until `ci_selection_receipt_shadow_from_git_diff` lands; forbidden at construction sites"
    );
    assert!(
        CI_DAG.contains("ci_wave3_shadow_claim_roster()"),
        "{CI_DAG_PATH}: selection must consume wave3_shadow_roster authority"
    );
    let roster_module = parse_module(WAVE3_ROSTER_DAG, WAVE3_ROSTER_PATH);
    for name in [
        "ci_wave3_shadow_manual_claim_roster",
        "ci_wave3_shadow_generated_claim_roster",
        "ci_wave3_shadow_claim_roster",
    ] {
        assert!(
            surface_declares_fn(&roster_module, name),
            "{WAVE3_ROSTER_PATH}: must declare `{name}`"
        );
    }
}

#[test]
fn v4_workflow_ci_wave3_live_emit_deferred_in_ci_yml() {
    assert!(
        !CI_YML.contains("emit-ci-wave3-shadow-receipt"),
        "{CI_YML_PATH}: Phase 2 — live shadow emit waits on bootstrap eval entry (adhoc-331899f9-19a)"
    );
    assert!(
        !CI_YML.contains("ci_selection_receipt_shadow_from_git_diff"),
        "{CI_YML_PATH}: modeled live entry not wired until eval harness lands"
    );
}

#[test]
fn v4_workflow_ci_wave3_node_selection_still_shadow_while_component_receipt_live() {
    assert!(
        CI_YML.contains("needs: [affected, ci_floor, infra_isolation]")
            && CI_YML.contains("needs.affected.result"),
        "{CI_YML_PATH}: component affected-set receipt must be live"
    );
    let ci_floor_block = CI_YML
        .split("  ci_floor:")
        .nth(1)
        .and_then(|rest| rest.split("\n  ci:").next())
        .unwrap_or("");
    assert!(
        !ci_floor_block.contains("needs: [affected]"),
        "{CI_YML_PATH}: `ci_floor` must not need `affected`"
    );
    assert!(
        !CI_YML.contains("ci_selection_receipt_shadow_from_git_diff")
            && !CI_YML.contains("emit-ci-wave3-shadow-receipt"),
        "{CI_YML_PATH}: Wave 3 node-frontier receipt emit remains deferred"
    );
}

#[test]
fn v4_workflow_ci_wave3_fixture_receipt_documents_live_ci_deferral() {
    assert!(
        CI_DAG.contains("data ci_wave3_shadow_fixture_fail_closed_receipt")
            && CI_DAG.contains("FixtureReceipt")
            && CI_DAG.contains("adhoc-331899f9-19a"),
        "{CI_DAG_PATH}: must model fixture receipt and live-CI deferral"
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
        CI_DAG.contains("manual_corpus_node_subject_rows` -> `run_manual_testclaim_corpus_eval` -> `corpus_report_tally` -> `witness_manual_corpus_gate_closed"),
        "{CI_DAG_PATH}: TestClaimCorpusEvalCommand dissolution comment must bind the per-row TestClaimRun verdict surface"
    );
    assert!(
        CI_DAG.contains("fn_name == ci_testclaim_corpus_selection_fn"),
        "{CI_DAG_PATH}: ci_command_authority_ok must enforce selection_fn == ci_testclaim_corpus_selection_fn (not unconditional true)"
    );
    for needle in [
        "type TestClaimCorpusDeclarationAuthority",
        "type TestClaimCorpusVerdictSurfaceAuthority",
        "module_path: ci_testclaim_corpus_module_manual_roster_path",
        "module_path: ci_testclaim_corpus_module_runner_path",
        "module_path: ci_testclaim_corpus_module_eval_path",
        "ci_testclaim_corpus_module_manual_roster_path: Symbol = v4_test_claim_manual_manual_corpus_roster",
        "ci_testclaim_corpus_module_runner_path: Symbol = v4_test_claim_workflow_testclaim_corpus_runner",
        "ci_testclaim_corpus_module_eval_path: Symbol = v4_test_claim_workflow_manual_corpus_eval",
        "declaration_name: ci_testclaim_corpus_decl_manual_corpus_node_subject_rows_name",
        "declaration_name: ci_testclaim_corpus_decl_run_manual_testclaim_corpus_eval_name",
        "declaration_name: ci_testclaim_corpus_decl_corpus_report_tally_name",
        "declaration_name: ci_testclaim_corpus_decl_witness_manual_corpus_gate_closed_name",
        "ci_testclaim_corpus_decl_manual_corpus_node_subject_rows_name: Symbol = manual_corpus_node_subject_rows",
        "ci_testclaim_corpus_decl_run_manual_testclaim_corpus_eval_name: Symbol = run_manual_testclaim_corpus_eval",
        "ci_testclaim_corpus_decl_corpus_report_tally_name: Symbol = corpus_report_tally",
        "ci_testclaim_corpus_decl_witness_manual_corpus_gate_closed_name: Symbol = witness_manual_corpus_gate_closed",
        "fn ci_testclaim_corpus_eval_command() -> CiCommand",
        "verdict_surface: ci_testclaim_corpus_verdict_surface_authority()",
        "surface == ci_testclaim_corpus_verdict_surface_authority()",
        "ci_projection_command_verdict_surface_edge",
        "ci_projection_corpus_surface_run_roster_edge",
        "ci_projection_corpus_surface_eval_report_edge",
        "ci_projection_corpus_surface_verdict_tally_edge",
        "ci_projection_corpus_surface_gate_witness_edge",
        "ci_projection_declaration_module_edge",
        "ci_projection_declaration_name_edge",
        "ci_declaration_authority_projection_node",
        "ci_atom(sym: a.module_path)",
        "ci_atom(sym: a.declaration_name)",
        "feature:t38-testclaim-corpus-roster-claim-ref-frontier",
        "generated roster/item-registry reflection derives these TestClaimRef inputs directly from",
        "Forbidden: adding/removing manual corpus rows without the matching claim id here.",
        "ci_upsert_file_set_input(segment: \"src/v4/test/claim/workflow/manual_corpus_eval.dag\")",
        "segment == \"src/v4/test/claim/workflow/manual_corpus_eval.dag\"",
    ] {
        assert!(
            CI_DAG.contains(needle),
            "{CI_DAG_PATH}: TestClaimCorpusEvalCommand must carry modeled corpus verdict authority `{needle}`"
        );
    }
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
        CI_DAG.contains("command: ci_testclaim_corpus_eval_command()"),
        "{CI_DAG_PATH}: testclaim_corpus_eval_execution job must bind the canonical corpus-eval command"
    );
    assert!(
        CI_DAG.contains("payload_type: ci_command_projection_node(")
            && CI_DAG.contains("c: ci_testclaim_corpus_eval_command()"),
        "{CI_DAG_PATH}: testclaim corpus CiUpsertStep payload_type must use command projection (content_hash authority, not static tag)"
    );
    assert!(
        !CI_DAG.contains("ci_cache_cmd_testclaim_corpus_eval_tag"),
        "{CI_DAG_PATH}: static testclaim cache tag dissolved — cache via ci_upsert_step_cache_digest / content_hash projection"
    );
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiCommand"
        )),
        "{CI_DAG_PATH}: CiCommand sum type must exist"
    );
}

#[test]
fn v4_workflow_ci_t38_script_checks_generated_manual_corpus_eval_receipt() {
    for needle in [
        "src/v4_test_claim_workflow_manual_corpus_eval.rs",
        "check_generated_corpus_eval",
        "manual_corpus_all_pass",
        "manual_corpus_gate",
        "witness_manual_corpus_gate_closed",
        "corpus_report_tally(report);",
        "explicit_return",
        "\\breturn\\b",
        "inverted_zero_comparison",
        "(?<![A-Za-z0-9_:])(?:!\\(*|\\(*false\\)*={2}\\(*|\\(*true\\)*!=\\(*)",
        "tally\\.(?:fail|deferred)={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\\b\\)*",
        "fail_deferred_conjunction",
        "(?:^|;)\\(*tally\\.fail={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\\b",
        "\\)*&&",
        "&&",
        "tally\\.deferred={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\\b",
        "\\)*\\}$",
        "inline_empty_gate",
        "if(?<!!)is_empty\\([^)]*report[^)]*entries",
        "\\{false\\}else\\{manual_corpus_all_pass\\([^)]*report",
        "manual_corpus_gate(run_manual_testclaim_corpus_eval())",
    ] {
        assert!(
            TESTCLAIM_CORPUS_EVAL_SCRIPT.contains(needle),
            "{TESTCLAIM_CORPUS_EVAL_SCRIPT_PATH}: missing generated corpus-eval receipt probe `{needle}`"
        );
    }
}

#[test]
fn v4_workflow_ci_t38_script_receipt_rejects_inverted_zero_predicates() {
    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(
            r#"
import re

explicit_return = re.compile(r"\breturn\b")
inverted_zero_comparison = re.compile(
    r"(?<![A-Za-z0-9_:])(?:!\(*|\(*false\)*={2}\(*|\(*true\)*!=\(*)"
    r"tally\.(?:fail|deferred)={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\b\)*"
)
fail_deferred_conjunction = re.compile(
    r"(?:^|;)\(*tally\.fail={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\b\)*&&"
    r"\(*tally\.deferred={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\b\)*\}$"
)

def receipt_accepts(source):
    normalized_source = "".join(source.split())
    return (
        not explicit_return.search(source)
        and not inverted_zero_comparison.search(normalized_source)
        and fail_deferred_conjunction.search(normalized_source)
    )

assert receipt_accepts("lettally=x;tally.fail==Nat::Zero&&tally.deferred==Nat::Zero}")
assert receipt_accepts("lettally=x;(tally.fail==Zero)&&(tally.deferred==Zero)}")
assert not receipt_accepts(
    "lettally=x;(tally.fail==Zero)==false&&tally.deferred==Zero}"
)
assert not receipt_accepts(
    "lettally=x;tally.fail==Zero&&(tally.deferred==Zero)==false}"
)
assert not receipt_accepts(
    "lettally=x;tally.fail==NotZero&&tally.deferred==NotZero}"
)
for non_returned in [
    "lettally=x;tally.fail==Zero&&tally.deferred==Zero;false}",
    "lettally=x;letok=tally.fail==Zero&&tally.deferred==Zero;false}",
    "lettally=x;{letinner=1;tally.fail==Zero&&tally.deferred==Zero};false}",
    "return (tally.fail == Zero) == false && tally.deferred == Zero; tally.fail == Zero && tally.deferred == Zero}",
    "let tally = x; return (tally.fail == Zero) == false && tally.deferred == Zero; tally.fail == Zero && tally.deferred == Zero}",
]:
    assert not receipt_accepts(non_returned)
for inverted in [
    "lettally=x;!tally.fail==Zero&&tally.deferred==Zero}",
    "lettally=x;!(tally.fail==Zero)&&tally.deferred==Zero}",
    "lettally=x;!((tally.fail==Zero))&&tally.deferred==Zero}",
    "lettally=x;false==(tally.fail==Zero)&&tally.deferred==Zero}",
    "lettally=x;(false)==(tally.fail==Zero)&&tally.deferred==Zero}",
    "lettally=x;false==((tally.fail==Zero))&&tally.deferred==Zero}",
    "lettally=x;true!=(tally.fail==Zero)&&tally.deferred==Zero}",
    "lettally=x;(true)!=(tally.fail==Zero)&&tally.deferred==Zero}",
    "lettally=x;true!=((tally.fail==Zero))&&tally.deferred==Zero}",
    "lettally=x;tally.fail==Zero&&!tally.deferred==Zero}",
    "lettally=x;tally.fail==Zero&&!(tally.deferred==Zero)}",
    "lettally=x;tally.fail==Zero&&!((tally.deferred==Zero))}",
    "lettally=x;tally.fail==Zero&&false==(tally.deferred==Zero)}",
    "lettally=x;tally.fail==Zero&&(false)==(tally.deferred==Zero)}",
    "lettally=x;tally.fail==Zero&&false==((tally.deferred==Zero))}",
    "lettally=x;tally.fail==Zero&&true!=(tally.deferred==Zero)}",
    "lettally=x;tally.fail==Zero&&(true)!=(tally.deferred==Zero)}",
    "lettally=x;tally.fail==Zero&&true!=((tally.deferred==Zero))}",
]:
    assert not receipt_accepts(inverted)
"#,
        )
        .output()
        .expect("python3 should run T-38 receipt regex regression");

    assert!(
        output.status.success(),
        "{TESTCLAIM_CORPUS_EVAL_SCRIPT_PATH}: generated corpus-eval receipt regex accepted an inverted zero predicate\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
