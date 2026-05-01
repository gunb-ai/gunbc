//! Ratchet: substrate `signed_int_diagnostic_order` / `unsigned_int_diagnostic_order`
//! (`src/v3/std/integer_diagnostic_order.dag`) lower to `DeclarationRef` lists in bootstrap order.

use v3_compiler::dag::{Dag, FieldValue, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;

fn ref_list_declaration_names(dag: &Dag, data_name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(data_name)
        .unwrap_or_else(|| panic!("bootstrap Dag must declare `{data_name}`"));
    let Some(ValueBody::List(elements)) = &decl.value_body else {
        panic!(
            "`{data_name}` must lower to ValueBody::List, got {:?}",
            decl.value_body
        );
    };
    elements
        .iter()
        .map(|el| match el {
            FieldValue::Reference(id) => dag
                .declaration(*id)
                .name
                .clone()
                .unwrap_or_else(|| panic!("referenced decl has no name: {id:?}")),
            other => panic!("expected Reference list element, got {other:?}"),
        })
        .collect()
}

#[test]
fn integer_diagnostic_orders_are_declref_chains() {
    let dag = generated_full_bootstrap_dag();
    let signed = ref_list_declaration_names(&dag, "signed_int_diagnostic_order");
    assert_eq!(signed, vec!["Int8", "Int16", "Int32", "Int64", "Int128",]);
    let unsigned = ref_list_declaration_names(&dag, "unsigned_int_diagnostic_order");
    assert_eq!(
        unsigned,
        vec!["UInt8", "UInt16", "UInt32", "UInt64", "UInt128",]
    );
}
