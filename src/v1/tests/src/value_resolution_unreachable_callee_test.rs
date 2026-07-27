//! Discriminating value-resolution witnesses. Every type in the RED fixture is
//! structurally or explicitly bound; only the bare callee is off-chain.

use crate::helpers::{compile_dag_named_with_source_roots, diagnostic_messages, workspace_root};
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_std_core::is_error_diagnostic;

const FIXTURE_DIR: &str = "dag/test/fixtures/value_resolution_unreachable_callee";
const UNREACHABLE_CALLEE: &str = "fixture_callee_read_typed";

fn fixture_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("dag"), ws.join(FIXTURE_DIR)]
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(workspace_root().join(FIXTURE_DIR).join(name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

fn compile_probe(
    entry_name: &str,
) -> std::rc::Rc<v1_compiler::v1_compiler_compile::PipelineResult> {
    let entry_path = format!("{FIXTURE_DIR}/{entry_name}");
    compile_dag_named_with_source_roots(
        &entry_path,
        &read_fixture(entry_name),
        RenderTarget::Dag,
        &fixture_roots(),
    )
}

fn hard_messages(
    result: &std::rc::Rc<v1_compiler::v1_compiler_compile::PipelineResult>,
) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

#[test]
fn qualified_callee_control_compiles_clean() {
    let result = compile_probe("probe_value_green.dag");
    assert!(
        hard_messages(&result).is_empty(),
        "qualified callee control must compile clean; got {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn pool_only_callee_refuses_at_the_call_occurrence() {
    let result = compile_probe("probe_value_arm.dag");
    let hard = hard_messages(&result);
    assert!(
        hard.iter().any(|message| {
            message.contains(UNREACHABLE_CALLEE)
                && (message.contains("unbound") || message.contains("not found in scope"))
        }),
        "off-chain callee must produce a located hard refusal; got {:?}",
        diagnostic_messages(&result)
    );
    assert!(
        !hard
            .iter()
            .any(|message| message.contains("Primitive(fixture_callee_read_typed)")),
        "callee refusal must not fabricate a primitive signature; got {hard:?}"
    );
}
