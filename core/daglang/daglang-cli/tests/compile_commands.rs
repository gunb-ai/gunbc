use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

fn makegen_file() -> PathBuf {
    workspace_root().join("dsl/tools/makegen.dag")
}

fn unique_temp_file(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "daglang_cli_compile_cmd_{name}_{}_{}.dag",
        std::process::id(),
        nanos
    ))
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "daglang_cli_compile_cmd_dir_{name}_{}_{}",
        std::process::id(),
        nanos
    ))
}

#[test]
fn compile_command_emits_summary_for_single_file() {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        output.status.success(),
        "compile command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));
    assert!(stdout.contains("target/generated/rust/main.rs"));
}

#[test]
fn compile_command_single_file_unresolved_service_call_reports_lower_error() {
    let fixture = unique_temp_file("compile_unresolved_service_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved service call fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for unresolved service call"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("lower error: unresolved service call"));
    assert!(stderr.contains("MissingStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_shows_lowered_nodes_and_edges() {
    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand");

    assert!(
        output.status.success(),
        "expand command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Nodes:"));
    assert!(stdout.contains("tools.makegen::render_makefile"));
    assert!(stdout.contains("tools.makegen::makegen"));
}

#[test]
fn manifest_command_shows_derived_progress_manifest() {
    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ProgressManifest:"));
    assert!(stdout.contains("total_nodes:"));
    assert!(stdout.contains("waves:"));
    assert!(stdout.contains("TestObligations:"));
    assert!(stdout.contains("service_transport_prepare_targets:"));
    assert!(stdout.contains("service_param_source_targets:"));
    assert!(stdout.contains("resource_provide_targets:"));
}

#[test]
fn manifest_command_reports_non_zero_transport_and_lifecycle_obligations() {
    let fixture = unique_temp_file("manifest_obligations");
    std::fs::write(
        &fixture,
        r#"module sample.obligations
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
resource TempFile {
  acquire {
    let path = "/tmp/file"
  }
  release {
    let done = true
  }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on fixture");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("service_transport_prepare_targets: 1"));
    assert!(stdout.contains("service_transport_execute_targets: 1"));
    assert!(stdout.contains("service_transport_parse_targets: 1"));
    assert!(stdout.contains("service_param_source_targets: 1"));
    assert!(stdout.contains("resource_acquire_targets: 1"));
    assert!(stdout.contains("resource_release_targets: 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn manifest_command_reports_zero_service_param_source_targets_for_literal_args() {
    let fixture = unique_temp_file("manifest_param_sources_zero");
    std::fs::write(
        &fixture,
        r#"module sample.literal
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(path: "README.md")
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on fixture");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("service_param_source_targets: 0"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn manifest_command_interface_only_provides_has_no_release_obligation() {
    let fixture = unique_temp_file("manifest_interface_provides");
    std::fs::write(
        &fixture,
        r#"module sample.provides
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on fixture");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("resource_provide_targets: 1"));
    assert!(stdout.contains("resource_release_targets: 0"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn viz_command_renders_mermaid_for_compiled_file() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz file");

    assert!(
        output.status.success(),
        "viz command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("flowchart TB"));
    assert!(stdout.contains("tools.makegen::render_makefile"));
}

#[test]
fn compile_command_reports_diagnostics_for_invalid_file() {
    let broken = unique_temp_file("broken");
    std::fs::write(&broken, "module sample.broken\nfn broken( -> String {")
        .expect("failed to write broken source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&broken)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for broken file");

    assert!(!output.status.success(), "broken source should fail compile");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}

#[test]
fn expand_command_reports_unresolved_service_call_lower_error() {
    let fixture = unique_temp_file("unresolved_service_call");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on unresolved service fixture");

    assert!(
        !output.status.success(),
        "expand should fail when service call endpoint cannot be resolved"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("lower error: unresolved service call"));
    assert!(stderr.contains("MissingStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_reports_unresolved_uses_lower_error() {
    let fixture = unique_temp_file("unresolved_uses");
    std::fs::write(
        &fixture,
        r#"module sample.resources
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on unresolved uses fixture");

    assert!(
        !output.status.success(),
        "expand should fail when uses target cannot be resolved"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("lower error: unresolved used resource"));
    assert!(stderr.contains("fs: MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_reports_unresolved_provides_lower_error() {
    let fixture = unique_temp_file("unresolved_provides");
    std::fs::write(
        &fixture,
        r#"module sample.resources
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on unresolved provides fixture");

    assert!(
        !output.status.success(),
        "expand should fail when provides target cannot be resolved"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("lower error: unresolved provided resource"));
    assert!(stderr.contains("out: MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_shows_param_source_wiring_for_identifier_service_args() {
    let fixture = unique_temp_file("service_param_source_expand");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on fixture");

    assert!(
        output.status.success(),
        "expand command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("param_source_sample_services_run_path"));
    assert!(stdout.contains(
        "param_source_sample_services_run_path.path -> prepare_transport_sample_services_FsStorage_read.path"
    ));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_imports() {
    let root = unique_temp_dir("unresolved_import");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nimport missing.dep\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved imports"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("unresolved import"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_definitions() {
    let root = unique_temp_dir("duplicate_definitions");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn run() -> Unit {}
func run() -> { ok: Bool } {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("duplicate definition `run` in module `sample.main`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_output_fields() {
    let root = unique_temp_dir("duplicate_output_fields");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run() -> { ok: Bool, ok: Bool } {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate output fields"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("duplicate output field `ok` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_parameters() {
    let root = unique_temp_dir("duplicate_parameters");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run(a: String, a: Int) -> String { a }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate parameters"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("duplicate parameter `a` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_interface_reference() {
    let root = unique_temp_dir("ambiguous_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/first.dag"),
        "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/second.dag"),
        "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nservice FsStorage implements Storage { operation read(path: String) -> { body: String } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("ambiguous interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_resource_interface_reference() {
    let root = unique_temp_dir("ambiguous_resource_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/first.dag"),
        "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/second.dag"),
        "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nresource Disk implements Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous resource interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("`Disk` references ambiguous interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_interface_reference() {
    let root = unique_temp_dir("unresolved_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nservice FsStorage implements MissingStorage { operation read(path: String) -> { body: String } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("references unresolved interface `MissingStorage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_resource_interface_reference() {
    let root = unique_temp_dir("unresolved_resource_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nresource Disk implements MissingStorage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved resource interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("`Disk` references unresolved interface `MissingStorage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_uses_resource_type() {
    let root = unique_temp_dir("unknown_uses_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } uses fs: MissingResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unknown uses resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("unknown used resource type `MissingResource`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_uses_resource_type() {
    let root = unique_temp_dir("ambiguous_uses_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous uses resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("ambiguous used resource type `SharedResource`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_provides_resource_type() {
    let root = unique_temp_dir("unknown_provides_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } provides out: MissingResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unknown provides resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("unknown provided resource type `MissingResource`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_provides_resource_type() {
    let root = unique_temp_dir("ambiguous_provides_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous provides resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("ambiguous provided resource type `SharedResource`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_reports_call_arity_typecheck_error() {
    let fixture = unique_temp_file("call_arity");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("call arity mismatch"));
    assert!(stderr.contains("fmt"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_unknown_named_call_argument_typecheck_error() {
    let fixture = unique_temp_file("call_unknown_arg");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on unknown named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("unknown named argument `text`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_service_call_arity_typecheck_error() {
    let fixture = unique_temp_file("service_call_arity");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read()
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on service call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("service call arity mismatch"));
    assert!(stderr.contains("FsStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_unknown_named_service_argument_typecheck_error() {
    let fixture = unique_temp_file("service_call_unknown_arg");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(file: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on unknown named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("unknown named argument `file`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_duplicate_named_call_argument_typecheck_error() {
    let fixture = unique_temp_file("duplicate_named_call_arg");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String, mode: String) -> String { value }
fn run() -> String {
  fmt(value: "a", value: "b")
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on duplicate named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("duplicate named argument `value`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_duplicate_named_service_argument_typecheck_error() {
    let fixture = unique_temp_file("duplicate_named_service_arg");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String, mode: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String, mode: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path, path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on duplicate named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("duplicate named argument `path`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_service_call() {
    let root = unique_temp_dir("unresolved_service_call_typecheck");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved service call"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("unresolved service call `MissingStorage.read`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_service_call() {
    let root = unique_temp_dir("ambiguous_service_call_typecheck");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/first.dag"),
        r#"module sample.first
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
    )
    .expect("failed to write first service source");
    std::fs::write(
        root.join("sample/second.dag"),
        r#"module sample.second
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
    )
    .expect("failed to write second service source");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run(path: String) -> { body: String } {
  let response = SharedService.read(path: path)
  return { body: response.body }
}"#,
    )
    .expect("failed to write main source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous service call"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("ambiguous service call `SharedService.read`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_callable_target() {
    let root = unique_temp_dir("ambiguous_callable_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nfn render(value: String) -> String { value }",
    )
    .expect("failed to write first callable source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nfn render(value: String) -> String { value }",
    )
    .expect("failed to write second callable source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { render(value: \"ok\") }",
    )
    .expect("failed to write main source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous callable target"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("ambiguous call target `render`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_callable_target() {
    let root = unique_temp_dir("unresolved_callable_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { missing(value: \"ok\") }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved callable target"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("typecheck errors"));
    assert!(stderr.contains("unresolved call target `missing`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_reports_module_path_mismatch() {
    let root = unique_temp_dir("path_mismatch");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module wrong.name\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on mismatch directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on module path mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}
