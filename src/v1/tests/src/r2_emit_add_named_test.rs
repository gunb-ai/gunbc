use std::rc::Rc;
use std::sync::OnceLock;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CERT_ENTRY: &str = "src/v2/test/claim/manual/rust_add_emit_translate_test.dag";
const WITNESS_FN: &str = "rust_add_emit_add_fn_accepts_holds";

const PINNED_ADD_SOURCE: &str = "fn add(x: i32, y: i32) -> i32 { x + y }";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn cert_source_pairs() -> &'static Vec<(String, String)> {
    static CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let entry_content = std::fs::read_to_string(workspace_root().join(CERT_ENTRY))
            .unwrap_or_else(|e| panic!("read {CERT_ENTRY}: {e}"));
        resolve_imports_transitively_with_source_roots(
            CERT_ENTRY,
            &entry_content,
            &v2_source_roots(),
        )
        .iter()
        .map(|s| (s.path.clone(), s.content.clone()))
        .collect()
    })
}

fn cert_sources() -> Vec<Rc<SourceFile>> {
    cert_source_pairs()
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect()
}

fn assert_resolved_ok(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected clean resolved graph for {CERT_ENTRY}, got diagnostics {msgs:?} \
         (graph present: {})",
        resolved.graph.is_some()
    );
}

fn run_witness(sources: Vec<Rc<SourceFile>>) -> Value {
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_ok(&resolved);
    let graph = resolved.graph.as_ref().expect("resolved graph");
    v1_interpreter::run(graph, resolved.source_indices.clone(), WITNESS_FN)
        .unwrap_or_else(|e| panic!("run {WITNESS_FN}: {e:?}"))
}

pub fn assert_r2_emit_add_keystone() {
    assert_keystone_green();
    assert_keystone_discriminates_on_mutation();
}

pub fn assert_keystone_green() {
    match run_witness(cert_sources()) {
        Value::Bool(true) => {}
        other => panic!(
            "R2 keystone regressed: emit(add) must produce exactly `{PINNED_ADD_SOURCE}` \
             with no diagnostics, but {WITNESS_FN} returned {other:?}. This is a real \
             emit/translate/grammar substrate regression (cf. #4484)."
        ),
    }
}

pub fn assert_keystone_discriminates_on_mutation() {
    let mut sources = cert_sources();
    let mutant = "fn add(x: i64, y: i64) -> i64 { x + y }";
    let mut mutated = false;
    for src in sources.iter_mut() {
        if src.content.contains(PINNED_ADD_SOURCE) {
            let new_content = src.content.replace(PINNED_ADD_SOURCE, mutant);
            *src = Rc::new(SourceFile {
                path: src.path.clone(),
                content: new_content,
            });
            mutated = true;
        }
    }
    assert!(
        mutated,
        "discrimination setup failed: pinned source `{PINNED_ADD_SOURCE}` not found in the \
         cert's source closure (did rust_mvp1_source_text move/change?)"
    );

    match run_witness(sources) {
        Value::Bool(false) => {}
        other => panic!(
            "R2 keystone is NOT discriminating: after mutating the pinned expected source to \
             `{mutant}`, {WITNESS_FN} returned {other:?} (expected false). A non-discriminating \
             green proves nothing."
        ),
    }
}
