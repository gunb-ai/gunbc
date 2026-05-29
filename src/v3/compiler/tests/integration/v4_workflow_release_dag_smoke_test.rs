//! **Layer:** integration
//!
//! RELEASE_TODO.md §5 Phase 1: `src/v4/workflow/release.dag` is semantic authority;
//! `.github/workflows/release.yml` is a hand-synced projection until YamlStatic emission lands
//! (same deferral posture as `v4.workflow.ci` / T-24).
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge deferred.
//!
//! **ROADMAP:** `_internal/ROADMAP_OPS.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`.
//! **TASKS.md** RELEASE_TODO §5 Phase 1 (`src/v4/workflow/release.dag` + hand-synced release.yml).
//!
//! **Dissolution:** remove when release pipeline + install target selection are exercised only by
//! `.dag` `TestClaim` rows / YamlStatic emission without this hand-Rust parse harness.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem, SurfaceLiteral, SurfaceRecordField};
use v3_compiler::tokenize_for_test;

const RELEASE_DAG: &str = include_str!("../../../../v4/workflow/release.dag");
const RELEASE_DAG_PATH: &str = "src/v4/workflow/release.dag";
const RELEASE_YML: &str = include_str!("../../../../../.github/workflows/release.yml");
const RELEASE_YML_PATH: &str = ".github/workflows/release.yml";
const RELEASE_TARGET_SCRIPT: &str =
    include_str!("../../../../../scripts/release-target-triples.sh");
const RELEASE_TARGET_SCRIPT_PATH: &str = "scripts/release-target-triples.sh";
const INSTALL_SH: &str = include_str!("../../../../../install.sh");
const INSTALL_SH_PATH: &str = "install.sh";

const RELEASE_PUBLISHED_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

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

fn workflow_contains_targets(workflow_yml: &str, targets: &[&str]) {
    for target in targets {
        assert!(
            workflow_yml.contains(target),
            "{RELEASE_YML_PATH}: must reference matrix target `{target}`"
        );
        assert!(
            workflow_yml.contains(&format!("gunbc-{target}")),
            "{RELEASE_YML_PATH}: artifact name must include `gunbc-{target}`"
        );
    }
}

#[test]
fn v4_workflow_release_dag_tokenizes_and_parses() {
    let _module = parse_module(RELEASE_DAG, RELEASE_DAG_PATH);
}

#[test]
fn v4_workflow_release_semantics_modeled() {
    let module = parse_module(RELEASE_DAG, RELEASE_DAG_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "workflow", "release_dist"]],
        "{RELEASE_DAG_PATH}: module authority path"
    );
    assert!(
        RELEASE_DAG.contains("| CrossMuslGunbcBuild { target: String }"),
        "{RELEASE_DAG_PATH}: musl builds must use CrossMuslGunbcBuild"
    );
    assert!(
        RELEASE_DAG.contains("| NativeDarwinGunbcBuild { target: String }"),
        "{RELEASE_DAG_PATH}: darwin builds must use NativeDarwinGunbcBuild"
    );
    assert!(
        RELEASE_DAG.contains("| PublishGitHubRelease { bundle_install_sh: Bool }"),
        "{RELEASE_DAG_PATH}: publish job must model GH Release upload"
    );
    assert!(
        RELEASE_DAG.contains("YamlStatic `release_pipeline →"),
        "{RELEASE_DAG_PATH}: must document YamlStatic emission deferral"
    );
    assert!(
        RELEASE_DAG.contains("data release_published_target_triples: List<String> ="),
        "{RELEASE_DAG_PATH}: must declare published target triple authority"
    );
    assert!(
        RELEASE_DAG.contains("release_matrix_row_targets(rows: release_build_matrix)"),
        "{RELEASE_DAG_PATH}: published triples must project from release_build_matrix (single source)"
    );
    assert!(
        RELEASE_DAG.contains("import v4.std.node { Symbol }"),
        "{RELEASE_DAG_PATH}: job/gate ids must import Symbol from v4.std.node"
    );
    assert!(
        RELEASE_DAG.contains("🟡 coproduct dissolution")
            && RELEASE_DAG.contains("🟡 matrix-row validator dissolution")
            && RELEASE_DAG.contains("type ReleaseCommand"),
        "{RELEASE_DAG_PATH}: ReleaseCommand must carry Practice-4 coproduct dissolution mark"
    );
    assert!(
        RELEASE_DAG.contains("release_build_musl_x86")
            && RELEASE_DAG.contains("aarch64-unknown-linux-musl")
            && RELEASE_DAG.contains("aarch64-apple-darwin"),
        "{RELEASE_DAG_PATH}: release_pipeline must model one build job per release_build_matrix row"
    );
    assert!(
        RELEASE_DAG.contains("release_pipeline_jobs_cover_matrix")
            && RELEASE_DAG.contains("release_build_commands_use_matrix_targets_only"),
        "{RELEASE_DAG_PATH}: well-formedness must bind hand-expanded build targets to release_build_matrix"
    );
    assert!(
        RELEASE_DAG.contains("command: UploadGunbcMatrixArtifact")
            && RELEASE_DAG.contains("release_matrix_upload_jobs_present")
            && RELEASE_DAG.contains("release_upload_needs_build_for_target")
            && RELEASE_DAG.contains("release_publish_job_drains_matrix_uploads"),
        "{RELEASE_DAG_PATH}: pipeline must model matrix artifact upload drain before publish"
    );
    assert!(
        RELEASE_DAG.contains("PublishGitHubRelease")
            && RELEASE_DAG.contains("release_publish_drain_gap"),
        "{RELEASE_DAG_PATH}: well-formedness must fail-closed when publish drain is missing"
    );
    assert!(
        !RELEASE_DAG.contains("!row.cross"),
        "{RELEASE_DAG_PATH}: M1(2.7) surface cannot tokenize unary ! on fields; use row.cross == false"
    );
    for target in RELEASE_PUBLISHED_TARGETS {
        assert!(
            RELEASE_DAG.contains(target),
            "{RELEASE_DAG_PATH}: release_build_matrix must include `{target}`"
        );
    }
}

#[test]
fn v4_workflow_release_target_authority_single_writer() {
    for target in RELEASE_PUBLISHED_TARGETS {
        assert!(
            RELEASE_DAG.contains(target),
            "{RELEASE_DAG_PATH}: `release_published_target_triples` must include `{target}`"
        );
        assert!(
            RELEASE_YML.contains(target),
            "{RELEASE_YML_PATH}: matrix must include `{target}`"
        );
        assert!(
            RELEASE_TARGET_SCRIPT.contains(target),
            "{RELEASE_TARGET_SCRIPT_PATH}: shell authority must include `{target}`"
        );
    }
    assert!(
        INSTALL_SH.contains("scripts/release-target-triples.sh"),
        "{INSTALL_SH_PATH}: must load scripts/release-target-triples.sh"
    );
    assert!(
        INSTALL_SH.contains("detect_release_target"),
        "{INSTALL_SH_PATH}: must delegate target detection to shell authority"
    );
    assert!(
        INSTALL_SH.contains("releases/download/${VERSION}/")
            && INSTALL_SH.contains("releases/latest/download/"),
        "{INSTALL_SH_PATH}: target authority curl fallback must use same GH Release channel as binary"
    );
    assert!(
        !INSTALL_SH.contains("./scripts/release-target-triples.sh"),
        "{INSTALL_SH_PATH}: must not source cwd ./scripts before release channel (P2 single authority)"
    );
    assert!(
        !INSTALL_SH.contains("printf '%s\\n' 'x86_64-unknown-linux-musl'"),
        "{INSTALL_SH_PATH}: must not embed a parallel OS/arch → triple mapping"
    );
}

#[test]
fn v4_workflow_release_modeled_and_bound_to_release_yml() {
    let module = parse_module(RELEASE_DAG, RELEASE_DAG_PATH);
    let live = data_body(&module, "release_live_workflow_signal");
    let tag_pattern = expr_string(record_body_field(live, "tag_pattern"));
    let workflow_name = expr_string(record_body_field(live, "workflow_name"));
    let cross_step = expr_string(record_body_field(live, "cross_install_step_name"));
    let aarch64_strip_step = expr_string(record_body_field(live, "aarch64_strip_step_name"));
    let build_step = expr_string(record_body_field(live, "build_gunbc_step_name"));
    let upload_step = expr_string(record_body_field(live, "upload_artifact_step_name"));
    let stage_step = expr_string(record_body_field(live, "stage_install_sh_step_name"));
    let publish_step = expr_string(record_body_field(live, "create_release_step_name"));
    let gh_action = expr_string(record_body_field(live, "gh_release_action"));
    let cargo_package = expr_string(record_body_field(live, "cargo_package"));
    let cargo_bin = expr_string(record_body_field(live, "cargo_bin"));
    let cross_version = expr_string(record_body_field(live, "cross_version"));
    let install_sh = expr_string(record_body_field(live, "install_sh_bundle_name"));

    assert!(
        RELEASE_YML.contains(&format!("name: {workflow_name}")),
        "{RELEASE_YML_PATH}: workflow name must match modeled `{workflow_name}`"
    );
    assert!(
        RELEASE_YML.contains(tag_pattern),
        "{RELEASE_YML_PATH}: tag trigger must include modeled pattern `{tag_pattern}`"
    );
    assert!(
        RELEASE_YML.contains("on:\n  push:\n    tags:") || RELEASE_YML.contains("tags:\n      -"),
        "{RELEASE_YML_PATH}: must use tag push trigger"
    );
    workflow_contains_targets(RELEASE_YML, RELEASE_PUBLISHED_TARGETS);
    assert!(
        RELEASE_YML.contains(&format!("- name: {cross_step}")),
        "{RELEASE_YML_PATH}: must declare cross install step"
    );
    assert!(
        RELEASE_YML.contains(&format!("cross --version {cross_version}")),
        "{RELEASE_YML_PATH}: must pin cross to modeled version"
    );
    assert!(
        RELEASE_YML.contains(&format!("- name: {aarch64_strip_step}")),
        "{RELEASE_YML_PATH}: must declare aarch64 strip step for musl artifacts"
    );
    assert!(
        RELEASE_YML.contains("aarch64-linux-gnu-strip"),
        "{RELEASE_YML_PATH}: aarch64 musl strip must use cross-target binutils"
    );
    assert!(
        RELEASE_YML.contains(&format!("- name: {build_step}")),
        "{RELEASE_YML_PATH}: must declare gunbc build step"
    );
    assert!(
        RELEASE_YML.contains(&format!("-p {cargo_package} --bin {cargo_bin}")),
        "{RELEASE_YML_PATH}: build must target modeled package/bin"
    );
    assert!(
        RELEASE_YML.contains("matrix.cross"),
        "{RELEASE_YML_PATH}: build must branch on matrix.cross for musl vs native"
    );
    assert!(
        RELEASE_YML.contains(&format!("- name: {upload_step}")),
        "{RELEASE_YML_PATH}: must upload matrix artifacts"
    );
    assert!(
        RELEASE_YML.contains(&format!("- name: {stage_step}")),
        "{RELEASE_YML_PATH}: must stage release assets"
    );
    assert!(
        RELEASE_YML.contains(&format!("cp {install_sh} dist/{install_sh}")),
        "{RELEASE_YML_PATH}: publish bundle must include modeled install.sh"
    );
    assert!(
        RELEASE_YML
            .contains("scripts/release-target-triples.sh dist/scripts/release-target-triples.sh"),
        "{RELEASE_YML_PATH}: publish bundle must ship target-authority script beside install.sh"
    );
    assert!(
        RELEASE_YML.contains("scripts/release-target-triples.sh dist/release-target-triples.sh"),
        "{RELEASE_YML_PATH}: publish bundle must ship flat target-authority asset for releases/latest/download"
    );
    assert!(
        RELEASE_YML.contains("dist/*") && RELEASE_YML.contains("dist/scripts/*"),
        "{RELEASE_YML_PATH}: gh-release upload globs must include nested dist/scripts/ assets"
    );
    assert!(
        RELEASE_YML.contains(&format!("- name: {publish_step}")),
        "{RELEASE_YML_PATH}: must create GitHub Release"
    );
    assert!(
        RELEASE_YML.contains(&format!("uses: {gh_action}")),
        "{RELEASE_YML_PATH}: release step must use modeled action"
    );
    assert!(
        RELEASE_YML.contains("generate_release_notes: true"),
        "{RELEASE_YML_PATH}: release must generate notes"
    );
    assert!(
        RELEASE_YML.contains("ubuntu-24.04") && RELEASE_YML.contains("macos-15-intel"),
        "{RELEASE_YML_PATH}: must use github-hosted runners for release matrix"
    );
    assert!(
        !RELEASE_YML.contains("self-hosted"),
        "{RELEASE_YML_PATH}: release workflow must not use srv1/srv2 self-hosted pool"
    );
}
