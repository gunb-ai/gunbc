//! **Layer:** integration
//!
//! Carrier-shape ratchet for `src/v3/std/bin_shim.dag`, per
//! `docs/design-pb-runtime-interpreter.md` sections 4.2 and 5.4.
//!
//! This intentionally covers only the substrate carrier. PB-owned instance rows
//! under `dsl/std/runtime/bin_shims/` and emitter/runtime behavior land in
//! later retirement slices.

use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn decl_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"))
        .id
}

fn bin_shim_fields(dag: &Dag) -> Vec<(&str, DeclarationId)> {
    let decl = dag
        .declaration_by_name("BinShim")
        .expect("`BinShim` missing from full bootstrap");
    match &decl.connective {
        TypeConnective::Conj { children } => children
            .iter()
            .map(|field| (field.label.as_str(), field.ty))
            .collect(),
        other => panic!("`BinShim` must be a record carrier, got {other:?}"),
    }
}

#[test]
fn bin_shim_carrier_lives_in_v3_std_authority() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name("BinShim")
        .expect("`BinShim` missing from full bootstrap");

    assert_eq!(
        decl.span.file, "src/v3/std/bin_shim.dag",
        "`BinShim` carrier authority must stay in the staged v3 std surface; \
         concrete shim rows belong under `dsl/std/runtime/bin_shims/`"
    );
}

#[test]
fn bin_shim_carrier_has_locked_three_field_shape() {
    let dag = generated_full_bootstrap_dag();
    let labels: Vec<&str> = bin_shim_fields(&dag)
        .into_iter()
        .map(|(label, _)| label)
        .collect();

    assert_eq!(
        labels,
        ["entrypoint_name", "description", "entry"],
        "`BinShim` must remain metadata plus entry declaration; adding a \
         pipeline-step DSL or extra emitter state requires a substrate \
         amendment"
    );
}

#[test]
fn bin_shim_field_types_match_design_lock() {
    let dag = generated_full_bootstrap_dag();
    let fields = bin_shim_fields(&dag);

    let expected = [
        ("entrypoint_name", decl_id_by_name(&dag, "NonEmptyStr")),
        ("description", decl_id_by_name(&dag, "String")),
        ("entry", decl_id_by_name(&dag, "DeclarationRef")),
    ];

    assert_eq!(
        fields, expected,
        "`BinShim.entry` stays a DeclarationRef to a .dag `() -> \
         std.process.ProcessExit` function until DeclarationRef signature \
         refinement lands"
    );
}
