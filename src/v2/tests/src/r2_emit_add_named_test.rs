//! R2 add-emit keystone — non-gated ordinary cargo test (always runs in CI).
//!
//! This is the durable, always-running discriminating guard for the R2 keystone
//! ("gunbc emits correct Rust end-to-end", #4462): `emit(add)` must produce EXACTLY
//! `fn add(x: i32, y: i32) -> i32 { x + y }` with no diagnostics. It executes the v4
//! witness `mvp1_rust_emit_add_fn_accepts_holds` through the real v2 compile+interpret
//! pipeline (the same path as `gunbc run --claim-run`) — NOT a parse/structure probe,
//! so a regression in the emit/translate/grammar substrate turns it RED.
//!
//! Distinct from the corpus-eval-gated `v4_roster_pilot` row of the same witness: that
//! row only runs when ci_corpus_eval is affected; THIS test always runs. It is the
//! no-ratchet promotion ([[feedback_no_ratchets_promote_greens_to_tests]]) — an
//! ordinary green that is run, not a one-way lock — and the durable guard against
//! B5-style emit-consumer regressions (exactly the #4484 grammar-inverse break that
//! shipped undetected because the only execution cert lived in a gated roster).
//!
//! Discrimination is proven member-wise: the positive test asserts `true` on the real
//! substrate; the negative test mutates the pinned expected source text and asserts the
//! whole-text equality flips the witness to `false` (the green is non-vacuous).

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CERT_ENTRY: &str = "src/v4/test/claim/manual/mvp1_rust_add_translate.dag";
const WITNESS_FN: &str = "mvp1_rust_emit_add_fn_accepts_holds";

/// The exact canonical Rust source the R2 keystone pins (rust.dag:rust_mvp1_source_text).
const PINNED_ADD_SOURCE: &str = "fn add(x: i32, y: i32) -> i32 { x + y }";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

/// Resolve the cert's transitive v4 closure (same set `gunbc run --source-root src/v4` builds).
fn cert_sources() -> Vec<Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(CERT_ENTRY))
        .unwrap_or_else(|e| panic!("read {CERT_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(CERT_ENTRY, &entry_content, &v4_source_roots())
}

fn assert_resolved_ok(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
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
    v2_interpreter::run(graph, resolved.source_indices.clone(), WITNESS_FN)
        .unwrap_or_else(|e| panic!("run {WITNESS_FN}: {e:?}"))
}

/// POSITIVE — the keystone holds on the real substrate: emit(add) == the pinned source.
#[test]
fn r2_emit_add_fn_keystone_is_green() {
    match run_witness(cert_sources()) {
        Value::Bool(true) => {}
        other => panic!(
            "R2 keystone regressed: emit(add) must produce exactly `{PINNED_ADD_SOURCE}` \
             with no diagnostics, but {WITNESS_FN} returned {other:?}. This is a real \
             emit/translate/grammar substrate regression (cf. #4484)."
        ),
    }
}

/// NEGATIVE (discrimination) — mutate the pinned expected source text; the whole-text
/// equality must flip the witness to `false`. Proves the green is non-vacuous: any drift
/// between emit output and the pinned text reds the keystone.
#[test]
fn r2_emit_add_fn_keystone_discriminates_on_mutation() {
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
