use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const ENTRY: &str = "src/v2/test/claim/manual/pbp_body_producer_perf_repro.dag";

const WRONG_TYPE_SRC: &str = r#"module v2.test.manual.pbp_body_producer_wrong_type_repro

import v2.compiler.body_producer { produce_mvp1_add_arrow_with_body_from_resolved }
import v2.std.logic { Bool }
import v2.std.integer { Int }

fn pbp_wrong(cond: Bool) -> Int {
  if cond { 1 } else { true }
}
"#;

fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v2"), ws.join("dsl")]
}

fn sources_for(entry: &str) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let content = std::fs::read_to_string(ws.join(entry)).expect("read entry");
    resolve_imports_transitively_with_source_roots(entry, &content, &source_roots())
}

fn sources_for_inline(entry: &str, content: &str) -> Vec<Rc<SourceFile>> {
    resolve_imports_transitively_with_source_roots(entry, content, &source_roots())
}

fn non_complexity_errors(
    resolved: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect()
}

#[test]
fn body_producer_infer_perf_witness_resolves_clean() {
    let sources = sources_for(ENTRY);
    let resolved = compile_to_resolved(Rc::new(sources));
    let errs = non_complexity_errors(&resolved);
    assert!(
        errs.is_empty() && resolved.graph.is_some(),
        "body_producer closure must resolve cleanly, got errors {errs:?}"
    );
}

#[test]
fn body_producer_infer_perf_witness_wrong_type_still_rejects() {
    let sources = sources_for_inline(
        "src/v2/test/claim/manual/pbp_body_producer_wrong_type_repro.dag",
        WRONG_TYPE_SRC,
    );
    let resolved = compile_to_resolved(Rc::new(sources));
    let errs = non_complexity_errors(&resolved);
    assert!(
        !errs.is_empty(),
        "embedded wrong-type repro must produce a real inference/type diagnostic, got errs={errs:?} graph={}",
        resolved.graph.is_some()
    );
}
