//! Inventory registration for resource-backed DSL tools.
//!
//! These marker functions exist only to host `resource_test_target` metadata.
//! They are intentionally non-public and do not create app-layer wrapper APIs.

#[allow(dead_code)]
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "build",
    builder = "gunbc_resolve::builder::build_dsl_graph_dag(\"tools/build.dag\", crate::extern_ops::gunbc_runtime_bindings(), gunbc_resolve::BuildOpts { entry_func: Some(\"build_all\"), profile: None })",
    returns_result
)]
fn register_build_resource_target() {}

#[allow(dead_code)]
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "codegen",
    builder = "gunbc_resolve::builder::build_dsl_graph_dag(\"tools/codegen.dag\", crate::extern_ops::gunbc_runtime_bindings(), gunbc_resolve::BuildOpts { entry_func: Some(\"codegen\"), profile: None })",
    returns_result
)]
fn register_codegen_resource_target() {}

#[allow(dead_code)]
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "infra",
    builder = "gunbc_resolve::builder::build_dsl_graph_dag(\"tools/infra.dag\", crate::extern_ops::gunbc_runtime_bindings(), gunbc_resolve::BuildOpts { entry_func: Some(\"infra\"), profile: None })",
    returns_result
)]
fn register_infra_resource_target() {}

#[allow(dead_code)]
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "docgen",
    builder = "gunbc_resolve::builder::build_dsl_graph_dag(\"tools/docgen.dag\", crate::extern_ops::gunbc_runtime_bindings(), gunbc_resolve::BuildOpts::default())",
    returns_result
)]
fn register_docgen_resource_target() {}

#[allow(dead_code)]
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "readme",
    builder = "gunbc_resolve::builder::build_dsl_graph_dag(\"tools/readme.dag\", crate::extern_ops::gunbc_runtime_bindings(), gunbc_resolve::BuildOpts { entry_func: Some(\"readme\"), profile: None })",
    returns_result
)]
fn register_readme_resource_target() {}
