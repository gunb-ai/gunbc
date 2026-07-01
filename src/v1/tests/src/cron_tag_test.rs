use std::fs;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively, workspace_root};

const CRON_TAG_DAG: &str = "dsl/gunbc/tools/cron_tag.dag";
const WITNESS_FN: &str = "cron_tag_upsert_protocol_keystone_holds";

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph for {CRON_TAG_DAG}, got diagnostics {msgs:?} \
         (graph present: {})",
        result.graph.is_some()
    );
}

fn cron_tag_sources() -> Vec<Rc<v1_compiler::v1_compiler_compile::SourceFile>> {
    let path = workspace_root().join(CRON_TAG_DAG);
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    resolve_imports_transitively(CRON_TAG_DAG, &content)
}

fn run_witness(sources: Vec<Rc<v1_compiler::v1_compiler_compile::SourceFile>>) -> Value {
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved.graph.as_ref().expect("resolved graph");
    v1_interpreter::run(graph, resolved.source_indices.clone(), WITNESS_FN)
        .unwrap_or_else(|e| panic!("run {WITNESS_FN}: {e:?}"))
}

#[test]
fn cron_tag_upsert_protocol_keystone_holds_via_interpreter() {
    match run_witness(cron_tag_sources()) {
        Value::Bool(true) => {}
        other => panic!("cron_tag upsert protocol regressed: {WITNESS_FN} returned {other:?}"),
    }
}

#[test]
fn cron_tag_upsert_protocol_witness_discriminates_on_mutation() {
    let path = workspace_root().join(CRON_TAG_DAG);
    let mut content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    content = content.replace("got == want", "got != want");
    let sources = resolve_imports_transitively(CRON_TAG_DAG, &content);
    match run_witness(sources) {
        Value::Bool(false) => {}
        other => {
            panic!("expected mutated witness to return false (non-vacuous green), got {other:?}")
        }
    }
}
