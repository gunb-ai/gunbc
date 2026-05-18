//! **Layer:** integration
//!
//! T-19 Wave-0: `src/v4/lens/testgen.dag` parses and exposes manual-anchor-key-driven
//! `Generator` wiring (`kind` + `t19_anchor` + `classification` + `slot: TestgenConcept`).
//! **Note:** `compile_to_dag` on this module alone does not resolve `import v4.std.*` peers
//! (Import lowering is still M2-scoped); full merge compile lands with cross-file M2 per TASKS T-19.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

#[test]
fn v4_lens_testgen_wave0_substrate_parses() {
    let m = parse_module(
        include_str!("../../../../v4/lens/testgen.dag"),
        "src/v4/lens/testgen.dag",
    );

    assert_eq!(
        module_paths(&m),
        vec![vec!["v4", "lens", "testgen"]],
        "T-19 authority module should remain v4.lens.testgen"
    );
    assert_eq!(
        function_count(&m, "bootstrap_claim_generator_for_manual_anchor"),
        1,
        "T-19 Wave-0: single generator entrypoint keyed by T19ManualAnchorKey"
    );
    assert_eq!(function_count(&m, "assert_kind_for_manual_anchor"), 1);
    assert_eq!(function_count(&m, "testgen_concept_for_manual_anchor"), 1);
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn function_count(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> usize {
    module
        .items
        .iter()
        .filter(|item| match item {
            SurfaceItem::Fn {
                name: item_name, ..
            }
            | SurfaceItem::FnExternalBody {
                name: item_name, ..
            } => item_name == name,
            _ => false,
        })
        .count()
}
