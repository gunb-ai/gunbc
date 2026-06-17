//! R2 add-emit keystone — always-running discriminating guard.
//!
//! This is the durable guard for the R2 keystone ("gunbc emits correct Rust end-to-end",
//! #4462): `emit(add)` must produce EXACTLY `fn add(x: i32, y: i32) -> i32 { x + y }` with
//! no diagnostics. It executes the v2 witness `mvp1_rust_emit_add_fn_accepts_holds` through
//! the real v2 compile+interpret pipeline (the same path as `gunbc run --claim-run`) — NOT
//! a parse/structure probe, so a regression in the emit/translate/grammar substrate turns
//! it RED. It is the standing sentinel for B5-style emit-consumer regressions (exactly the
//! #4484 grammar-inverse break that shipped undetected because the only execution cert lived
//! in a corpus-eval-gated roster).
//!
//! **Always-runs without a new CI step.** CI executes the `v1-compiler-tests` crate only via
//! ONE `--exact` invocation — the `pipeline::dag_emit_from_resolved_matches_compile_sources_
//! for_v4_slice` parity receipt in `ci_floor` (runs on every non-draft PR). A standalone
//! `#[test]` here would therefore be DORMANT (never selected). So these are `pub` helpers,
//! NOT `#[test]`s, invoked by that always-on parity test — the guard rides an existing
//! always-on path with zero `ci.yml` change (mgmt CI-policy ruling 2026-06-07: prefer an
//! existing always-on home over adding a CI step). See [[project_v2_tests_not_run_broadly_in_ci]].
//!
//! Discrimination is proven member-wise: `assert_keystone_green` asserts `true` on the real
//! substrate; `assert_keystone_discriminates_on_mutation` mutates the pinned expected source
//! text and asserts the whole-text equality flips the witness to `false` (non-vacuous green).
//! [[feedback_no_ratchets_promote_greens_to_tests]]

use std::rc::Rc;
use std::sync::OnceLock;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CERT_ENTRY: &str = "src/v2/compiler/manual/mvp1_rust_add_translate_test.dag";
const WITNESS_FN: &str = "mvp1_rust_emit_add_fn_accepts_holds";

/// The exact canonical Rust source the R2 keystone pins (rust.dag:rust_mvp1_source_text).
const PINNED_ADD_SOURCE: &str = "fn add(x: i32, y: i32) -> i32 { x + y }";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v2")]
}

/// The cert's transitive v2 closure as owned `(path, content)` pairs, resolved once and shared
/// by both tests via a process-wide cache. The module-index scan over src/v2 is the dominant
/// cost; resolving it once (rather than per-test) roughly halves the step's wall-clock. Owned
/// Strings (not `Rc<SourceFile>`, which is !Sync) so the value can live in a `OnceLock`.
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

/// Resolve the cert's transitive v2 closure (same set `gunbc run --source-root src/v2` builds).
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

/// Run both members of the R2 keystone guard. Invoked by the always-on parity test
/// (`pipeline::dag_emit_from_resolved_matches_compile_sources_for_v4_slice`) so the guard
/// runs on every non-draft PR without adding a CI step.
pub fn assert_r2_emit_add_keystone() {
    assert_keystone_green();
    assert_keystone_discriminates_on_mutation();
}

/// POSITIVE — the keystone holds on the real substrate: emit(add) == the pinned source.
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

/// NEGATIVE (discrimination) — mutate the pinned expected source text; the whole-text
/// equality must flip the witness to `false`. Proves the green is non-vacuous: any drift
/// between emit output and the pinned text reds the keystone.
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
