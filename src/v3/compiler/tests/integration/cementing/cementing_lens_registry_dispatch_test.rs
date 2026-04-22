//! **Layer:** integration
//!
//! Band-C cementing dispatch for lenses enumerated in `regen.dag` and
//! described in `docs/v3-lens-capability-register.md`.
//!
//! - `TESTING.md` — *Cementing tests (Band C — lens subsumption)*
//! - `src/v3/compiler/regen.dag` — header comment on cementing dispatch

use std::path::PathBuf;

use v3_compiler::dag::{Dag, LiteralBits};
use v3_compiler::diagnostics::SourceSpan;
use v3_compiler::lens_provenance::{origin_of, Origin};

fn span() -> SourceSpan {
    SourceSpan::new("<cementing-provenance>", 0, 0)
}

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

#[test]
fn provenance_origin_of_cements_behavior_complete_row_on_minimal_ports() {
    // Register row: `provenance.dag` — BEHAVIORALLY COMPLETE, v3-native.
    // Cementing here pins the public `Origin` classification contract the
    // same way a future v2-oracle test would pin cross-implementation parity.
    let mut dag = Dag::new();
    let int_shape = dag.int_shape().expect("bootstrap Int");
    let param = dag.alloc_port_with_shape(int_shape);
    assert_eq!(origin_label(&origin_of(&dag, &param)), "NoProducer");

    let literal = dag.push_value(LiteralBits::Int(7), span());
    assert_eq!(origin_label(&origin_of(&dag, &literal)), "Source");
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
