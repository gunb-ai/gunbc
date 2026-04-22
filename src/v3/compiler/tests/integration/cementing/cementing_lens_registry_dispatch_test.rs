//! **Layer:** integration
//!
//! Band-C cementing dispatch for lenses enumerated in `regen.dag` and
//! described in `docs/v3-lens-capability-register.md`.
//!
//! - `TESTING.md` — *Cementing tests (Band C — lens subsumption)*
//! - `src/v3/compiler/regen.dag` — header comment on cementing dispatch

use std::path::PathBuf;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Behavior;
use v3_compiler::lens_provenance::{origin_of, Origin};

fn origin_label(origin: &Origin) -> &'static str {
    match origin {
        Origin::NoProducer => "NoProducer",
        Origin::MissingPort => "MissingPort",
        Origin::MissingBehavior => "MissingBehavior",
        Origin::Source { .. } => "Source",
        Origin::Computed { .. } => "Computed",
        Origin::Selected { .. } => "Selected",
        Origin::Accumulated { .. } => "Accumulated",
    }
}

/// Pairs of (`regen_lens --lens <name>` registry key, cementing module stem
/// under `tests/integration/cementing/` without `.rs`).
///
/// Append an entry when `docs/v3-lens-capability-register.md` promotes a
/// lens to `BEHAVIORALLY COMPLETE` **and** the v2 counterpart column names a
/// concrete v2 artifact (not `None (v3-native)` / not `N/A`). Land the new
/// `cementing/<stem>.rs` module and a `#[path = ...]` line in
/// `tests/integration.rs` in the same PR.
const CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS: &[(&str, &str)] = &[];

fn find_bind_value_port(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

#[test]
fn provenance_origin_of_cements_behavior_complete_row_on_minimal_ports() {
    // Register row: `provenance.dag` — BEHAVIORALLY COMPLETE, v3-native.
    // Integration crate cannot reach `Dag`'s `pub(crate)` builder helpers
    // (`alloc_port_with_shape`); `compile_to_dag` fixtures still cement the
    // shipped `origin_of` contract on the live lowering path. Exhaustive
    // `NoProducer` / `Missing*` cases stay in `lib.rs::lens_provenance::tests`.
    let dag = compile_to_dag("let lit: Int = 7", "cementing_provenance_lit.v3").expect("compiles");
    assert_eq!(
        origin_label(&origin_of(&dag, &find_bind_value_port(&dag, "lit"))),
        "Source"
    );

    let dag =
        compile_to_dag("let sum: Int = 1 + 2", "cementing_provenance_sum.v3").expect("compiles");
    assert_eq!(
        origin_label(&origin_of(&dag, &find_bind_value_port(&dag, "sum"))),
        "Computed"
    );
}

#[test]
fn cementing_test_modules_exist_for_escalated_v2_complete_registry_claims() {
    let cementing_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("integration")
        .join("cementing");
    for (registry_name, stem) in CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS {
        let path = cementing_dir.join(format!("{stem}.rs"));
        assert!(
            path.is_file(),
            "registry lens `{registry_name}` is listed for v2-complete cementing; expected cementing module at {}",
            path.display()
        );
    }
}
