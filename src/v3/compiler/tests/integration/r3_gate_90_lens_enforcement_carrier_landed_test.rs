//! **Layer:** integration
//!
//! R3 §1.8 gate #90 (`lens_enforcement_carrier_landed`) — T-Lens-Application-Surface
//! Slice B receipt: canonical `LensEnforcement` + `EnforceableLens` **data** rows
//! co-located with each in-scope `Lens<C>` producer (`complexity`, symbolic `cost`,
//! Stage 2e `parallelism`, `timing`).

use v3_compiler::dag::{Dag, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;

#[test]
fn r3_gate_90_lens_enforcement_carrier_instances_exist_in_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    assert_lens_enforcement_bundle(&dag, "complexity.dag", "complexity_enforcement");
    assert_lens_enforcement_bundle(&dag, "cost.dag", "cost_enforcement");
    assert_lens_enforcement_bundle(&dag, "parallelism.dag", "parallelism_enforcement");
    assert_lens_enforcement_bundle(&dag, "timing_lens.dag", "timing_enforcement");
}

fn assert_lens_enforcement_bundle(dag: &Dag, stem_suffix: &str, enforcement: &str) {
    let enforce = dag
        .declaration_by_name(enforcement)
        .unwrap_or_else(|| panic!("bootstrap missing `{enforcement}` data row"));
    assert!(
        enforce.span.file.ends_with(stem_suffix),
        "`{enforcement}` must remain authored in `*{stem_suffix}`; got {:?}",
        enforce.span.file
    );
    let fields = match enforce.value_body.as_ref() {
        Some(ValueBody::Structural { fields }) => fields,
        other => panic!(
            "`{enforcement}` must carry a structural `data` body at bootstrap HEAD; got {other:?}"
        ),
    };
    let labels: Vec<&str> = fields.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(
        labels,
        vec!["project", "violates"],
        "`{enforcement}` must instantiate `LensEnforcement` with project + violates"
    );

    let enforceable = enforcement.strip_suffix("_enforcement").unwrap_or(enforcement);
    let enforceable = format!("{enforceable}_enforceable");
    let bundle = dag.declaration_by_name(&enforceable).unwrap_or_else(|| {
        panic!("bootstrap missing `{enforceable}` data row (paired with `{enforcement}`)")
    });
    assert!(
        bundle.span.file.ends_with(stem_suffix),
        "`{enforceable}` must remain authored in `*{stem_suffix}`; got {:?}",
        bundle.span.file
    );
    let bundle_fields = match bundle.value_body.as_ref() {
        Some(ValueBody::Structural { fields }) => fields,
        other => panic!(
            "`{enforceable}` must carry a structural `data` body at bootstrap HEAD; got {other:?}"
        ),
    };
    let bundle_labels: Vec<&str> = bundle_fields
        .iter()
        .map(|(l, _)| l.as_str())
        .collect();
    assert_eq!(
        bundle_labels,
        vec!["lens", "enforcement"],
        "`{enforceable}` must instantiate `EnforceableLens`"
    );
}
