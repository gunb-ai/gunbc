//! BT9: Testgen integration — verify SDLC DAGs are auto-discoverable.
//!
//! Validates that SDLC DAG modules are discovered by the testgen
//! auto-discovery pipeline and that auto-testgen produces test code.
//!
//! Testing level: L6 (testgen)
//! Profile: none (auto-discovery doesn't require a profile)

#![allow(clippy::disallowed_methods)]

use gunbc_dag::testgen_dag::dag_test_discovery::{
    auto_testgen_for_module, discover_compilable_modules, AutoTestgenResult,
};

/// SDLC func modules are discovered by testgen auto-discovery.
#[test]
fn sdlc_modules_are_auto_discovered() {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .expect("workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let modules = discover_compilable_modules(&dsl_root);

    let module_names: Vec<&str> = modules.iter().map(|m| m.module_name.as_str()).collect();

    // All 4 SDLC func modules should be discovered
    assert!(
        module_names.contains(&"funcs.sdlc_stages"),
        "missing funcs.sdlc_stages. Got: {module_names:?}"
    );
    assert!(
        module_names.contains(&"funcs.sdlc_worker"),
        "missing funcs.sdlc_worker. Got: {module_names:?}"
    );
    assert!(
        module_names.contains(&"funcs.sdlc_dispatch_runtime"),
        "missing funcs.sdlc_dispatch_runtime. Got: {module_names:?}"
    );
    assert!(
        module_names.contains(&"funcs.sdlc_validation_runtime"),
        "missing funcs.sdlc_validation_runtime. Got: {module_names:?}"
    );
}

/// SDLC func modules with interfaces are flagged as requiring profile.
#[test]
fn sdlc_modules_with_interfaces_require_profile() {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .expect("workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let modules = discover_compilable_modules(&dsl_root);

    let worker = modules
        .iter()
        .find(|m| m.module_name == "funcs.sdlc_worker")
        .expect("sdlc_worker should be discovered");

    // Worker imports interfaces (IssueProvider, ClaimStore, etc.)
    assert!(
        worker.requires_profile,
        "sdlc_worker should require profile (uses interfaces)"
    );
    assert!(
        !worker.interface_imports.is_empty(),
        "sdlc_worker should have interface imports"
    );
}

/// Auto-testgen generates test code for SDLC dispatch runtime.
///
/// We pick sdlc_dispatch_runtime because it's self-contained (no interface
/// imports, compiles without profile).
#[test]
fn sdlc_dispatch_runtime_generates_tests() {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .expect("workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let modules = discover_compilable_modules(&dsl_root);

    let dispatch_rt = modules
        .iter()
        .find(|m| m.module_name == "funcs.sdlc_dispatch_runtime")
        .expect("sdlc_dispatch_runtime should be discovered");

    let output_dir = std::path::Path::new("gunbc-dag/src");
    let result = auto_testgen_for_module(dispatch_rt, output_dir);

    match result {
        AutoTestgenResult::Generated { test_code, .. } => {
            assert!(
                test_code.contains("#[test]"),
                "generated code should contain test functions"
            );
            // Should generate boundary tests for the dispatch functions
            assert!(
                test_code.len() > 1000,
                "generated code should be substantial. Got {} bytes",
                test_code.len()
            );
        }
        AutoTestgenResult::Skipped { reason } => {
            panic!("sdlc_dispatch_runtime should compile, but got: {reason}");
        }
    }
}

/// Auto-testgen generates test code for SDLC validation runtime.
#[test]
fn sdlc_validation_runtime_generates_tests() {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .expect("workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let modules = discover_compilable_modules(&dsl_root);

    let validation_rt = modules
        .iter()
        .find(|m| m.module_name == "funcs.sdlc_validation_runtime")
        .expect("sdlc_validation_runtime should be discovered");

    let output_dir = std::path::Path::new("gunbc-dag/src");
    let result = auto_testgen_for_module(validation_rt, output_dir);

    match result {
        AutoTestgenResult::Generated { test_code, .. } => {
            assert!(
                test_code.contains("#[test]"),
                "generated code should contain test functions"
            );
        }
        AutoTestgenResult::Skipped { reason } => {
            panic!("sdlc_validation_runtime should compile, but got: {reason}");
        }
    }
}
