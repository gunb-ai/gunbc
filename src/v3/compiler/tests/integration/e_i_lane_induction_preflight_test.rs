//! **Layer:** integration
//!
//! Lane E-I Step 0: prove `CostBound`'s `SumBound` variant carries `terms: List<CostBound>`
//! in the bootstrap `Dag` (structural receipt, not label-only); `regen_bootstrap` must succeed.

use v3_compiler::dag::{Declaration, TypeConnective};
use v3_compiler::Dag;

fn conj_field_ty(payload: &Declaration, want_label: &str) -> v3_compiler::dag::DeclarationId {
    let TypeConnective::Conj { children } = &payload.connective else {
        panic!(
            "expected `{}` payload to be a record (Conj), got {:?}",
            want_label, payload.connective
        );
    };
    children
        .iter()
        .find(|c| c.label == want_label)
        .unwrap_or_else(|| {
            panic!(
                "expected field `{want_label}` on SumBound payload; have {:?}",
                children
                    .iter()
                    .map(|c| c.label.as_str())
                    .collect::<Vec<_>>()
            )
        })
        .ty
}

#[test]
fn e_i_lane_costbound_carries_list_of_costbound_in_bootstrap() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap dag should have no diagnostics, got {:?}",
        dag.diagnostics()
    );
    let cost_decl = dag
        .declaration_by_name("CostBound")
        .expect("CostBound declaration");
    let cost_id = cost_decl.id;
    let TypeConnective::Disj { variants } = &cost_decl.connective else {
        panic!(
            "expected CostBound to be a sum, got {:?}",
            cost_decl.connective
        );
    };
    let sum_bound = variants
        .iter()
        .find(|v| v.label == "SumBound")
        .expect("CostBound should include SumBound variant");
    let payload = dag.declaration(sum_bound.ty);
    let terms_ty = conj_field_ty(payload, "terms");
    let terms_decl = dag.declaration(terms_ty);
    match &terms_decl.connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let template_decl = dag.declaration(*template);
            assert_eq!(
                template_decl.name.as_deref(),
                Some("List"),
                "SumBound.terms must instantiate List<…>; template was {:?} ({:?})",
                template_decl.name,
                template_decl.connective
            );
            assert_eq!(
                arguments.len(),
                1,
                "List<CostBound> should have exactly one type argument; got {arguments:?}"
            );
            assert_eq!(
                arguments[0].value, cost_id,
                "SumBound.terms must be List<CostBound> (element type id should match CostBound)"
            );
        }
        other => {
            panic!("SumBound.terms must be List<CostBound> (Instantiation of List), got {other:?}")
        }
    }

    dag.declaration_by_name("sum_bound")
        .expect("sum_bound helper for List<CostBound>");
}
