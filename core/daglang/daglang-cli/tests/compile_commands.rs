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

fn assert_typecheck_stage_failure(stderr: &str) {
    assert!(stderr.contains("typecheck errors"));
    assert!(
        !stderr.contains("lower error"),
        "expected failure to remain in typecheck stage: {stderr}"
    );
}

fn assert_lower_stage_failure(stderr: &str) {
    assert!(stderr.contains("lower error"));
    assert!(
        !stderr.contains("typecheck errors"),
        "expected failure to remain in lowering stage: {stderr}"
    );
}

fn assert_no_stage_failures(stderr: &str) {
    assert!(
        !stderr.contains("typecheck errors"),
        "did not expect typecheck-stage banner in successful path: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "did not expect lowering-stage banner in successful path: {stderr}"
    );
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));
    assert!(stdout.contains("target/generated/rust/main.rs"));
}

#[test]
fn compile_command_single_file_fails_on_duplicate_definition() {
    let fixture = unique_temp_file("compile_single_file_duplicate_definition");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run() -> String { "a" }
fn run() -> String { "b" }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-definition fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `run` in module `sample.single`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_callable_does_not_report_ambiguous_call_target() {
    let fixture = unique_temp_file("compile_single_file_duplicate_callable_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-callable fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate callable definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `helper` in module `sample.single`"));
    assert!(
        !stderr.contains("ambiguous call target `helper`"),
        "single-file relaxed mode should not report ambiguous call-target diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-callable path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_service_does_not_report_ambiguous_service_call() {
    let fixture = unique_temp_file("compile_single_file_duplicate_service_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
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
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-service fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate service definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `FsStorage` in module `sample.single`"));
    assert!(
        !stderr.contains("ambiguous service call `FsStorage.read`"),
        "single-file relaxed mode should not report ambiguous service-call diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-service path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_parameter() {
    let fixture = unique_temp_file("compile_single_file_duplicate_parameter");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run(a: String, a: Int) -> String { a }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-parameter fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate parameter"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate parameter `a` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_output_field() {
    let fixture = unique_temp_file("compile_single_file_duplicate_output_field");
    std::fs::write(
        &fixture,
        r#"module sample.single
func run() -> { ok: Bool, ok: String } { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-output fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate output field"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate output field `ok` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_uses_binding() {
    let fixture = unique_temp_file("compile_single_file_duplicate_uses_binding");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-uses fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate uses binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate uses binding `fs` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_resource_uses_does_not_report_ambiguous_used_type() {
    let fixture = unique_temp_file("compile_single_file_duplicate_resource_uses_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-resource-uses fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate resource definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains(
        "duplicate definition `SharedResource` in module `sample.single`"
    ));
    assert!(
        !stderr.contains("ambiguous used resource type `SharedResource`"),
        "single-file relaxed mode should suppress ambiguous used resource diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-resource uses path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_provides_binding() {
    let fixture = unique_temp_file("compile_single_file_duplicate_provides_binding");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } provides out: Storage provides out: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-provides fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate provides binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate provides binding `out` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_resource_provides_does_not_report_ambiguous_provided_type() {
    let fixture = unique_temp_file("compile_single_file_duplicate_resource_provides_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-resource-provides fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate resource definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains(
        "duplicate definition `SharedResource` in module `sample.single`"
    ));
    assert!(
        !stderr.contains("ambiguous provided resource type `SharedResource`"),
        "single-file relaxed mode should suppress ambiguous provided resource diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-resource provides path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_use_provide_binding_conflict() {
    let fixture = unique_temp_file("compile_single_file_use_provide_binding_conflict");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses io: Storage provides io: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for use/provide-conflict fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on use/provide binding conflict"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("binding `io` is declared in both uses/provides in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_missing_resource_capability() {
    let fixture = unique_temp_file("compile_single_file_missing_resource_capability");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for missing-resource-capability fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on missing resource capability"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_missing_service_operation() {
    let fixture = unique_temp_file("compile_single_file_missing_service_operation");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for missing-service-operation fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on missing service operation"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_interface_signature_mismatch() {
    let fixture = unique_temp_file("compile_single_file_interface_signature_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for service-signature-mismatch fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on service signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`FsStorage` does not match `Storage.read` contract"));
    assert!(stderr.contains("expected `String` but found `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_resource_interface_signature_mismatch() {
    let fixture = unique_temp_file("compile_single_file_resource_signature_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for resource-signature-mismatch fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on resource signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` does not match `Storage.read` contract"));
    assert!(stderr.contains("expected `String` but found `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
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
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved service call"));
    assert!(stderr.contains("MissingStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_uses_reports_lower_error() {
    let fixture = unique_temp_file("compile_unresolved_uses_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.uses
func run() -> { ok: Bool } uses fs: MissingResource {
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
        .expect("failed to run daglang compile for unresolved uses fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for unresolved uses binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved used resource"));
    assert!(stderr.contains("fs: MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_provides_reports_lower_error() {
    let fixture = unique_temp_file("compile_unresolved_provides_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.provides
func run() -> { ok: Bool } provides out: MissingResource {
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
        .expect("failed to run daglang compile for unresolved provides fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for unresolved provides binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved provided resource"));
    assert!(stderr.contains("out: MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_allows_unresolved_imports() {
    let fixture = unique_temp_file("compile_single_file_unresolved_import");
    std::fs::write(
        &fixture,
        r#"module sample.single
import missing.dep
fn run() -> Unit {}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved import fixture");

    assert!(
        output.status.success(),
        "single-file compile should tolerate unresolved imports: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "relaxed unresolved-import path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "relaxed unresolved-import path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_allows_unit_return_without_tail_expression() {
    let fixture = unique_temp_file("compile_single_file_unit_missing_tail");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run() -> Unit { let x = 42 }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unit-missing-tail fixture");

    assert!(
        output.status.success(),
        "single-file compile should allow missing tail for Unit return: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "Unit-return success path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "Unit-return success path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_allows_unresolved_call_targets() {
    let fixture = unique_temp_file("compile_single_file_unresolved_call_target");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn run() -> String { missing(value: "ok") }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved call-target fixture");

    assert!(
        output.status.success(),
        "single-file compile should tolerate unresolved call targets: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "relaxed unresolved-call path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "relaxed unresolved-call path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_service_interface_reports_typecheck_error() {
    let fixture = unique_temp_file("compile_single_file_unresolved_service_interface");
    std::fs::write(
        &fixture,
        r#"module sample.services
service FsStorage implements MissingStorage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved service-interface fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on unresolved service interface"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`FsStorage` references unresolved interface `MissingStorage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_resource_interface_reports_typecheck_error() {
    let fixture = unique_temp_file("compile_single_file_unresolved_resource_interface");
    std::fs::write(
        &fixture,
        r#"module sample.resources
resource Disk implements MissingStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved resource-interface fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on unresolved resource interface"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` references unresolved interface `MissingStorage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_interface_reports_ambiguous_implements() {
    let fixture = unique_temp_file("compile_single_file_duplicate_interface_ambiguous_impl");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-interface fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for duplicate interface definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `Storage` in module `sample.single`"));
    assert!(stderr.contains("`FsStorage` references ambiguous interface `Storage`"));

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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}

#[test]
fn expand_command_reports_diagnostics_for_invalid_file() {
    let broken = unique_temp_file("expand_broken");
    std::fs::write(&broken, "module sample.broken\nfn broken( -> String {")
        .expect("failed to write broken source");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&broken)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand for broken file");

    assert!(!output.status.success(), "broken source should fail expand");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}

#[test]
fn manifest_command_reports_diagnostics_for_invalid_file() {
    let broken = unique_temp_file("manifest_broken");
    std::fs::write(&broken, "module sample.broken\nfn broken( -> String {")
        .expect("failed to write broken source");

    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(&broken)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest for broken file");

    assert!(!output.status.success(), "broken source should fail manifest");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    assert_lower_stage_failure(&stderr);
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
    assert_lower_stage_failure(&stderr);
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
    assert_lower_stage_failure(&stderr);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved import"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved imports should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `run` in module `sample.main`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_interface_also_reports_ambiguous_implements() {
    let root = unique_temp_dir("duplicate_interface_ambiguous_implements");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read { input { path: String } output { body: String } }
}
interface Storage {
  capability read { input { path: String } output { body: String } }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
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
        "directory compile should fail on duplicate interface definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `Storage` in module `sample.main`"));
    assert!(stderr.contains("`FsStorage` references ambiguous interface `Storage`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-interface layering should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_service_also_reports_ambiguous_service_call() {
    let root = unique_temp_dir("duplicate_service_ambiguous_call");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
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
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate service definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `FsStorage` in module `sample.main`"));
    assert!(stderr.contains("ambiguous service call `FsStorage.read` in `run`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-service layering should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
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
    assert_typecheck_stage_failure(&stderr);
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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous interface `Storage`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous interface references should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` references ambiguous interface `Storage`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous interface references should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("references unresolved interface `MissingStorage`"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved interface references should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` references unresolved interface `MissingStorage`"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved interface references should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_missing_resource_capability() {
    let root = unique_temp_dir("missing_resource_capability");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
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
        "directory compile should fail when resource is missing interface capability"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_missing_service_operation() {
    let root = unique_temp_dir("missing_service_operation");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
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
        "directory compile should fail when service is missing interface operation"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_interface_signature_mismatch() {
    let root = unique_temp_dir("interface_signature_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
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
        "directory compile should fail on interface signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`FsStorage` does not match `Storage.read` contract"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_resource_interface_signature_mismatch() {
    let root = unique_temp_dir("resource_interface_signature_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
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
        "directory compile should fail on resource interface signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` does not match `Storage.read` contract"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_use_provide_binding_conflict() {
    let root = unique_temp_dir("use_provide_binding_conflict");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses io: Storage provides io: Storage {
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
        "directory compile should fail on use/provide binding conflict"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("binding `io` is declared in both uses/provides in `run`"));

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown used resource type `MissingResource`"));
    assert!(
        !stderr.contains("lower error"),
        "unknown uses resource type should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_uses_binding() {
    let root = unique_temp_dir("duplicate_uses_binding");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage {
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
        "directory compile should fail on duplicate uses binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate uses binding `fs` in `run`"));

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous used resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous uses resource types should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_resource_uses_also_reports_ambiguous_used_type() {
    let root = unique_temp_dir("duplicate_resource_uses_ambiguous");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
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
        "directory compile should fail on duplicate resource definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `SharedResource` in module `sample.main`"));
    assert!(stderr.contains("ambiguous used resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-resource uses layering should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_provides_binding() {
    let root = unique_temp_dir("duplicate_provides_binding");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage provides out: Storage {
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
        "directory compile should fail on duplicate provides binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate provides binding `out` in `run`"));

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown provided resource type `MissingResource`"));
    assert!(
        !stderr.contains("lower error"),
        "unknown provides resource type should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous provided resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous provides resource types should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_resource_provides_also_reports_ambiguous_provided_type() {
    let root = unique_temp_dir("duplicate_resource_provides_ambiguous");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
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
        "directory compile should fail on duplicate resource definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `SharedResource` in module `sample.main`"));
    assert!(stderr.contains("ambiguous provided resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-resource provides layering should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown named argument `text`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_undefined_type_typecheck_error() {
    let fixture = unique_temp_file("undefined_type");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(input: MissingType) -> String { "ok" }
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
        "compile should fail on undefined type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("undefined type `MissingType"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_type_mismatch_typecheck_error() {
    let fixture = unique_temp_file("type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { return 42 }
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
        "compile should fail on return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_implicit_return_type_mismatch_typecheck_error() {
    let fixture = unique_temp_file("implicit_return_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { 42 }
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
        "compile should fail on implicit return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_missing_tail_expression_type_mismatch_typecheck_error() {
    let fixture = unique_temp_file("missing_tail_expression_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { let x = 42 }
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
        "compile should fail when fn has no tail expression for non-unit return type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Unit`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_no_such_field_typecheck_error() {
    let fixture = unique_temp_file("no_such_field");
    std::fs::write(
        &fixture,
        r#"module sample.types
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
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
        "compile should fail on missing record field access"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Record` has no field `missing`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_no_such_field_for_named_record_type() {
    let fixture = unique_temp_file("no_such_field_named_record");
    std::fs::write(
        &fixture,
        r#"module sample.types
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
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
        "compile should fail on missing field access for named record type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Payload` has no field `missing`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_unsatisfiable_refinement_typecheck_error() {
    let fixture = unique_temp_file("unsatisfiable_refinement");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(value: Int @range(min: 5, max: 1)) -> Int { value }
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
        "compile should fail on unsatisfiable refinement"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unsatisfiable refinement on `Int`: range min 5 exceeds max 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_generic_arity_mismatch_typecheck_error() {
    let fixture = unique_temp_file("generic_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(values: Map<String>) -> Int { 1 }
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
        "compile should fail on generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Map`: expected 2, got 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_user_defined_generic_arity_mismatch_typecheck_error() {
    let fixture = unique_temp_file("user_defined_generic_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
type Box<T> = T
fn run(values: Box<String, Int>) -> String { values }
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
        "compile should fail on user-defined generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Box`: expected 1, got 2"));

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
    assert_typecheck_stage_failure(&stderr);
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
    assert_typecheck_stage_failure(&stderr);
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
    assert_typecheck_stage_failure(&stderr);
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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate named argument `path`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_directory_mode_fails_on_call_arity_typecheck_error() {
    let root = unique_temp_dir("call_arity");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
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
        "directory compile should fail on call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("call arity mismatch"));
    assert!(stderr.contains("fmt"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_named_call_argument_typecheck_error() {
    let root = unique_temp_dir("call_unknown_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
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
        "directory compile should fail on unknown named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown named argument `text`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_named_call_argument_typecheck_error() {
    let root = unique_temp_dir("duplicate_named_call_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn fmt(value: String, mode: String) -> String { value }
fn run() -> String {
  fmt(value: "a", value: "b")
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
        "directory compile should fail on duplicate named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate named argument `value`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_service_call_arity_typecheck_error() {
    let root = unique_temp_dir("service_call_arity");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
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
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on service call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("service call arity mismatch"));
    assert!(stderr.contains("FsStorage.read"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_named_service_argument_typecheck_error() {
    let root = unique_temp_dir("service_call_unknown_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
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
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unknown named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown named argument `file`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_named_service_argument_typecheck_error() {
    let root = unique_temp_dir("duplicate_named_service_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
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
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate named argument `path`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved service call `MissingStorage.read`"));
    assert!(
        !stderr.contains("lower error"),
        "directory unresolved service call should fail before lowering: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous service call `SharedService.read`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous service calls should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous call target `render`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous callable targets should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_callable_also_reports_ambiguous_call_target() {
    let root = unique_temp_dir("duplicate_callable_ambiguous_call_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
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
        "directory compile should fail on duplicate callable definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `helper` in module `sample.main`"));
    assert!(stderr.contains("ambiguous call target `helper` in `run`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-callable layering should fail in typecheck stage: {stderr}"
    );

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved call target `missing`"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved callable targets should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_type_mismatch() {
    let root = unique_temp_dir("type_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { return 42 }",
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
        "directory compile should fail on return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_implicit_return_type_mismatch() {
    let root = unique_temp_dir("implicit_type_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { 42 }",
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
        "directory compile should fail on implicit return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_allows_unit_return_without_tail_expression() {
    let root = unique_temp_dir("unit_missing_tail");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit { let x = 42 }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should allow missing tail for Unit return: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "Unit-return success path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "Unit-return success path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_missing_tail_expression_type_mismatch() {
    let root = unique_temp_dir("missing_tail_expression_type_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { let x = 42 }",
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
        "directory compile should fail when fn has no tail expression for non-unit return type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Unit`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_no_such_field_for_record_literal() {
    let root = unique_temp_dir("no_such_field_record_literal");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
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
        "directory compile should fail on missing field access for record literal"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Record` has no field `missing`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_no_such_field_for_named_record() {
    let root = unique_temp_dir("no_such_field_named_record");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
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
        "directory compile should fail on missing field for named record type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Payload` has no field `missing`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_generic_arity_mismatch() {
    let root = unique_temp_dir("generic_arity_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn run(values: Map<String>) -> Int { 1 }
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
        "directory compile should fail on generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Map`: expected 2, got 1"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unsatisfiable_refinement() {
    let root = unique_temp_dir("unsatisfiable_refinement");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run(value: Int @range(min: 9, max: 1)) -> Int { value }",
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
        "directory compile should fail on unsatisfiable refinement"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unsatisfiable refinement on `Int`: range min 9 exceeds max 1"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_undefined_type_typecheck_error() {
    let root = unique_temp_dir("undefined_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn run(input: MissingType) -> String { "ok" }
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
        "directory compile should fail on undefined type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("undefined type `MissingType"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_user_defined_generic_arity_mismatch() {
    let root = unique_temp_dir("user_defined_generic_arity");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type Box<T> = T
fn run(value: Box<String, Int>) -> String { value }
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
        "directory compile should fail on user-defined generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Box`: expected 1, got 2"));

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
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn expand_command_directory_mode_reports_module_path_mismatch() {
    let root = unique_temp_dir("expand_path_mismatch");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module wrong.name\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on mismatch directory");

    assert!(
        !output.status.success(),
        "directory expand should fail on module path mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn manifest_command_directory_mode_reports_module_path_mismatch() {
    let root = unique_temp_dir("manifest_path_mismatch");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module wrong.name\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on mismatch directory");

    assert!(
        !output.status.success(),
        "directory manifest should fail on module path mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}
