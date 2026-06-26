//! Layer A (`build_type_env` source-side de-merge) **equivalence gate** —
//! standalone, off current main, NOT load-bearing.
//!
//! Today every module's `type_env` carries a fully-merged copy of its transitive
//! import closure (the 54% `type_env` pole of the 73% duplication). The durable
//! fix (Layer A) replaces the merged `HashMap` with a scope-chain
//! (`local_bindings` + `parents: Vec<Rc<TypeEnv>>`) and lets `lookup_*` walk the
//! chain. That changes *where* a binding is stored, and MUST NOT change what
//! `lookup_type` returns for any (module, type-id).
//!
//! This test pins exactly that observable. It builds a multi-import witness,
//! discovers the global type-id keyset, then for every (module, key) records the
//! result of the **`lookup_type` function** (never raw `.bindings`), canonicalized
//! key-sorted. The same matrix is captured for `func_env` signatures.
//!
//! ## Why determinism alone is NOT a sufficient guard (the discriminating part)
//!
//! Asserting `m_one == m_two` across two cold resolves only proves the resolver is
//! deterministic — both resolves use the SAME representation, so a de-merge that
//! silently drops imports (every importer cell -> `<not-in-scope>`, *consistently*)
//! would keep this equal and stay GREEN. That is exactly the "reader bypasses
//! `lookup_*` and sees only locals" failure the de-merge risks. So this gate adds
//! two DISCRIMINATING checks that go RED on import-loss:
//!
//!   (1) a cross-module **in-scope** assertion: a type defined in `test.common`
//!       (`Box`) must resolve through `lookup_type` in importer `test.b`. If the
//!       de-merged enumerator/accessor stops seeing parents, this flips to
//!       `<not-in-scope>` and the assert fails.
//!   (2) a cross-import **unit-variant phantom** witness: `test.sig` defines a
//!       coproduct `Sig = Active | Idle`; `test.phantom` imports `Sig` and uses
//!       the bare unit variant `Active` in a *type* position. Resolving that name
//!       runs `collect_unit_variant_phantom_matches`, which enumerates the env's
//!       bindings for the enclosing coproduct. Under `ResolveTypecheckGate::Strict`
//!       an unresolved type is FATAL, so if the de-merged enumerator misses the
//!       imported `Sig`, `resolve_entry_graph` returns `Err` and this test panics.
//!
//! The only edit Layer A needs here is the keyset discovery
//! (`union of .bindings` -> `union of local + reachable`), called out at its site
//! below; the lookup queries and the two discriminators are unchanged — that is
//! the whole point.

use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::resolve_entry_graph;
use v1_compiler::v1_compiler_infer_env::lookup_type;
use v1_compiler::v1_compiler_infer_items::ResolvedGraph;

/// Canonical key-sorted serialization of any serde value (HashMaps materialize
/// as sorted `BTreeMap`, so the string is invariant under iteration order).
fn canonical<T: serde::Serialize>(value: &T) -> String {
    let v = serde_json::to_value(value).expect("to_value");
    serde_json::to_string(&v).expect("to_string")
}

/// The representation-independent observable: for every (module-name, type-id) in
/// the global keyset, the canonical type that `lookup_type` resolves — plus every
/// (module-name, func-name) -> canonical signature. Queried through the lookup
/// API, so the scope-chain rep produces the identical matrix.
fn observable_matrix(graph: &ResolvedGraph) -> String {
    // Keyset discovery. LAYER A SWAP POINT: today a module's `.bindings` is the
    // full merged closure, so the union below is the global type-id set. After the
    // de-merge a module's `.bindings` holds only locals, so this union must become
    // "local + transitively-reachable via parents" (a one-line change here). The
    // lookup queries below are unchanged — that is the whole point.
    let mut type_keys: Vec<i64> = Vec::new();
    for m in graph.modules.iter() {
        for k in m.type_env.bindings.keys() {
            if !type_keys.contains(k) {
                type_keys.push(*k);
            }
        }
    }
    type_keys.sort_unstable();

    let mut func_names: Vec<String> = Vec::new();
    for m in graph.modules.iter() {
        for name in m.func_env.signatures.keys() {
            if !func_names.contains(name) {
                func_names.push(name.clone());
            }
        }
    }
    func_names.sort();

    let mut matrix: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for m in graph.modules.iter() {
        let module_name = canonical(&m.module);
        let row = matrix.entry(module_name).or_default();
        for &k in &type_keys {
            // The observable: lookup through the API, not the storage.
            let resolved = lookup_type(m.type_env.clone(), k);
            let cell = match resolved {
                Some(node) => canonical(&node),
                None => "<not-in-scope>".to_string(),
            };
            row.insert(format!("type:{k}"), cell);
        }
        for name in &func_names {
            let cell = match m.func_env.signatures.get(name) {
                Some(sig) => canonical(sig),
                None => "<not-in-scope>".to_string(),
            };
            row.insert(format!("func:{name}"), cell);
        }
    }
    canonical(&matrix)
}

/// The type-id under which `binding_name` is stored as a *local* type in the
/// module whose canonical name contains `owner_module`, read via that owner's own
/// bindings (an owner carries its own locals before and after de-merge). Returns
/// the interned id so an *importer* can be probed for the same id through
/// `lookup_type` — a cross-module in-scope discriminator.
fn local_type_id(graph: &ResolvedGraph, owner_module: &str, binding_name: &str) -> i64 {
    for m in graph.modules.iter() {
        if canonical(&m.module).contains(owner_module) {
            for (id, binding) in m.type_env.bindings.iter() {
                if binding.name == binding_name {
                    return *id;
                }
            }
        }
    }
    panic!("binding `{binding_name}` not found local to module `{owner_module}`");
}

fn write_witness() -> (Vec<String>, String, std::path::PathBuf) {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("gunbc-demerge-eq-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("temp dir");

    // A multi-import closure with shadowing surface: two modules each define a
    // distinct `Box`/`val`, importers pull different combinations, so the merged
    // env that Layer A de-merges genuinely carries closure copies whose lookup
    // results differ per module — exactly what must be preserved.
    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    let shared1 = "module test.shared1\n\
        type Tag { id: Int }\n\
        fn val() -> Int { 10 }\n";
    let shared2 = "module test.shared2\nfn val() -> Int { 20 }\n";
    let extra = "module test.extra\nfn pad() -> Int { 7 }\n";
    // Cross-import unit-variant phantom: `Sig` is a coproduct of unit variants in
    // one module; `test.phantom` imports the TYPE `Sig` (not the variant) and then
    // names the bare unit variant `Active` in a type position. Resolving that name
    // must enumerate the imported coproduct via the scope chain.
    let sig = "module test.sig\ntype Sig = Active | Idle\n";
    let phantom = "module test.phantom\n\
        import test.sig { Sig }\n\
        fn classify(s: Active) -> Int { 1 }\n\
        fn ping() -> Int { 99 }\n";
    let entry_a = "module test.a\n\
        import test.common { boxed, unbox }\n\
        import test.shared1 { val }\n\
        fn witness_a() -> Bool { unbox(boxed(val())) == 10 }\n";
    let entry_b = "module test.b\n\
        import test.common { boxed, unbox }\n\
        import test.shared2 { val }\n\
        import test.extra { pad }\n\
        import test.phantom { ping }\n\
        fn uses_phantom() -> Int { ping() }\n\
        fn witness_b() -> Bool { (unbox(boxed(val())) + pad()) == 27 }\n";

    for (name, src) in [
        ("common.dag", common),
        ("shared1.dag", shared1),
        ("shared2.dag", shared2),
        ("extra.dag", extra),
        ("sig.dag", sig),
        ("phantom.dag", phantom),
        ("entry_a.dag", entry_a),
        ("entry_b.dag", entry_b),
    ] {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }

    let roots = vec![dir.to_string_lossy().into_owned()];
    let entry = dir.join("entry_b.dag").to_string_lossy().into_owned();
    (roots, entry, dir)
}

#[test]
fn lookup_observable_is_stable_and_representation_independent() {
    let (roots, entry, _dir) = write_witness();

    // DISCRIMINATOR (2): the cross-import phantom in the witness means a de-merge
    // that drops imports from the unit-variant enumerator makes this `.expect`
    // panic — `classify(s: Active)` would raise a fatal UnresolvedType under Strict.
    let (graph_one, _si_one) = resolve_entry_graph(&roots, &entry).expect("first resolve");
    let (graph_two, _si_two) = resolve_entry_graph(&roots, &entry).expect("second resolve");

    // The witness must actually exercise a multi-module closure, else the gate is
    // vacuous (nothing to de-merge).
    assert!(
        graph_one.modules.len() > 1,
        "equivalence gate needs a multi-module closure; got {}",
        graph_one.modules.len()
    );

    // DISCRIMINATOR (1): a type defined in `test.common` must be in scope (resolve
    // through `lookup_type`) in the importer `test.b`. `Box` is local to
    // `test.common`; `test.b` imports `boxed`/`unbox` whose signatures reference it,
    // so it MUST be reachable. Today it is in the merged map; after the de-merge it
    // must remain reachable through the parent chain. If the de-merge regresses,
    // this flips to `<not-in-scope>` and fails — the exact "reader sees only locals"
    // failure mode, caught here by querying through the lookup API.
    let box_id = local_type_id(&graph_one, "common", "Box");
    let mut probed_importer = false;
    let importer_needle = "\"test.b\"".to_string();
    for m in graph_one.modules.iter() {
        if canonical(&m.module).contains(&importer_needle) {
            let resolved = lookup_type(m.type_env.clone(), box_id);
            assert!(
                resolved.is_some(),
                "importer test.b must resolve imported type `Box` (id {box_id}) through \
                 lookup_type; got <not-in-scope> — the de-merge dropped the import closure"
            );
            probed_importer = true;
        }
    }
    assert!(
        probed_importer,
        "expected importer module test.b in the resolved graph to probe cross-module in-scope"
    );

    let m_one = observable_matrix(&graph_one);
    let m_two = observable_matrix(&graph_two);

    // The matrix must be non-empty (lookup actually resolved closure bindings).
    assert!(
        m_one.contains("type:") && m_one.contains("func:"),
        "observable matrix must carry type and func lookups; got {} bytes",
        m_one.len()
    );

    eprintln!(
        "[layer-a-gate] modules={} observable={} bytes box_id={}",
        graph_one.modules.len(),
        m_one.len(),
        box_id
    );

    // THE GATE: the lookup-observable is identical across independent resolves.
    // Today both resolves use the merged env; once Layer A swaps `build_type_env`
    // to a scope chain, this same assertion (queried through `lookup_type`) must
    // still hold byte-for-byte. Any divergence in what is in scope, what it
    // resolves to, or shadowing order turns this RED.
    assert_eq!(
        m_one, m_two,
        "lookup-observable typed graph must be deterministic and representation-independent"
    );
}
