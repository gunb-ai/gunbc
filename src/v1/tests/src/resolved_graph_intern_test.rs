//! Layer B (`resolved_graph_intern`) proof-by-execution: the content-addressed
//! cache encoding dedups the per-module import-closure and restores `Rc` sharing
//! on the cache-hit (decode) path. Both properties are asserted deterministically
//! — the size win is a dedup oracle, the pointer-sharing is the cache-hit RSS
//! mechanism (serde `rc` would shatter it).

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use v1_compiler::cli_run::resolve_entry_graph;
use v1_compiler::resolved_graph_cache::serialize_fixture_payload_for_test;
use v1_compiler::resolved_graph_intern::{decode, encode};
use v1_compiler::v1_std_core::NewlineIndex;

fn workspace_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !dir.join("DESIGN.md").exists() {
        if !dir.pop() {
            panic!("could not locate workspace root (DESIGN.md)");
        }
    }
    dir
}

/// A witness with a non-trivial transitive import closure, so multiple modules
/// share the same merged-closure bindings / source-indices / intern-table — the
/// exact duplication Layer B targets.
fn big_closure_graph() -> (
    Rc<v1_compiler::v1_compiler_compile::ResolvedGraph>,
    Rc<HashMap<String, Rc<NewlineIndex>>>,
) {
    let ws = workspace_root();
    let roots = vec![ws.join("dsl").to_string_lossy().into_owned()];
    let entry = ws
        .join("dsl/test/claim/cache_layer_planner_test.dag")
        .to_string_lossy()
        .into_owned();
    resolve_entry_graph(&roots, &entry).expect("resolve big-closure witness")
}

#[test]
fn interned_encoding_dedups_the_import_closure() {
    let (graph, si) = big_closure_graph();

    let plain = serialize_fixture_payload_for_test(&graph, &si).expect("plain encode");
    let interned = serde_json::to_vec(&encode(&graph, &si)).expect("interned encode");

    let module_count = graph.modules.len();
    eprintln!(
        "[layer-b] modules={module_count} plain={} bytes  interned={} bytes  ratio={:.3}",
        plain.len(),
        interned.len(),
        interned.len() as f64 / plain.len() as f64,
    );

    // The closure is duplicated across modules, so interning must shrink the
    // artifact materially. (A multi-module closure dedups well below 0.9; if this
    // regresses the dedup silently broke.)
    assert!(
        module_count > 1,
        "witness must pull a multi-module closure to exercise dedup; got {module_count}"
    );
    assert!(
        interned.len() < plain.len(),
        "interned artifact ({}) must be smaller than plain ({})",
        interned.len(),
        plain.len()
    );
}

#[test]
fn decode_restores_rc_sharing_across_modules() {
    let (graph, si) = big_closure_graph();
    assert!(graph.modules.len() > 1, "need >1 module to test sharing");

    let payload = encode(&graph, &si);
    let (decoded, _top_si) = decode(&payload);

    // Pick two modules that share the global source-indices / intern-table. After
    // a serde round-trip WITHOUT interning these would be distinct allocations
    // (the cache-hit RAM blow-up); interning rebuilds one `Rc` per pool entry, so
    // modules referencing the same pooled value share a single allocation.
    let m0 = &decoded.modules[0];
    let mut shared_si = false;
    let mut shared_intern = false;
    for m in decoded.modules.iter().skip(1) {
        if Rc::ptr_eq(&m0.type_env.source_indices, &m.type_env.source_indices) {
            shared_si = true;
        }
        if Rc::ptr_eq(&m0.type_env.intern_table, &m.type_env.intern_table) {
            shared_intern = true;
        }
    }
    assert!(
        shared_si,
        "decode must restore Rc-sharing of source_indices across modules (the cache-hit RSS win)"
    );
    assert!(
        shared_intern,
        "decode must restore Rc-sharing of intern_table across modules"
    );
}
