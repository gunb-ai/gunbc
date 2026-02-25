//! Shared helpers used by both Makefile and Justfile renderers.
//!
//! These are the common workflow/meta/tool rendering primitives that compute
//! dependency lists and command bodies from the registry model.

use std::borrow::Cow;

use crate::makegen::registry::{BuildConfig, MetaTarget, ResourceTargetMap, ToolInfo, WorkflowSpec};
use crate::WorkspaceBinary;
use gunbc_ir::cargo::{CargoCommand, CargoInvocation, Subcommand};
use gunbc_ir::resource::ExecMode;

/// Produce a comment string for a core workflow.
///
/// The "build" workflow gets special treatment to describe the full transaction.
pub(crate) fn core_workflow_comment(workflow: &WorkflowSpec, config: &BuildConfig) -> String {
    if workflow.name == "build" {
        let build_desc = if config.use_dag_entrypoints {
            "codegen \u{2192} testgen \u{2192} gunbc-build"
        } else {
            "codegen \u{2192} testgen \u{2192} cargo build"
        };
        return format!("Full build transaction: {build_desc}");
    }
    workflow.description.clone()
}

/// Produce the body commands for a core workflow.
pub(crate) fn core_workflow_body(
    workflow: &WorkflowSpec,
    config: &BuildConfig,
) -> Vec<Cow<'static, str>> {
    match workflow.name.as_str() {
        "preflight-fix" => {
            vec!["@cargo fix --workspace --all-targets --allow-dirty --allow-staged".into()]
        }
        "ensure-codegen" => vec![config.ensure_codegen.shell().into()],
        "build-release-bins" => {
            vec!["@RUSTFLAGS=\"-D warnings\" cargo build --workspace --release --bins".into()]
        }
        "lint-upsert" => {
            let lint_cmd = config.lint.to_shell();
            let lint_fix_cmd = config.lint_fix.to_shell();
            let lint_upsert = format!("@{} || ({} && {})", lint_cmd, lint_fix_cmd, lint_cmd);
            vec![config.pragma.shell().into(), lint_upsert.into()]
        }
        "codegen" => vec![config.codegen.shell().into()],
        "build" => vec![config.build.shell().into()],
        "clean" => vec!["@cargo clean".into()],
        "testgen" => vec![config.testgen.shell().into()],
        "testgen-check" => vec![config.testgen.shell().into()],
        "deps-config" => vec![format!(
            "@target/release/{} --mode=ensure",
            WorkspaceBinary::DepsConfig.invocation().binary
        )
        .into()],
        "deps-config-check" => vec![format!(
            "@target/release/{} --mode=verify",
            WorkspaceBinary::DepsConfig.invocation().binary
        )
        .into()],
        "makegen-check" => vec![config.makegen.shell().into()],
        "bootstrap-check" => vec![config.bootstrap.shell().into()],
        "pragma-check" => vec![config.pragma.shell().into()],
        "verify" => vec![
            "@$(MAKE) deps-config-check".into(),
            "@$(MAKE) makegen-check".into(),
            "@$(MAKE) bootstrap-check".into(),
            "@$(MAKE) testgen-check".into(),
            "@$(MAKE) pragma-check".into(),
        ],
        "verify-fix" => vec![
            "@$(MAKE) deps-config".into(),
            config.makegen.shell().into(),
            config.bootstrap.shell().into(),
            config.testgen.shell().into(),
            config.pragma.shell().into(),
        ],
        "fmt-fix" => vec![config.fmt.shell().into()],
        "lint-fix" => vec![config.lint_fix.shell().into()],
        "ci" => vec![workflow_planner_command("ci", config).into()],
        "test-all" => {
            vec![workflow_planner_command("test-all", config).into()]
        }
        _ => panic!(
            "missing core workflow body renderer for '{}'",
            workflow.name
        ),
    }
}

/// Build the dependency list for a meta target (base/check variant).
pub(crate) fn meta_target_deps(
    meta: &MetaTarget,
    res_map: &ResourceTargetMap,
) -> Vec<Cow<'static, str>> {
    meta.workflow_spec(res_map)
        .deps
        .into_iter()
        .map(Cow::Owned)
        .collect()
}

/// Build the dependency list for a tool target.
pub(crate) fn tool_target_deps(tool: &ToolInfo, config: &BuildConfig) -> Vec<Cow<'static, str>> {
    tool.workflow_spec(config)
        .deps
        .into_iter()
        .map(Cow::Owned)
        .collect()
}

fn workflow_planner_command(name: &str, config: &BuildConfig) -> String {
    let workflow_inv = CargoInvocation::composed("workflow", "dag");
    let cmd = CargoCommand::new(Subcommand::Run(workflow_inv))
        .quiet()
        .release()
        .warnings(config.warnings);
    format!("@{} -- {name}", cmd.to_shell_with_env())
}
