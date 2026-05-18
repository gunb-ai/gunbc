//! **Layer:** integration
//!
//! Ratchet P9's LLVM instruction cost-table move to the cost-lens authority.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

#[test]
fn v4_lens_cost_owns_llvm_instruction_cost_table() {
    let cost_lens = parse_module(
        include_str!("../../../../v4/lens/cost.dag"),
        "src/v4/lens/cost.dag",
    );
    let llvm_ir = parse_module(
        include_str!("../../../../v4/extdeps/languages/llvm_ir.dag"),
        "src/v4/extdeps/languages/llvm_ir.dag",
    );

    assert_eq!(
        module_paths(&cost_lens),
        vec![vec!["v4", "lens", "cost"]],
        "P9 authority module should remain v4.lens.cost"
    );
    assert_eq!(
        function_count(&cost_lens, "llvm_instruction_cost"),
        1,
        "P9 requires exactly one parsed llvm_instruction_cost declaration under v4/lens/cost.dag"
    );
    assert_eq!(
        function_count(&llvm_ir, "llvm_instruction_cost"),
        0,
        "llvm_ir.dag must not keep a second parsed llvm_instruction_cost authority"
    );
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
