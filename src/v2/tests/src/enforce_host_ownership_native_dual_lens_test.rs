//! Snappy ownership discriminator (msg_18888249): native substrate dual-lens
//! vs marshaled-root dual-lens on modeled ByteSize carrier.

use std::rc::Rc;

use v2_compiler::cli_run::make_eval_context;
use v2_compiler::coproduct_reflection::marshal_conj_type_item;
use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};
use v2_compiler::v2_compiler_infer_items::ItemKind;
use v2_compiler::v2_interpreter::{run_in_context_with_args, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const NATIVE_ENROLLMENT_ENTRY: &str =
    "src/v4/test/claim/lens_unit_modeling/modeled_unit_field_admitted_via_always_required_lenses.dag";
const NATIVE_WITNESS_FN: &str = "modeled_unit_field_admitted_via_always_required_lenses_holds";

const MARSHAL_HARNESS_ENTRY: &str = "src/v4/test/claim/manual/enforce_host_lens_bridge_harness.dag";
const MARSHAL_PROBE_FN: &str = "probe_lens_accepts_from_marshaled_root";
const MODELED_CARRIER_FIXTURE: &str =
    "src/v4/test/fixtures/enforce_host/modeled_carrier_memory_spec.dag";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn load_entry(relative: &str) -> Vec<Rc<SourceFile>> {
    let path = workspace_root().join(relative);
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {relative}: {e}"));
    resolve_imports_transitively_with_source_roots(relative, &content, &v4_source_roots())
        .into_iter()
        .map(|s| {
            Rc::new(SourceFile {
                path: s.path.clone(),
                content: s.content.clone(),
            })
        })
        .collect()
}

fn assert_resolved_ok(resolved: &v2_compiler::v2_compiler_compile::ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph, diagnostics: {msgs:?}"
    );
}

fn run_bool_witness(
    resolved: &v2_compiler::v2_compiler_compile::ResolvedPipelineResult,
    fn_name: &str,
) -> Result<bool, String> {
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = make_eval_context(graph, resolved.source_indices.clone());
    match run_in_context_with_args(&ctx, fn_name, &[], false) {
        Ok(Value::Bool(v)) => Ok(v),
        Ok(other) => Err(format!("{fn_name}: expected Bool, got {other:?}")),
        Err(e) => Err(format!("{fn_name}: {e}")),
    }
}

fn compile_bundle(
    entries: &[&str],
) -> Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult> {
    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    for entry in entries {
        sources.extend(load_entry(entry));
    }
    compile_to_resolved(Rc::new(sources))
}

fn memory_spec_marshaled_root(
    resolved: &v2_compiler::v2_compiler_compile::ResolvedPipelineResult,
) -> Value {
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = make_eval_context(graph, resolved.source_indices.clone());
    let item = graph
        .item_registry
        .values()
        .find(|info| info.kind == ItemKind::TypeItem && info.name == "MemorySpec")
        .and_then(|info| {
            graph
                .modules
                .iter()
                .flat_map(|m| m.items.iter())
                .find(|node| {
                    graph
                        .item_registry
                        .get(&node.name)
                        .is_some_and(|i| i.kind == ItemKind::TypeItem && i.name == info.name)
                })
        })
        .expect("MemorySpec type item");
    marshal_conj_type_item(&ctx, item).expect("marshal MemorySpec")
}

/// Ownership probe — run manually: `cargo test -p v2-compiler-tests enforce_host_ownership_native_dual_lens -- --nocapture --ignored`
#[test]
#[ignore = "snappy ownership discriminator msg_18888249 — manual bisect, not CI-enrolled"]
fn enforce_host_ownership_native_dual_lens_discriminator() {
    // Native path: substrate Node literals in enrollment witness (compile_to_resolved, no marshal).
    let native = compile_bundle(&[NATIVE_ENROLLMENT_ENTRY]);
    assert_resolved_ok(native.as_ref());
    let native_result = run_bool_witness(native.as_ref(), NATIVE_WITNESS_FN);
    eprintln!("NATIVE enrollment witness: {native_result:?}");

    // Marshaled path: same roster, marshaled MemorySpec root (current accept-arm probe).
    let marshal_bundle = compile_bundle(&[MARSHAL_HARNESS_ENTRY, MODELED_CARRIER_FIXTURE]);
    assert_resolved_ok(marshal_bundle.as_ref());
    let root = memory_spec_marshaled_root(marshal_bundle.as_ref());
    let graph = marshal_bundle.graph.as_ref().expect("graph");
    let ctx = make_eval_context(graph, marshal_bundle.source_indices.clone());
    let marshal_result = run_in_context_with_args(
        &ctx,
        MARSHAL_PROBE_FN,
        &[(Some("root".to_string()), root)],
        false,
    )
    .map_err(|e| e.to_string())
    .and_then(|v| match v {
        Value::Bool(b) => Ok(b),
        other => Err(format!("expected Bool, got {other:?}")),
    });
    eprintln!("MARSHAL bridge probe: {marshal_result:?}");

    match (&native_result, &marshal_result) {
        (Ok(true), Err(e)) if e.contains("fold_list_right expects a list, got Variant") => {
            eprintln!("DISCRIMINATOR: HOST-SIDE (marshal seam) — native OK, marshaled fold_list_right Variant");
        }
        (Ok(true), Ok(true)) => {
            eprintln!("DISCRIMINATOR: both paths green — prior failure may be flake");
        }
        (Err(e1), Err(e2)) if e1 == e2 => {
            panic!("DISCRIMINATOR: IN-MODEL — both paths fail identically: {e1}");
        }
        (Err(e1), Ok(_)) => {
            panic!("DISCRIMINATOR: unexpected — native failed, marshaled OK: native={e1}");
        }
        (Ok(false), _) => {
            panic!("DISCRIMINATOR: native enrollment witness returned false");
        }
        (_, Err(e)) => {
            panic!("DISCRIMINATOR: marshaled path error (not fold_list_right Variant): {e}");
        }
        other => panic!("DISCRIMINATOR: unclassified outcome: {other:?}"),
    }
}
