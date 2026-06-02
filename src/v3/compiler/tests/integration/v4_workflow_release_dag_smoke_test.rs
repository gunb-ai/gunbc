//! **Layer:** integration
//!
//! RELEASE_TODO.md §5 Phase 1a–1b: `src/v4/workflow/release.dag` is semantic authority;
//! `.github/workflows/release.yml` is a hand-synced projection until YamlStatic emission lands
//! (same deferral posture as `v4.workflow.ci` / T-24). Install scripts (`install.sh`,
//! `install/release-target-triples.sh`) are hand-synced from `install.dag` (Phase 1b).
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge deferred.
//!
//! **ROADMAP:** `_internal/ROADMAP_OPS.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`.
//! **TASKS.md** RELEASE_TODO §5 Phase 1a (`src/v4/workflow/release.dag` + hand-synced release.yml).
//!
//! **Dissolution:** remove when release pipeline is exercised only by `.dag` `TestClaim` rows /
//! YamlStatic emission without this hand-Rust parse harness.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem, SurfaceLiteral, SurfaceRecordField};
use v3_compiler::tokenize_for_test;

const RELEASE_DAG: &str = include_str!("../../../../v4/workflow/release.dag");
const RELEASE_DAG_PATH: &str = "src/v4/workflow/release.dag";
const RELEASE_YML: &str = include_str!("../../../../../.github/workflows/release.yml");
const RELEASE_YML_PATH: &str = ".github/workflows/release.yml";
const INSTALL_SH: &str = include_str!("../../../../../install.sh");
const INSTALL_SH_PATH: &str = "install.sh";
const RELEASE_TARGET_TRIPLES_SH: &str =
    include_str!("../../../../../install/release-target-triples.sh");
const RELEASE_TARGET_TRIPLES_SH_PATH: &str = "install/release-target-triples.sh";
const V2_COMPILER_CARGO_TOML: &str = include_str!("../../../../v2/stage0/Cargo.toml");
const V2_COMPILER_CARGO_TOML_PATH: &str = "src/v2/stage0/Cargo.toml";

const RELEASE_PUBLISHED_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
];

const RELEASE_PUBLISHED_ARTIFACTS: &[&str] = &[
    "gunbc-x86_64-unknown-linux-musl",
    "gunbc-aarch64-unknown-linux-musl",
    "gunbc-x86_64-apple-darwin",
    "gunbc-aarch64-apple-darwin",
    "gunbc-x86_64-pc-windows-msvc.exe",
    "gunbc-aarch64-pc-windows-msvc.exe",
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
        RELEASE_DAG.contains("CrossMuslGunbcBuild { target: String }"),
        "{RELEASE_DAG_PATH}: musl builds must use CrossMuslGunbcBuild"
    );
    assert!(
        RELEASE_DAG.contains("| NativeDarwinGunbcBuild { target: String }"),
        "{RELEASE_DAG_PATH}: darwin builds must use NativeDarwinGunbcBuild"
    );
    assert!(
        RELEASE_DAG.contains("| NativeWindowsGunbcBuild { target: String }"),
        "{RELEASE_DAG_PATH}: windows builds must use NativeWindowsGunbcBuild"
    );
    assert!(
        RELEASE_DAG.contains("| PublishGitHubRelease"),
        "{RELEASE_DAG_PATH}: publish job must model GH Release upload"
    );
    assert!(
        !RELEASE_DAG.contains("bundle_install_sh"),
        "{RELEASE_DAG_PATH}: install bundling is modeled in install.dag, not release.dag"
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
        RELEASE_DAG.contains("data release_published_artifact_names: List<String> ="),
        "{RELEASE_DAG_PATH}: must declare published artifact basename authority (`.exe` policy)"
    );
    assert!(
        RELEASE_DAG.contains("release_matrix_row_targets(rows: release_build_matrix)"),
        "{RELEASE_DAG_PATH}: published triples must project from release_build_matrix (single source)"
    );
    assert!(
        RELEASE_DAG.contains("release_matrix_row_artifact_basenames(rows: release_build_matrix)"),
        "{RELEASE_DAG_PATH}: published artifact names must project from matrix artifact_basename"
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
        RELEASE_DAG.contains("release_build_windows_x86")
            && RELEASE_DAG.contains("aarch64-pc-windows-msvc"),
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
    for artifact in RELEASE_PUBLISHED_ARTIFACTS {
        assert!(
            RELEASE_DAG.contains(artifact),
            "{RELEASE_DAG_PATH}: release_build_matrix must model artifact basename `{artifact}`"
        );
    }
}

#[test]
fn v4_workflow_release_published_authority_single_writer() {
    for target in RELEASE_PUBLISHED_TARGETS {
        assert!(
            RELEASE_YML.contains(target),
            "{RELEASE_YML_PATH}: matrix must include `{target}`"
        );
    }
    for artifact in RELEASE_PUBLISHED_ARTIFACTS {
        assert!(
            RELEASE_YML.contains(artifact),
            "{RELEASE_YML_PATH}: matrix must include modeled artifact basename `{artifact}`"
        );
        assert!(
            RELEASE_YML.contains(&format!("artifact_basename: {artifact}")),
            "{RELEASE_YML_PATH}: artifact basename must be explicit matrix field `{artifact}`"
        );
    }
    assert!(
        RELEASE_YML.contains("dist/${{ matrix.artifact_basename }}"),
        "{RELEASE_YML_PATH}: build/upload must use modeled artifact_basename (not hardcoded .exe branch)"
    );
    assert!(
        RELEASE_YML.contains("install.sh"),
        "{RELEASE_YML_PATH}: Phase 1b must bundle install.sh on GitHub Releases"
    );
    assert!(
        RELEASE_YML.contains("dist/release-target-triples.sh"),
        "{RELEASE_YML_PATH}: Phase 1b must stage flat release-target-triples.sh (no duplicate basename under dist/scripts/)"
    );
    assert!(
        !RELEASE_YML.contains("dist/scripts/"),
        "{RELEASE_YML_PATH}: must not upload duplicate release-target-triples.sh basename via dist/scripts/"
    );
}

#[test]
fn v4_install_scripts_hand_synced_to_release_authority() {
    for target in &RELEASE_PUBLISHED_TARGETS[..4] {
        assert!(
            RELEASE_TARGET_TRIPLES_SH.contains(target),
            "{RELEASE_TARGET_TRIPLES_SH_PATH}: POSIX install must list `{target}`"
        );
        assert!(
            INSTALL_SH.contains(target) || RELEASE_TARGET_TRIPLES_SH.contains(target),
            "{INSTALL_SH_PATH}: install path must cover release triple `{target}`"
        );
    }
    assert!(
        INSTALL_SH.contains("asset=\"gunbc-${target}\""),
        "{INSTALL_SH_PATH}: asset naming must project release artifact basename gunbc-{{triple}} at runtime"
    );
    for artifact in &RELEASE_PUBLISHED_ARTIFACTS[..4] {
        let triple = artifact.strip_prefix("gunbc-").unwrap_or(artifact);
        assert!(
            RELEASE_TARGET_TRIPLES_SH.contains(triple),
            "{RELEASE_TARGET_TRIPLES_SH_PATH}: POSIX targets must include triple `{triple}` for artifact `{artifact}`"
        );
    }
    assert!(
        INSTALL_SH.contains("install/release-target-triples.sh"),
        "{INSTALL_SH_PATH}: must source hand-synced target authority script"
    );
    assert!(
        RELEASE_TARGET_TRIPLES_SH.contains("detect_release_target"),
        "{RELEASE_TARGET_TRIPLES_SH_PATH}: must export detect_release_target"
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
    let publish_step = expr_string(record_body_field(live, "create_release_step_name"));
    let gh_action = expr_string(record_body_field(live, "gh_release_action"));
    let cargo_package = expr_string(record_body_field(live, "cargo_package"));
    let cargo_bin = expr_string(record_body_field(live, "cargo_bin"));
    let cross_version = expr_string(record_body_field(live, "cross_version"));

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
        V2_COMPILER_CARGO_TOML.contains("name = \"v2-compiler\"")
            && V2_COMPILER_CARGO_TOML.contains("[[bin]]")
            && V2_COMPILER_CARGO_TOML.contains("name = \"gunbc\""),
        "{V2_COMPILER_CARGO_TOML_PATH}: release cargo_package/cargo_bin must match declared [[bin]] authority"
    );
    assert!(
        cargo_package == "v2-compiler" && cargo_bin == "gunbc",
        "{RELEASE_DAG_PATH}: cargo_package/cargo_bin must mirror {V2_COMPILER_CARGO_TOML_PATH}"
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
        RELEASE_DAG.contains("release_matrix_row_tuple_well_formed"),
        "{RELEASE_DAG_PATH}: matrix well-formedness must validate (target, runner, cross, artifact_basename) tuples"
    );
    const RELEASE_MATRIX_YML_ROW_SNIPPETS: &[&str] = &[
        "target: x86_64-unknown-linux-musl\n            runner: ubuntu-24.04\n            cross: true\n            artifact_basename: gunbc-x86_64-unknown-linux-musl",
        "target: aarch64-unknown-linux-musl\n            runner: ubuntu-24.04\n            cross: true\n            artifact_basename: gunbc-aarch64-unknown-linux-musl",
        "target: x86_64-apple-darwin\n            runner: macos-15-intel\n            cross: false\n            artifact_basename: gunbc-x86_64-apple-darwin",
        "target: aarch64-apple-darwin\n            runner: macos-14\n            cross: false\n            artifact_basename: gunbc-aarch64-apple-darwin",
        "target: x86_64-pc-windows-msvc\n            runner: windows-2022\n            cross: false\n            artifact_basename: gunbc-x86_64-pc-windows-msvc.exe",
        "target: aarch64-pc-windows-msvc\n            runner: windows-11-arm\n            cross: false\n            artifact_basename: gunbc-aarch64-pc-windows-msvc.exe",
    ];
    const RELEASE_MATRIX_DAG_ROW_SNIPPETS: &[&str] = &[
        "target: \"x86_64-unknown-linux-musl\"\n    runner: \"ubuntu-24.04\"\n    cross: true\n    artifact_basename: \"gunbc-x86_64-unknown-linux-musl\"",
        "target: \"aarch64-unknown-linux-musl\"\n    runner: \"ubuntu-24.04\"\n    cross: true\n    artifact_basename: \"gunbc-aarch64-unknown-linux-musl\"",
        "target: \"x86_64-apple-darwin\"\n    runner: \"macos-15-intel\"\n    cross: false\n    artifact_basename: \"gunbc-x86_64-apple-darwin\"",
        "target: \"aarch64-apple-darwin\"\n    runner: \"macos-14\"\n    cross: false\n    artifact_basename: \"gunbc-aarch64-apple-darwin\"",
        "target: \"x86_64-pc-windows-msvc\"\n    runner: \"windows-2022\"\n    cross: false\n    artifact_basename: \"gunbc-x86_64-pc-windows-msvc.exe\"",
        "target: \"aarch64-pc-windows-msvc\"\n    runner: \"windows-11-arm\"\n    cross: false\n    artifact_basename: \"gunbc-aarch64-pc-windows-msvc.exe\"",
    ];
    for (yml_snippet, dag_snippet) in RELEASE_MATRIX_YML_ROW_SNIPPETS
        .iter()
        .zip(RELEASE_MATRIX_DAG_ROW_SNIPPETS.iter())
    {
        assert!(
            RELEASE_YML.contains(yml_snippet) && RELEASE_DAG.contains(dag_snippet),
            "{RELEASE_YML_PATH}: matrix row tuple must match {RELEASE_DAG_PATH} (target+runner+cross+artifact_basename)"
        );
    }
    assert!(
        !RELEASE_YML.contains("self-hosted"),
        "{RELEASE_YML_PATH}: release workflow must not use srv1/srv2 self-hosted pool"
    );
}
