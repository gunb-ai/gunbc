//! §5 execution witnesses: body_producer closure resolves cleanly AND inference stays fail-closed.
//! Structural complexity-lens guards are in follow-on #5139 (not this fix-only PR).
//!
//! Run: cargo test -p v1-compiler-tests body_producer_infer_perf_witness -- --nocapture

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const ENTRY: &str = "src/v2/compiler/manual/pbp_body_producer_perf_repro.dag";
const WRONG_TYPE_ENTRY: &str = "src/v2/compiler/manual/pbp_body_producer_wrong_type_repro.dag";

fn sources_for(entry: &str) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let content = std::fs::read_to_string(ws.join(entry)).expect("read entry");
    resolve_imports_transitively_with_source_roots(
        entry,
        &content,
        &[ws.join("src/v2"), ws.join("dsl")],
    )
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
    let sources = sources_for(WRONG_TYPE_ENTRY);
    let resolved = compile_to_resolved(Rc::new(sources));
    let errs = non_complexity_errors(&resolved);
    assert!(
        !errs.is_empty() || resolved.graph.is_none(),
        "wrong-type repro must fail inference/resolve (fail-closed), got clean graph; errs={errs:?}"
    );
}

#[test]
fn debug_module_name_match_vs_mismatch() {
    let body = r#"
import v2.compiler.body_producer { produce_mvp1_add_arrow_with_body_from_resolved }

type Refined<T> {
  base: T
}
type UserId = Refined<String>
type AccountId = Refined<String>

fn take_acct(acct: AccountId) -> String {
  ""
}

fn caller(uid: UserId) -> String {
  let x = uid
  take_acct(x)
}

fn pbp_wrong() -> String {
  caller(uid: Refined { base: "" })
}
"#;
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dsl")];
    for (entry, module_name) in [
        (
            WRONG_TYPE_ENTRY,
            "v2.test.manual.pbp_body_producer_wrong_type_repro",
        ),
        ("pd3adv.dag", "pd3adv.twin_let"),
    ] {
        let source = format!("module {module_name}\n{body}");
        let resolved = compile_to_resolved(Rc::new(
            resolve_imports_transitively_with_source_roots(entry, &source, &roots),
        ));
        eprintln!(
            "module={module_name} entry={entry} errs={:?} graph={}",
            non_complexity_errors(&resolved),
            resolved.graph.is_some()
        );
    }
}
