//! **Layer:** integration
//!
//! R3 §1.8 gate #90 (`lens_enforcement_carrier_landed`) — T-Lens-Application-Surface
//! Slice B receipt: canonical `LensEnforcement` + `EnforceableLens` **data** rows
//! co-located with each `Lens<C>` producer where LAS packaging lands today:
//! complexity (`complexity.dag`), symbolic cost (`cost.dag`), timing (`timing_lens.dag`).
//! Stage 2e parallelism (`parallelism.dag`) defers its LAS enforcement packaging —
//! `import lenses.parallelism { analyze_parallelism }` is not yet resolver-clean for a sibling
//! LAS module without merging regen emit paths (tracked with Cluster F #81 / gate #95 sequencing).
//!
//! **Bootstrap vs lens authorities:** `generated_full_bootstrap_dag()` concatenates `src/v3/std`
//! (among others), not `src/v3/lenses/`. `timing_lens.dag` therefore pins against full bootstrap;
//! `complexity.dag` / `cost.dag` are compiled standalone via `cached_compile_to_dag` like other
//! lens-library receipts.

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{Dag, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;

#[test]
fn r3_gate_90_timing_lens_enforcement_carrier_bundle_locked() {
    let boot = generated_full_bootstrap_dag();
    assert_lens_enforcement_bundle(&boot, "timing_lens.dag", "timing_enforcement");
}

#[test]
fn r3_gate_90_complexity_lens_enforcement_carrier_bundle_locked() {
    let complexity = cached_compile_to_dag(
        include_str!("../../../lenses/complexity.dag"),
        "src/v3/lenses/complexity.dag",
    );
    assert_lens_enforcement_bundle(&complexity, "complexity.dag", "complexity_enforcement");
}

#[test]
fn r3_gate_90_cost_lens_enforcement_carrier_bundle_locked() {
    let cost = cached_compile_to_dag(
        include_str!("../../../lenses/cost.dag"),
        "src/v3/lenses/cost.dag",
    );
    assert_lens_enforcement_bundle(&cost, "cost.dag", "cost_enforcement");
}

fn assert_lens_enforcement_bundle(dag: &Dag, stem_suffix: &str, enforcement: &str) {
    let enforce = dag
        .declaration_by_name(enforcement)
        .unwrap_or_else(|| panic!("DAG missing `{enforcement}` data row"));
    assert!(
        enforce.span.file.ends_with(stem_suffix),
        "`{enforcement}` must remain authored in `*{stem_suffix}`; got {:?}",
        enforce.span.file
    );
    let fields = match enforce.value_body.as_ref() {
        Some(ValueBody::Structural { fields }) => fields,
        other => panic!(
            "`{enforcement}` must carry a structural `data` body in compiled DAG; got {other:?}"
        ),
    };
    let labels: Vec<&str> = fields.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(
        labels,
        vec!["project", "violates"],
        "`{enforcement}` must instantiate `LensEnforcement` with project + violates"
    );

    let enforceable = enforcement
        .strip_suffix("_enforcement")
        .unwrap_or(enforcement);
    let enforceable = format!("{enforceable}_enforceable");
    let bundle = dag.declaration_by_name(&enforceable).unwrap_or_else(|| {
        panic!("DAG missing `{enforceable}` data row (paired with `{enforcement}`)")
    });
    assert!(
        bundle.span.file.ends_with(stem_suffix),
        "`{enforceable}` must remain authored in `*{stem_suffix}`; got {:?}",
        bundle.span.file
    );
    let bundle_fields = match bundle.value_body.as_ref() {
        Some(ValueBody::Structural { fields }) => fields,
        other => panic!(
            "`{enforceable}` must carry a structural `data` body in compiled DAG; got {other:?}"
        ),
    };
    let bundle_labels: Vec<&str> = bundle_fields.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(
        bundle_labels,
        vec!["lens", "enforcement"],
        "`{enforceable}` must instantiate `EnforceableLens`"
    );
}
