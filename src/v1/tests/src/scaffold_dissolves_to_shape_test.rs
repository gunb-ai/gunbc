use v1_compiler::coproduct_reflection::decl_facts_for_roots;
use v1_compiler::v1_compiler_infer_items::item_kind;
use v1_compiler::v1_std_core::{ExprData, VarBindingKind};

use crate::helpers::workspace_root;

#[test]
fn dump_bytes_scaffold_dissolves_to_field_shape() {
    let roots = vec![
        workspace_root().join("dag").to_string_lossy().into_owned(),
        workspace_root().join("src/v2").to_string_lossy().into_owned(),
    ];
    let facts = decl_facts_for_roots(&roots);
    let fact = facts
        .iter()
        .find(|f| f.qualified_name == "std.bytes.bytes_seam_host_realization_marker")
        .expect("bytes marker");
    let body = fact.node.body.as_ref().expect("body");
    eprintln!("body expr_data: {:?}", body.expr_data);
    eprintln!("body name: {:?}", body.name);
    eprintln!("body children: {}", body.children.len());
    for (i, child) in body.children.iter().enumerate() {
        eprintln!("child{i} expr_data: {:?}", child.expr_data);
        eprintln!("child{i} name: {:?}", child.name);
        if let ExprData::ExprVar { binding_kind } = child.expr_data.as_ref() {
            eprintln!("child{i} binding_kind: {:?}", binding_kind);
        }
        for (j, gc) in child.children.iter().enumerate() {
            eprintln!("  gc{j} expr_data: {:?}", gc.expr_data);
            eprintln!("  gc{j} name: {:?}", gc.name);
            if let ExprData::ExprVar { binding_kind } = gc.expr_data.as_ref() {
                eprintln!("  gc{j} binding_kind: {:?}", binding_kind);
                if let Some(bk) = binding_kind {
                    eprintln!(
                        "  gc{j} is variant binding: {}",
                        matches!(bk.as_ref(), VarBindingKind::VariantValueBinding { .. })
                    );
                }
            }
        }
    }
    eprintln!("kind: {:?}", item_kind(fact.node.clone()));
}
